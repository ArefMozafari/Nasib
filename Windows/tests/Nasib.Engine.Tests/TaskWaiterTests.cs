using System.Diagnostics;

namespace Nasib.Engine.Tests;

public class TaskWaiterTests
{
    public class WaitAsync
    {
        [Fact(DisplayName = "should wait at least the duration asked for")]
        public async Task ShouldWaitAtLeastTheDurationAskedFor()
        {
            var duration = TimeSpan.FromMilliseconds(50);
            var stopwatch = Stopwatch.StartNew();

            await new TaskWaiter().WaitAsync(duration, CancellationToken.None);

            Assert.True(stopwatch.Elapsed >= duration - TimeSpan.FromMilliseconds(15));
        }

        [Fact(DisplayName = "should stop waiting when cancelled")]
        public async Task ShouldStopWaitingWhenCancelled()
        {
            using var cancellation = new CancellationTokenSource();
            cancellation.Cancel();

            await Assert.ThrowsAnyAsync<OperationCanceledException>(
                () => new TaskWaiter().WaitAsync(TimeSpan.FromMinutes(1), cancellation.Token));
        }
    }
}
