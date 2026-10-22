namespace Nasib.Engine;

/// <summary>
/// Drops datacenters where no address answers at all. A blackholed address cannot work on any protocol,
/// so skipping it loses nothing, and knocking on every address at once takes about two seconds.
/// </summary>
public sealed class ReachabilityFilter
{
    /// <summary>
    /// Every Windscribe datacenter listens on HTTPS, whichever protocol the tunnel uses.
    /// </summary>
    public const int ReachPort = 443;

    private readonly IReachabilityProbe probe;
    private readonly EngineTimings timings;

    public ReachabilityFilter(IReachabilityProbe probe, EngineTimings timings)
    {
        this.probe = probe;
        this.timings = timings;
    }

    /// <summary>
    /// Returns the datacenters with at least one answering address, in the order given.
    /// </summary>
    public async Task<IReadOnlyList<Datacenter>> ReachableAsync(
        IReadOnlyList<Datacenter> datacenters,
        CancellationToken cancellationToken)
    {
        var knocks = datacenters
            .Select(datacenter => datacenter.Addresses
                .Select(address => probe.AnswersAsync(address, ReachPort, timings.ReachTimeout, cancellationToken))
                .ToList())
            .ToList();
        await Task.WhenAll(knocks.SelectMany(addressKnocks => addressKnocks));
        return datacenters
            .Where((_, index) => knocks[index].Any(knock => knock.Result))
            .ToList();
    }
}
