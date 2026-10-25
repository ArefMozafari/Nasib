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

    public class IsStriped
    {
        [Fact(DisplayName = "should stripe every other line")]
        public void ShouldStripeEveryOtherLine()
        {
            Assert.False(EngineController.IsStriped(0));
            Assert.True(EngineController.IsStriped(1));
        }

        // Lines are numbered from the first one ever shown, so dropping the oldest doesn't flip every stripe.
        [Fact(DisplayName = "should keep each line's stripe when older lines scroll out")]
        public void ShouldKeepEachLinesStripeWhenOlderLinesScrollOut()
        {
            Assert.Equal(EngineController.IsStriped(201), EngineController.IsStriped(201));
            Assert.NotEqual(EngineController.IsStriped(200), EngineController.IsStriped(201));
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
