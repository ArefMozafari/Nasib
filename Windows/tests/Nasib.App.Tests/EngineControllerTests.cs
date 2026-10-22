namespace Nasib.App.Tests;

public class EngineControllerTests
{
    public class StatusInterval
    {
        [Fact(DisplayName = "should read the tunnel status every three seconds")]
        public void ShouldReadTheTunnelStatusEveryThreeSeconds()
        {
            Assert.Equal(TimeSpan.FromSeconds(3), EngineController.StatusInterval);
        }
    }

    public class WatchInterval
    {
        [Fact(DisplayName = "should keep the blocked-site check at once a minute")]
        public void ShouldKeepTheBlockedSiteCheckAtOnceAMinute()
        {
            Assert.Equal(TimeSpan.FromSeconds(60), EngineController.WatchInterval);
        }
    }
}
