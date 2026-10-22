namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// Answers for the listed addresses only, or for every address when no list is given.
/// </summary>
internal sealed class FakeReachabilityProbe : IReachabilityProbe
{
    private readonly IReadOnlySet<string>? answeringAddresses;

    public FakeReachabilityProbe(IReadOnlySet<string>? answeringAddresses = null)
    {
        this.answeringAddresses = answeringAddresses;
    }

    public List<(string Address, int Port)> Knocks { get; } = [];

    public Task<bool> AnswersAsync(string address, int port, TimeSpan timeout, CancellationToken cancellationToken)
    {
        lock (Knocks)
        {
            Knocks.Add((address, port));
        }
        return Task.FromResult(answeringAddresses is null || answeringAddresses.Contains(address));
    }
}
