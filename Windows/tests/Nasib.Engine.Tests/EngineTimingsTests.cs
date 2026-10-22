namespace Nasib.Engine.Tests;

public class EngineTimingsTests
{
    public class Default
    {
        [Fact(DisplayName = "should match the waits in the macOS engine")]
        public void ShouldMatchTheWaitsInTheMacOsEngine()
        {
            var timings = EngineTimings.Default;

            Assert.Equal(TimeSpan.FromSeconds(8), timings.ProbeTimeout);
            Assert.Equal(TimeSpan.FromSeconds(10), timings.ConnectTimeout);
            Assert.Equal(TimeSpan.FromSeconds(2), timings.SettleDelay);
            Assert.Equal(TimeSpan.FromSeconds(15), timings.DisconnectTimeout);
            Assert.Equal(TimeSpan.FromSeconds(1), timings.StatusPollInterval);
            Assert.Equal(TimeSpan.FromSeconds(2), timings.ReachTimeout);
        }
    }
}
