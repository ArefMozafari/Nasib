namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// Loads the blocked site whenever the fake tunnel is connected, unless a test says otherwise.
/// </summary>
internal sealed class FakeBlockedSiteProbe : IBlockedSiteProbe
{
    private readonly FakeWindscribeCli cli;

    public FakeBlockedSiteProbe(FakeWindscribeCli cli)
    {
        this.cli = cli;
    }

    /// <summary>
    /// Decides whether the site loads through a connected tunnel to the given location.
    /// </summary>
    public Func<string, bool> LoadsThrough { get; set; } = _ => true;

    public int ProbeCalls { get; private set; }

    public Task<bool> LoadsAsync(CancellationToken cancellationToken)
    {
        ProbeCalls++;
        return Task.FromResult(cli.State == FakeTunnelState.Connected && LoadsThrough(cli.ConnectedLocation));
    }
}
