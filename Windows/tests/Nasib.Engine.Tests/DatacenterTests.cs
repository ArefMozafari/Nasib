namespace Nasib.Engine.Tests;

public class DatacenterTests
{
    public class FromLocationName
    {
        [Fact(DisplayName = "should split the city from the nickname at the first separator")]
        public void ShouldSplitTheCityFromTheNicknameAtTheFirstSeparator()
        {
            var datacenter = Datacenter.FromLocationName("New York - Inside - Job", ["203.0.113.1"]);

            Assert.NotNull(datacenter);
            Assert.Equal("New York", datacenter.City);
            Assert.Equal("Inside - Job", datacenter.Nickname);
        }

        [Theory(DisplayName = "should reject names without a city and a nickname")]
        [InlineData("Amsterdam")]
        [InlineData(" - Stolen Cheese")]
        [InlineData("Amsterdam - ")]
        public void ShouldRejectNamesWithoutACityAndANickname(string locationName)
        {
            Assert.Null(Datacenter.FromLocationName(locationName, []));
        }
    }

    public class LocationName
    {
        [Fact(DisplayName = "should join the city and nickname as Windscribe shows them")]
        public void ShouldJoinTheCityAndNicknameAsWindscribeShowsThem()
        {
            Assert.Equal("Amsterdam - Stolen Cheese", new Datacenter("Amsterdam", "Stolen Cheese", []).LocationName);
        }
    }

    public class WithAddresses
    {
        [Fact(DisplayName = "should add new addresses after the known ones without duplicates")]
        public void ShouldAddNewAddressesAfterTheKnownOnesWithoutDuplicates()
        {
            var datacenter = new Datacenter("Dubai", "Khalifa", ["203.0.113.1", "203.0.113.2"]);

            var merged = datacenter.WithAddresses(["203.0.113.2", "203.0.113.3"]);

            Assert.Equal(["203.0.113.1", "203.0.113.2", "203.0.113.3"], merged.Addresses);
        }
    }
}
