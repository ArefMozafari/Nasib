using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class ClientLogParserTests
{
    private const string LocationLine =
        """{"tm": "2026-10-22 14:40:24.882", "lvl": "debug", "mod": "basic", "msg": "MutableLocationInfo created:  \"Amsterdam - Stolen Cheese\" \"203.0.113.1; 203.0.113.2; \" ; Selected node: 1"}""";

    public class ParseLine
    {
        [Fact(DisplayName = "should read the city, nickname and addresses from a JSON line")]
        public void ShouldReadTheCityNicknameAndAddressesFromAJsonLine()
        {
            var datacenter = ClientLogParser.ParseLine(LocationLine);

            Assert.NotNull(datacenter);
            Assert.Equal("Amsterdam", datacenter.City);
            Assert.Equal("Stolen Cheese", datacenter.Nickname);
            Assert.Equal(["203.0.113.1", "203.0.113.2"], datacenter.Addresses);
        }

        [Fact(DisplayName = "should read a plain-text line without the JSON wrapper")]
        public void ShouldReadAPlainTextLineWithoutTheJsonWrapper()
        {
            var datacenter = ClientLogParser.ParseLine("""MutableLocationInfo created:  "New York - Inside Job" "203.0.113.9; " """);

            Assert.NotNull(datacenter);
            Assert.Equal("New York", datacenter.City);
            Assert.Equal("Inside Job", datacenter.Nickname);
            Assert.Equal(["203.0.113.9"], datacenter.Addresses);
        }

        [Theory(DisplayName = "should ignore lines that are not location lines")]
        [InlineData("""{"tm": "2026-10-22 14:40:24.882", "lvl": "debug", "mod": "user", "msg": "Location selected: \"131Amsterdam - Stolen Cheese\""}""")]
        [InlineData("""{"tm": "2026-10-22 14:40:24.882", "lvl": "info", "mod": "basic"}""")]
        [InlineData("""{"tm": "2026-10-22""")]
        [InlineData("")]
        public void ShouldIgnoreLinesThatAreNotLocationLines(string line)
        {
            Assert.Null(ClientLogParser.ParseLine(line));
        }
    }

    public class ReadDirectory
    {
        [Fact(DisplayName = "should merge every address seen per nickname across both files")]
        public void ShouldMergeEveryAddressSeenPerNicknameAcrossBothFiles()
        {
            using var directory = new TemporaryDirectory();
            File.WriteAllText(directory.Combine("client.1.log"), LocationLine + "\n");
            File.WriteAllText(
                directory.Combine("client.log"),
                LocationLine.Replace("203.0.113.2", "203.0.113.3") + "\n{\"msg\": \"something else\"}\n");

            var datacenters = ClientLogParser.ReadDirectory(directory.Path);

            var datacenter = Assert.Single(datacenters);
            Assert.Equal(["203.0.113.1", "203.0.113.2", "203.0.113.3"], datacenter.Addresses);
        }

        [Fact(DisplayName = "should return nothing when the folder has no logs")]
        public void ShouldReturnNothingWhenTheFolderHasNoLogs()
        {
            using var directory = new TemporaryDirectory();

            Assert.Empty(ClientLogParser.ReadDirectory(directory.Path));
        }

        [Fact(DisplayName = "should read a log that another process holds open for writing")]
        public void ShouldReadALogThatAnotherProcessHoldsOpenForWriting()
        {
            using var directory = new TemporaryDirectory();
            var path = directory.Combine("client.log");
            File.WriteAllText(path, LocationLine + "\n");
            using var writer = new FileStream(path, FileMode.Append, FileAccess.Write, FileShare.ReadWrite);

            Assert.Single(ClientLogParser.ReadDirectory(directory.Path));
        }
    }
}
