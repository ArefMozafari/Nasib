using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class DatacenterCatalogTests
{
    private static readonly IReadOnlyList<Datacenter> Seed =
    [
        new("Istanbul", "Lygos", ["203.0.113.1"]),
        new("Amsterdam", "Stolen Cheese", ["203.0.113.2"]),
    ];

    public class Merge
    {
        [Fact(DisplayName = "should add logged addresses to a known nickname")]
        public void ShouldAddLoggedAddressesToAKnownNickname()
        {
            var logged = new List<Datacenter> { new("Amsterdam", "Stolen Cheese", ["203.0.113.9"]) };

            var merged = DatacenterCatalog.Merge(Seed, logged, new EngineConfiguration([], []));

            Assert.Equal(["Lygos", "Stolen Cheese"], merged.Select(datacenter => datacenter.Nickname));
            Assert.Equal(["203.0.113.2", "203.0.113.9"], merged[1].Addresses);
        }

        [Fact(DisplayName = "should append nicknames only the log knows")]
        public void ShouldAppendNicknamesOnlyTheLogKnows()
        {
            var logged = new List<Datacenter> { new("Oslo", "Fjord", ["203.0.113.7"]) };

            var merged = DatacenterCatalog.Merge(Seed, logged, new EngineConfiguration([], []));

            Assert.Equal(["Lygos", "Stolen Cheese", "Fjord"], merged.Select(datacenter => datacenter.Nickname));
        }

        [Fact(DisplayName = "should keep only the chosen cities")]
        public void ShouldKeepOnlyTheChosenCities()
        {
            var merged = DatacenterCatalog.Merge(Seed, [], new EngineConfiguration(["amsterdam"], []));

            Assert.Equal(["Stolen Cheese"], merged.Select(datacenter => datacenter.Nickname));
        }
    }

    public class Load
    {
        [Fact(DisplayName = "should merge the Windscribe log over the seed")]
        public void ShouldMergeTheWindscribeLogOverTheSeed()
        {
            using var directory = new TemporaryDirectory();
            File.WriteAllText(
                directory.Combine("client.log"),
                """{"msg": "MutableLocationInfo created:  \"Amsterdam - Stolen Cheese\" \"203.0.113.5; \" ; Selected node: 1"}""" + "\n");
            var catalog = new DatacenterCatalog(Seed, directory.Path);

            var datacenters = catalog.Load(EngineConfiguration.Default);

            Assert.Equal(["203.0.113.2", "203.0.113.5"], datacenters.Single(datacenter => datacenter.Nickname == "Stolen Cheese").Addresses);
        }
    }
}
