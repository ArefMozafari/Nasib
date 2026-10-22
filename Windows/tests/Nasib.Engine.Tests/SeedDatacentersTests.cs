namespace Nasib.Engine.Tests;

public class SeedDatacentersTests
{
    public class Load
    {
        [Fact(DisplayName = "should load every bundled datacenter with its addresses")]
        public void ShouldLoadEveryBundledDatacenterWithItsAddresses()
        {
            var datacenters = SeedDatacenters.Load();

            Assert.Equal(20, datacenters.Count);
            Assert.All(datacenters, datacenter => Assert.NotEmpty(datacenter.Addresses));
            Assert.Equal(datacenters.Count, datacenters.Select(datacenter => datacenter.Nickname).Distinct().Count());
        }

        [Fact(DisplayName = "should cover every default city")]
        public void ShouldCoverEveryDefaultCity()
        {
            var cities = SeedDatacenters.Load().Select(datacenter => datacenter.City).ToHashSet();

            Assert.All(EngineConfiguration.DefaultCities, city => Assert.Contains(city, cities));
        }

        [Fact(DisplayName = "should split nicknames that contain spaces correctly")]
        public void ShouldSplitNicknamesThatContainSpacesCorrectly()
        {
            var datacenter = SeedDatacenters.Load().Single(datacenter => datacenter.Nickname == "Stolen Cheese");

            Assert.Equal("Amsterdam", datacenter.City);
            Assert.Equal(["161.129.60.66", "161.129.60.34"], datacenter.Addresses);
        }
    }
}
