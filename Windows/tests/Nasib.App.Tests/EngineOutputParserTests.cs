namespace Nasib.App.Tests;

public class EngineOutputParserTests
{
    public class Parse
    {
        [Fact(DisplayName = "should parse a trying line")]
        public void ShouldParseATryingLine()
        {
            Assert.Equal(new TryingEvent("Stolen Cheese", "ikev2"), EngineOutputParser.Parse("trying: Stolen Cheese / ikev2"));
        }

        [Fact(DisplayName = "should parse a connected line")]
        public void ShouldParseAConnectedLine()
        {
            Assert.Equal(new ConnectedEvent("Magic City", "ikev2"), EngineOutputParser.Parse("connected: Magic City / ikev2"));
        }

        [Fact(DisplayName = "should parse a failed line")]
        public void ShouldParseAFailedLine()
        {
            Assert.Equal(new FailedEvent("Sausage Party", "wireguard"), EngineOutputParser.Parse("failed: Sausage Party / wireguard"));
        }

        [Theory(DisplayName = "should parse the fixed lines")]
        [InlineData("nothing worked", typeof(NothingWorkedEvent))]
        [InlineData("survey finished", typeof(SurveyFinishedEvent))]
        [InlineData("survey stopped", typeof(SurveyStoppedEvent))]
        [InlineData("search stopped", typeof(SearchStoppedEvent))]
        [InlineData("tunnel down", typeof(TunnelDownEvent))]
        public void ShouldParseTheFixedLines(string line, Type expected)
        {
            Assert.IsType(expected, EngineOutputParser.Parse(line));
        }

        [Fact(DisplayName = "should treat an unknown line as other")]
        public void ShouldTreatAnUnknownLineAsOther()
        {
            Assert.Equal(
                new OtherEvent("protocol order (from Settings): ikev2, wireguard"),
                EngineOutputParser.Parse("protocol order (from Settings): ikev2, wireguard"));
        }

        [Fact(DisplayName = "should split the pair on the last separator")]
        public void ShouldSplitThePairOnTheLastSeparator()
        {
            Assert.Equal(new TryingEvent("Inside / Job", "ikev2"), EngineOutputParser.Parse("trying: Inside / Job / ikev2"));
        }

        [Fact(DisplayName = "should parse a missing CLI")]
        public void ShouldParseAMissingCli()
        {
            var parsed = EngineOutputParser.Parse(@"windscribe-cli missing: C:\Program Files\Windscribe\windscribe-cli.exe");

            Assert.Equal(new CliMissingEvent(@"C:\Program Files\Windscribe\windscribe-cli.exe"), parsed);
        }

        [Fact(DisplayName = "should parse a phase line")]
        public void ShouldParseAPhaseLine()
        {
            Assert.Equal(new PhaseEvent("tcp on port 443"), EngineOutputParser.Parse("phase: tcp on port 443"));
        }

        [Fact(DisplayName = "should keep the port in a trying line")]
        public void ShouldKeepThePortInATryingLine()
        {
            Assert.Equal(new TryingEvent("The 6", "tcp:443"), EngineOutputParser.Parse("trying: The 6 / tcp:443"));
        }
    }

    public class Tone
    {
        public static TheoryData<EngineEvent, OutputTone> Tones => new()
        {
            { new TryingEvent("Lygos", "tcp"), OutputTone.Primary },
            { new ConnectedEvent("Lygos", "tcp"), OutputTone.Success },
            { new FailedEvent("Lygos", "tcp"), OutputTone.Muted },
            { new TunnelDownEvent(), OutputTone.Warning },
            { new NothingWorkedEvent(), OutputTone.Error },
            { new CliMissingEvent("windscribe-cli.exe"), OutputTone.Error },
            { new SearchStoppedEvent(), OutputTone.Accent },
            { new SurveyStoppedEvent(), OutputTone.Accent },
            { new SurveyFinishedEvent(), OutputTone.Accent },
            { new PhaseEvent("reachability"), OutputTone.Phase },
            { new OtherEvent("protocol order (from Settings): tcp"), OutputTone.Detail },
        };

        [Theory(DisplayName = "should give each event its tone")]
        [MemberData(nameof(Tones))]
        public void ShouldGiveEachEventItsTone(EngineEvent engineEvent, OutputTone expected)
        {
            Assert.Equal(expected, engineEvent.Tone);
        }
    }

    public class ParseStatus
    {
        [Fact(DisplayName = "should parse a connected status line")]
        public void ShouldParseAConnectedStatusLine()
        {
            Assert.Equal(new TunnelStatus("Atlanta - Magic City", "IKEv2"), EngineOutputParser.ParseStatus("connected: Atlanta - Magic City / IKEv2"));
        }

        [Theory(DisplayName = "should return nothing for anything but a connected tunnel")]
        [InlineData("down")]
        [InlineData(@"windscribe-cli missing: C:\Program Files\Windscribe\windscribe-cli.exe")]
        [InlineData("")]
        public void ShouldReturnNothingForAnythingButAConnectedTunnel(string line)
        {
            Assert.Null(EngineOutputParser.ParseStatus(line));
        }
    }
}
