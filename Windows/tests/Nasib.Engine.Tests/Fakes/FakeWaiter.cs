namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// Returns at once and adds up the time the engine asked to wait, so tests can check waits without taking them.
/// </summary>
internal sealed class FakeWaiter : IWaiter
{
    public TimeSpan TotalWaited { get; private set; }

    public Task WaitAsync(TimeSpan duration, CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        TotalWaited += duration;
        return Task.CompletedTask;
    }
}
