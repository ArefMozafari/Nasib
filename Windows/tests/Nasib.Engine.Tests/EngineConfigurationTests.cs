using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class EngineConfigurationTests
{
    public class Load
    {
        [Fact(DisplayName = "should return the defaults when the file is missing")]
        public void ShouldReturnTheDefaultsWhenTheFileIsMissing()
        {
            using var directory = new TemporaryDirectory();

            Assert.Equal(EngineConfiguration.Default, EngineConfiguration.Load(directory.Combine("config.json")));
        }

        [Fact(DisplayName = "should return the defaults when the file is not valid JSON")]
        public void ShouldReturnTheDefaultsWhenTheFileIsNotValidJson()
        {
            using var directory = new TemporaryDirectory();
            var path = directory.Combine("config.json");
            File.WriteAllText(path, "CITIES=(Dubai)");

            Assert.Equal(EngineConfiguration.Default, EngineConfiguration.Load(path));
        }

        [Fact(DisplayName = "should drop protocols the engine does not know")]
        public void ShouldDropProtocolsTheEngineDoesNotKnow()
        {
            using var directory = new TemporaryDirectory();
            var path = directory.Combine("config.json");
            File.WriteAllText(path, """{ "cities": ["Dubai"], "protocolOrder": ["tcp", "openvpn", "ikev2"] }""");

            var configuration = EngineConfiguration.Load(path);

            Assert.Equal(["Dubai"], configuration.Cities);
            Assert.Equal(["tcp", "ikev2"], configuration.ProtocolOrder);
        }
    }

    public class Save
    {
        [Fact(DisplayName = "should round-trip cities with quotes and an empty protocol list")]
        public void ShouldRoundTripCitiesWithQuotesAndAnEmptyProtocolList()
        {
            using var directory = new TemporaryDirectory();
            var path = Path.Combine(directory.Path, "Nasib", "config.json");
            var configuration = new EngineConfiguration(["New York", "L'Aquila"], []);

            configuration.Save(path);
            var loaded = EngineConfiguration.Load(path);

            Assert.Equal(configuration.Cities, loaded.Cities);
            Assert.Empty(loaded.ProtocolOrder);
        }

        [Fact(DisplayName = "should write the camel-case keys the brief names")]
        public void ShouldWriteTheCamelCaseKeysTheBriefNames()
        {
            using var directory = new TemporaryDirectory();
            var path = directory.Combine("config.json");

            EngineConfiguration.Default.Save(path);
            var text = File.ReadAllText(path);

            Assert.Contains("\"cities\"", text);
            Assert.Contains("\"protocolOrder\"", text);
        }
    }

    public class IncludesCity
    {
        [Fact(DisplayName = "should include every city when the list is empty")]
        public void ShouldIncludeEveryCityWhenTheListIsEmpty()
        {
            Assert.True(new EngineConfiguration([], []).IncludesCity("Reykjavik"));
        }

        [Fact(DisplayName = "should match cities without regard to case")]
        public void ShouldMatchCitiesWithoutRegardToCase()
        {
            var configuration = new EngineConfiguration(["New York"], []);

            Assert.True(configuration.IncludesCity("new york"));
            Assert.False(configuration.IncludesCity("York"));
        }
    }
}
