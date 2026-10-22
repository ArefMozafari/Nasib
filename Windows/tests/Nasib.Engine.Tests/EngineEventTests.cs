namespace Nasib.Engine.Tests;

public class EngineEventTests
{
    public class Text
    {
        public static TheoryData<EngineEvent, string> Lines => new()
        {
            { new TryingEvent("Stolen Cheese", "ikev2"), "trying: Stolen Cheese / ikev2" },
            { new ConnectedEvent("Magic City", "wireguard"), "connected: Magic City / wireguard" },
            { new FailedEvent("Lygos", "tcp"), "failed: Lygos / tcp" },
            { new SearchStoppedEvent(), "search stopped" },
            { new NothingWorkedEvent(), "nothing worked" },
            { new TunnelDownEvent(), "tunnel down" },
            { new SurveyFinishedEvent(), "survey finished" },
            { new SurveyStoppedEvent(), "survey stopped" },
            { new MessageEvent("protocol order (from Settings): ikev2, tcp"), "protocol order (from Settings): ikev2, tcp" },
        };

        [Theory(DisplayName = "should print the same line as the macOS engine")]
        [MemberData(nameof(Lines))]
        public void ShouldPrintTheSameLineAsTheMacOsEngine(EngineEvent engineEvent, string expected)
        {
            Assert.Equal(expected, engineEvent.Text);
        }
    }
}
