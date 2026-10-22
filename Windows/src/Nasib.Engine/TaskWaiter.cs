namespace Nasib.Engine;

/// <summary>
/// Waits for real.
/// </summary>
public sealed class TaskWaiter : IWaiter
{
    public Task WaitAsync(TimeSpan duration, CancellationToken cancellationToken)
    {
        return Task.Delay(duration, cancellationToken);
    }
}
