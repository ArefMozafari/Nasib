namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// A whole engine wired to fakes in a throwaway folder: a Windscribe log with one datacenter, no bundled list,
/// and a fake CLI, probe and clock. The Windows counterpart of the sandbox in macOS/Tests/Engine/ws-find.test.zsh.
/// </summary>
internal sealed class EngineHarness : IDisposable
{
    public const string City = "Dubai";
    public const string Nickname = "Khalifa";
    public const string Protocol = "ikev2";
    public const string Address = "135.136.7.178";

    private static readonly DateTimeOffset FixedNow = new(2026, 10, 22, 14, 40, 24, TimeSpan.Zero);

    private readonly TemporaryDirectory directory = new();

    public EngineHarness(IReadOnlyList<Datacenter>? seed = null, FakeReachabilityProbe? reachabilityProbe = null)
    {
        var logDirectory = directory.Combine("Windscribe2");
        Directory.CreateDirectory(logDirectory);
        File.WriteAllText(
            Path.Combine(logDirectory, "client.log"),
            $$"""{"tm": "2026-10-22 14:40:24.882", "lvl": "debug", "mod": "basic", "msg": "MutableLocationInfo created:  \"{{City}} - {{Nickname}}\" \"{{Address}}; \" ; Selected node: 1"}""" + "\n");

        Paths = new StatePaths(directory.Combine("Nasib"));
        Paths.EnsureExists();
        Cli = new FakeWindscribeCli();
        BlockedSiteProbe = new FakeBlockedSiteProbe(Cli);
        ReachabilityProbe = reachabilityProbe ?? new FakeReachabilityProbe();
        Waiter = new FakeWaiter();
        AttemptLog = new AttemptLog(Paths, new FixedTimeProvider(FixedNow));
        Tunnel = new TunnelController(Cli, BlockedSiteProbe, Waiter, EngineTimings.Default);
        Finder = new ServerFinder(
            Paths,
            new DatacenterCatalog(seed ?? [], logDirectory),
            Tunnel,
            new ReachabilityFilter(ReachabilityProbe, EngineTimings.Default),
            AttemptLog);
        Finder.EventRaised += Events.Add;
    }

    public StatePaths Paths { get; }

    public FakeWindscribeCli Cli { get; }

    public FakeBlockedSiteProbe BlockedSiteProbe { get; }

    public FakeReachabilityProbe ReachabilityProbe { get; }

    public FakeWaiter Waiter { get; }

    public AttemptLog AttemptLog { get; }

    public TunnelController Tunnel { get; }

    public ServerFinder Finder { get; }

    public List<EngineEvent> Events { get; } = [];

    public void WriteLastGood(string nickname, string protocol)
    {
        AttemptLog.WriteLastGood(new ServerPair(nickname, protocol));
    }

    public void SaveConfiguration(EngineConfiguration configuration)
    {
        configuration.Save(Paths.ConfigurationFile);
    }

    /// <summary>
    /// The log lines without their timestamps, as the zsh test compares them.
    /// </summary>
    public IReadOnlyList<string> LoggedAttempts()
    {
        return File.Exists(Paths.LogFile)
            ? File.ReadAllLines(Paths.LogFile).Select(line => line[(line.IndexOf(',') + 1)..]).ToList()
            : [];
    }

    public void Dispose()
    {
        directory.Dispose();
    }
}
