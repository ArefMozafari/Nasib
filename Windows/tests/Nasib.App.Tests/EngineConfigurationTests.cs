using System.IO;

namespace Nasib.App.Tests;

public class EngineConfigurationTests
{
    /// <summary>
    /// A throwaway folder under the system temp folder, deleted when the test ends.
    /// </summary>
    private sealed class TemporaryDirectory : IDisposable
    {
        public string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "nasib-tests-" + Guid.NewGuid().ToString("N"));

        public string ConfigurationFile => System.IO.Path.Combine(Path, "config.json");

        public void Dispose()
        {
            if (Directory.Exists(Path))
            {
                Directory.Delete(Path, recursive: true);
            }
        }
    }

    public class Load
    {
        [Fact(DisplayName = "should return the defaults when nothing is saved")]
        public void ShouldReturnTheDefaultsWhenNothingIsSaved()
        {
            using var directory = new TemporaryDirectory();

            var loaded = EngineConfiguration.Load(directory.ConfigurationFile);

            Assert.Equal(EngineConfiguration.DefaultCities, loaded.Cities);
            Assert.Equal(EngineConfiguration.AllProtocols, loaded.ProtocolOrder);
            Assert.Null(loaded.Locations);
            Assert.True(loaded.RankBySuccess);
        }

        [Fact(DisplayName = "should drop protocols the engine does not know")]
        public void ShouldDropProtocolsTheEngineDoesNotKnow()
        {
            using var directory = new TemporaryDirectory();
            Directory.CreateDirectory(directory.Path);
            File.WriteAllText(directory.ConfigurationFile, """{ "cities": ["Dubai"], "protocolOrder": ["tcp", "bogus", "wireguard", "tcp"] }""");

            Assert.Equal(["tcp", "wireguard"], EngineConfiguration.Load(directory.ConfigurationFile).ProtocolOrder);
        }

        [Fact(DisplayName = "should default ranking on and keep locations unset when a file predates them")]
        public void ShouldDefaultRankingOnAndKeepLocationsUnsetWhenAFilePredatesThem()
        {
            using var directory = new TemporaryDirectory();
            Directory.CreateDirectory(directory.Path);
            File.WriteAllText(directory.ConfigurationFile, """{ "cities": ["Dubai"], "protocolOrder": ["tcp"] }""");

            var loaded = EngineConfiguration.Load(directory.ConfigurationFile);

            Assert.True(loaded.RankBySuccess);
            Assert.Null(loaded.Locations);
            Assert.Equal(["Dubai"], loaded.Cities);
        }

        [Theory(DisplayName = "should return the defaults for a file it cannot use")]
        [InlineData("CITIES=(Dubai)")]
        [InlineData("""{ "cities": ["Dubai"] }""")]
        public void ShouldReturnTheDefaultsForAFileItCannotUse(string content)
        {
            using var directory = new TemporaryDirectory();
            Directory.CreateDirectory(directory.Path);
            File.WriteAllText(directory.ConfigurationFile, content);

            Assert.Equal(EngineConfiguration.AllProtocols, EngineConfiguration.Load(directory.ConfigurationFile).ProtocolOrder);
        }
    }

    public class Save
    {
        [Fact(DisplayName = "should round-trip locations, ranking, protocols and cities")]
        public void ShouldRoundTripLocationsRankingProtocolsAndCities()
        {
            using var directory = new TemporaryDirectory();
            var configuration = new EngineConfiguration(["Saint John's", "New York"], ["ikev2", "tcp"], ["The 6", "Khalifa"], RankBySuccess: false);

            configuration.Save(directory.ConfigurationFile);
            var loaded = EngineConfiguration.Load(directory.ConfigurationFile);

            Assert.Equal(configuration.Cities, loaded.Cities);
            Assert.Equal(configuration.ProtocolOrder, loaded.ProtocolOrder);
            Assert.Equal(configuration.Locations, loaded.Locations);
            Assert.False(loaded.RankBySuccess);
        }

        [Fact(DisplayName = "should write the keys the engine reads")]
        public void ShouldWriteTheKeysTheEngineReads()
        {
            using var directory = new TemporaryDirectory();

            (EngineConfiguration.Default with { Locations = ["The 6"] }).Save(directory.ConfigurationFile);
            var text = File.ReadAllText(directory.ConfigurationFile);

            Assert.Contains("\"locations\"", text);
            Assert.Contains("\"rankBySuccess\"", text);
            Assert.Contains("\"protocolOrder\"", text);
            Assert.Contains("\"cities\"", text);
        }

        // The engine carries the cities over only when locations is missing, so unset locations aren't written.
        [Fact(DisplayName = "should leave locations out when they are unset")]
        public void ShouldLeaveLocationsOutWhenTheyAreUnset()
        {
            using var directory = new TemporaryDirectory();

            EngineConfiguration.Default.Save(directory.ConfigurationFile);

            Assert.DoesNotContain("\"locations\"", File.ReadAllText(directory.ConfigurationFile));
        }
    }

    public class Defaults
    {
        [Fact(DisplayName = "should carry over the default cities and know the twenty default locations")]
        public void ShouldCarryOverTheDefaultCitiesAndKnowTheTwentyDefaultLocations()
        {
            Assert.Null(EngineConfiguration.Default.Locations);
            Assert.Equal(EngineConfiguration.DefaultCities, EngineConfiguration.Default.Cities);
            Assert.Equal(20, EngineConfiguration.DefaultLocations.Count);
            Assert.Equal("Lygos", EngineConfiguration.DefaultLocations[0]);
            Assert.Equal(EngineConfiguration.AllProtocols, EngineConfiguration.Default.ProtocolOrder);
        }
    }
}
