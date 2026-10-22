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

    /// <summary>
    /// Called when a probe starts; a probe then never answers until it is cancelled, like a blackholed request.
    /// </summary>
    public Action? HangsAfter { get; set; }

    public async Task<bool> LoadsAsync(CancellationToken cancellationToken)
    {
        ProbeCalls++;
        if (HangsAfter is not null)
        {
            HangsAfter();
            await Task.Delay(Timeout.Infinite, cancellationToken);
        }
        return cli.State == FakeTunnelState.Connected && LoadsThrough(cli.ConnectedLocation);
    }
}
