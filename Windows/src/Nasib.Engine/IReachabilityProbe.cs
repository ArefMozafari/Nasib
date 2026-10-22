namespace Nasib.Engine;

/// <summary>
/// Knocks on one address and port. Tests replace it with a fake.
/// </summary>
public interface IReachabilityProbe
{
    Task<bool> AnswersAsync(string address, int port, TimeSpan timeout, CancellationToken cancellationToken);
}
