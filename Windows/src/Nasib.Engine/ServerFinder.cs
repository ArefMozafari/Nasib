namespace Nasib.Engine;

/// <summary>
/// Finds a Windscribe datacenter and protocol that carries traffic right now, connects to it, and keeps it
/// connected. The Windows counterpart of Engine/ws-find.sh, with the same three modes:
/// Find searches once, Watch searches only when the tunnel stopped working, and Survey tries every pair.
/// </summary>
public sealed class ServerFinder
{
    private const string ListSeparator = ", ";

    private readonly StatePaths paths;
    private readonly DatacenterCatalog catalog;
    private readonly TunnelController tunnel;
    private readonly ReachabilityFilter reachabilityFilter;
    private readonly AttemptLog attemptLog;

    // Only one search runs at a time: the watcher fires every minute and a search takes longer than that.
    private readonly SemaphoreSlim searchGate = new(1, 1);

    public ServerFinder(
        StatePaths paths,
        DatacenterCatalog catalog,
        TunnelController tunnel,
        ReachabilityFilter reachabilityFilter,
        AttemptLog attemptLog)
    {
        this.paths = paths;
        this.catalog = catalog;
        this.tunnel = tunnel;
        this.reachabilityFilter = reachabilityFilter;
        this.attemptLog = attemptLog;
    }

    /// <summary>
    /// Raised on whichever thread the search runs on.
    /// </summary>
    public event Action<EngineEvent>? EventRaised;

    /// <summary>
    /// Tries the last pair that worked, then every reachable datacenter by rank on every protocol by rank,
    /// and stops at the first pair that carries traffic. Returns whether one did.
    /// </summary>
    public Task<bool> FindAsync(CancellationToken cancellationToken = default)
    {
        return RunExclusiveAsync(scope => SearchAsync(scope, isSurvey: false, cancellationToken));
    }

    /// <summary>
    /// Does nothing when the tunnel is up and a blocked site loads through it; otherwise searches as Find does.
    /// </summary>
    public Task<bool> WatchAsync(CancellationToken cancellationToken = default)
    {
        return RunExclusiveAsync(async scope =>
        {
            if (await tunnel.WorksAsync(cancellationToken))
            {
                return true;
            }
            Raise(new TunnelDownEvent());
            return await SearchAsync(scope, isSurvey: false, cancellationToken);
        });
    }

    /// <summary>
    /// Tries every pair and logs each result without stopping at a success, then reconnects the last pair
    /// that worked. Cancelling it reconnects that pair too, so the user is put back online.
    /// </summary>
    public Task<bool> SurveyAsync(CancellationToken cancellationToken = default)
    {
        return RunExclusiveAsync(async scope =>
        {
            try
            {
                return await SearchAsync(scope, isSurvey: true, cancellationToken);
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
            {
                Raise(new SurveyStoppedEvent());
                await tunnel.DisconnectAndWaitAsync(CancellationToken.None);
                return await ReconnectLastGoodAsync(CancellationToken.None);
            }
        });
    }

    private async Task<bool> RunExclusiveAsync(Func<SearchScope, Task<bool>> search)
    {
        if (!searchGate.Wait(0))
        {
            Raise(new MessageEvent("search already running"));
            return false;
        }
        try
        {
            var configuration = EngineConfiguration.Load(paths.ConfigurationFile);
            var datacenters = catalog.Load(configuration);
            if (datacenters.Count == 0)
            {
                Raise(new MessageEvent("no datacenters match the cities in Settings"));
                return false;
            }
            return await search(new SearchScope(configuration, datacenters));
        }
        finally
        {
            searchGate.Release();
        }
    }

    private async Task<bool> SearchAsync(SearchScope scope, bool isSurvey, CancellationToken cancellationToken)
    {
        var datacenters = scope.Datacenters;
        if (!isSurvey && await ReconnectLastGoodAsync(cancellationToken))
        {
            return true;
        }

        var candidates = await reachabilityFilter.ReachableAsync(datacenters, cancellationToken);
        if (candidates.Count == 0)
        {
            // The filter may be wrong on a bad day, so fall back to trying everything.
            Raise(new MessageEvent("no address answered; trying all locations"));
            candidates = datacenters;
        }

        var recentAttempts = attemptLog.ReadRecent(SuccessRanker.RankWindowLines);
        var nicknames = SuccessRanker.Rank(candidates.Select(datacenter => datacenter.Nickname), recentAttempts, attempt => attempt.Nickname);
        var protocols = SuccessRanker.Rank(scope.Configuration.ProtocolOrder, recentAttempts, attempt => attempt.Protocol);
        Raise(new MessageEvent("protocol order: " + string.Join(ListSeparator, protocols)));
        Raise(new MessageEvent("location order: " + string.Join(ListSeparator, nicknames)));

        foreach (var protocol in protocols)
        {
            foreach (var nickname in nicknames)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (await TryPairAsync(nickname, protocol, cancellationToken))
                {
                    if (!isSurvey)
                    {
                        return true;
                    }
                    await tunnel.DisconnectAndWaitAsync(cancellationToken);
                }
            }
        }

        if (isSurvey)
        {
            Raise(new SurveyFinishedEvent());
            return await ReconnectLastGoodAsync(cancellationToken);
        }
        Raise(new NothingWorkedEvent());
        return false;
    }

    private async Task<bool> ReconnectLastGoodAsync(CancellationToken cancellationToken)
    {
        var lastGood = attemptLog.ReadLastGood();
        return lastGood is not null && await TryPairAsync(lastGood.Nickname, lastGood.Protocol, cancellationToken);
    }

    /// <summary>
    /// One attempt: connect, check that a blocked site loads, and log the result.
    /// </summary>
    private async Task<bool> TryPairAsync(string nickname, string protocol, CancellationToken cancellationToken)
    {
        Raise(new TryingEvent(nickname, protocol));
        await tunnel.ConnectAsync(nickname, protocol, cancellationToken);
        if (await tunnel.WorksAsync(cancellationToken))
        {
            attemptLog.Record(nickname, protocol, succeeded: true);
            attemptLog.WriteLastGood(new ServerPair(nickname, protocol));
            Raise(new ConnectedEvent(nickname, protocol));
            return true;
        }
        attemptLog.Record(nickname, protocol, succeeded: false);
        await tunnel.DisconnectAndWaitAsync(cancellationToken);
        return false;
    }

    private void Raise(EngineEvent engineEvent)
    {
        EventRaised?.Invoke(engineEvent);
    }

    /// <summary>
    /// The settings and datacenters one search works from, read once when it starts.
    /// </summary>
    private sealed record SearchScope(EngineConfiguration Configuration, IReadOnlyList<Datacenter> Datacenters);
}
