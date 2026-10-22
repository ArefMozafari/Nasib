namespace Nasib.Engine;

/// <summary>
/// Connects, disconnects and checks the tunnel through the CLI, with the waits the Windscribe app needs.
/// </summary>
public sealed class TunnelController
{
    private readonly IWindscribeCli cli;
    private readonly IBlockedSiteProbe blockedSiteProbe;
    private readonly IWaiter waiter;
    private readonly EngineTimings timings;

    public TunnelController(IWindscribeCli cli, IBlockedSiteProbe blockedSiteProbe, IWaiter waiter, EngineTimings timings)
    {
        this.cli = cli;
        this.blockedSiteProbe = blockedSiteProbe;
        this.waiter = waiter;
        this.timings = timings;
    }

    public async Task<TunnelStatus> ReadStatusAsync(CancellationToken cancellationToken)
    {
        return TunnelStatus.Parse(await cli.ReadStatusAsync(cancellationToken));
    }

    /// <summary>
    /// Disconnects and waits until the CLI reports Disconnected or an error. The app ignores a connect issued
    /// while it is still tearing down the previous tunnel, and it keeps reporting an error such as "Error: 11"
    /// after a failed connect, so an error ends the wait too. Returns false when the time runs out.
    /// </summary>
    public async Task<bool> DisconnectAndWaitAsync(CancellationToken cancellationToken)
    {
        await cli.DisconnectAsync(cancellationToken);
        var waited = TimeSpan.Zero;
        while (waited < timings.DisconnectTimeout)
        {
            if ((await ReadStatusAsync(cancellationToken)).IsDown)
            {
                return true;
            }
            await waiter.WaitAsync(timings.StatusPollInterval, cancellationToken);
            waited += timings.StatusPollInterval;
        }
        return false;
    }

    /// <summary>
    /// Tears down any tunnel first, since the app drops a connect issued while one is up (issue #2), then starts
    /// a non-blocking connect and polls, so a dead pair costs the connect timeout instead of the CLI's own 42 seconds.
    /// Ends with the settle delay, because the CLI can report Connected and drop seconds later.
    /// </summary>
    public async Task ConnectAsync(string nickname, string protocol, CancellationToken cancellationToken)
    {
        if (!(await ReadStatusAsync(cancellationToken)).IsDown)
        {
            await DisconnectAndWaitAsync(cancellationToken);
        }
        await cli.ConnectAsync(nickname, protocol, cancellationToken);
        var waited = TimeSpan.Zero;
        while (waited < timings.ConnectTimeout)
        {
            await waiter.WaitAsync(timings.StatusPollInterval, cancellationToken);
            waited += timings.StatusPollInterval;
            if ((await ReadStatusAsync(cancellationToken)).IsConnected)
            {
                break;
            }
        }
        await waiter.WaitAsync(timings.SettleDelay, cancellationToken);
    }

    /// <summary>
    /// True only when the tunnel is up and the blocked site answers through it.
    /// </summary>
    public async Task<bool> WorksAsync(CancellationToken cancellationToken)
    {
        if (!(await ReadStatusAsync(cancellationToken)).IsConnected)
        {
            return false;
        }
        return await blockedSiteProbe.LoadsAsync(cancellationToken);
    }
}
