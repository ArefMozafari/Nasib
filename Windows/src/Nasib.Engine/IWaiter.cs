namespace Nasib.Engine;

/// <summary>
/// Waits between status reads. Tests replace it with one that returns at once and adds up the time asked for.
/// </summary>
public interface IWaiter
{
    Task WaitAsync(TimeSpan duration, CancellationToken cancellationToken);
}
