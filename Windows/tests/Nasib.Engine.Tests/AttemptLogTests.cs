using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class AttemptLogTests
{
    private static readonly DateTimeOffset FixedNow = new(2026, 10, 22, 14, 40, 24, TimeSpan.Zero);

    public class Record
    {
        [Fact(DisplayName = "should append lines in the macOS log format")]
        public void ShouldAppendLinesInTheMacOsLogFormat()
        {
            using var directory = new TemporaryDirectory();
            var paths = new StatePaths(directory.Combine("Nasib"));
            var log = new AttemptLog(paths, new FixedTimeProvider(FixedNow));

            log.Record("Stolen Cheese", "ikev2", succeeded: true);
            log.Record("Khalifa", "tcp", succeeded: false);

            Assert.Equal(
                ["2026-10-22T14:40:24,Stolen Cheese,ikev2,ok", "2026-10-22T14:40:24,Khalifa,tcp,fail"],
                File.ReadAllLines(paths.LogFile));
        }
    }

    public class ReadRecent
    {
        [Fact(DisplayName = "should return only the last lines and skip malformed ones")]
        public void ShouldReturnOnlyTheLastLinesAndSkipMalformedOnes()
        {
            using var directory = new TemporaryDirectory();
            var paths = new StatePaths(directory.Path);
            File.WriteAllLines(paths.LogFile,
            [
                "2026-10-22T14:00:00,Lygos,tcp,ok",
                "2026-10-22T14:01:00,Khalifa,ikev2,fail",
                "not a log line",
                "2026-10-22T14:02:00,Khalifa,ikev2,ok",
            ]);
            var log = new AttemptLog(paths, TimeProvider.System);

            var recent = log.ReadRecent(3);

            Assert.Equal(
                [
                    new AttemptRecord("2026-10-22T14:01:00", "Khalifa", "ikev2", false),
                    new AttemptRecord("2026-10-22T14:02:00", "Khalifa", "ikev2", true),
                ],
                recent);
        }

        [Fact(DisplayName = "should return nothing before the first attempt")]
        public void ShouldReturnNothingBeforeTheFirstAttempt()
        {
            using var directory = new TemporaryDirectory();

            Assert.Empty(new AttemptLog(new StatePaths(directory.Path), TimeProvider.System).ReadRecent(SuccessRanker.RankWindowLines));
        }
    }

    public class LastGood
    {
        [Fact(DisplayName = "should round-trip the last good pair in the macOS format")]
        public void ShouldRoundTripTheLastGoodPairInTheMacOsFormat()
        {
            using var directory = new TemporaryDirectory();
            var paths = new StatePaths(directory.Path);
            var log = new AttemptLog(paths, TimeProvider.System);

            log.WriteLastGood(new ServerPair("Stolen Cheese", "wireguard"));

            Assert.Equal("Stolen Cheese|wireguard\n", File.ReadAllText(paths.LastGoodFile));
            Assert.Equal(new ServerPair("Stolen Cheese", "wireguard"), log.ReadLastGood());
        }

        [Theory(DisplayName = "should read no pair from a missing or malformed file")]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("Stolen Cheese")]
        [InlineData("|ikev2")]
        public void ShouldReadNoPairFromAMissingOrMalformedFile(string? content)
        {
            using var directory = new TemporaryDirectory();
            var paths = new StatePaths(directory.Path);
            if (content is not null)
            {
                File.WriteAllText(paths.LastGoodFile, content);
            }

            Assert.Null(new AttemptLog(paths, TimeProvider.System).ReadLastGood());
        }
    }
}
