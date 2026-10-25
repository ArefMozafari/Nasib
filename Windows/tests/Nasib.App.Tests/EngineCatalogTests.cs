namespace Nasib.App.Tests;

public class EngineCatalogTests
{
    public class Decode
    {
        [Fact(DisplayName = "should decode the catalog line")]
        public void ShouldDecodeTheCatalogLine()
        {
            const string line = """{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"The 6"}]},{"name":"Other","datacenters":[{"city":"Oslo","nickname":"Fjord"}]}]}""";

            var catalog = EngineCatalog.Decode(line);

            Assert.NotNull(catalog);
            Assert.Equal(["Canada East", "Other"], catalog.Regions.Select(region => region.Name));
            Assert.Equal(new CatalogEntry("Toronto", "The 6"), catalog.Regions[0].Datacenters[0]);
        }

        [Fact(DisplayName = "should read which datacenters the log knows")]
        public void ShouldReadWhichDatacentersTheLogKnows()
        {
            const string line = """{"regions":[{"name":"Canada East","datacenters":[{"city":"Toronto","nickname":"Comfort Zone","logged":true},{"city":"Toronto","nickname":"The 6"}]}]}""";

            Assert.Equal([true, false], EngineCatalog.Decode(line)!.Entries.Select(entry => entry.Logged));
        }

        [Theory(DisplayName = "should return nothing for anything else")]
        [InlineData(@"windscribe-cli missing: C:\Program Files\Windscribe\windscribe-cli.exe")]
        [InlineData("")]
        [InlineData("""{"regions":null}""")]
        public void ShouldReturnNothingForAnythingElse(string line)
        {
            Assert.Null(EngineCatalog.Decode(line));
        }
    }

    public class Entries
    {
        [Fact(DisplayName = "should list every entry in order")]
        public void ShouldListEveryEntryInOrder()
        {
            var catalog = new EngineCatalog(
            [
                new CatalogRegion("A", [new CatalogEntry("Toronto", "The 6")]),
                new CatalogRegion("B", [new CatalogEntry("Dubai", "Khalifa")]),
            ]);

            Assert.Equal(["The 6", "Khalifa"], catalog.Entries.Select(entry => entry.Nickname));
        }
    }

    public class Title
    {
        [Fact(DisplayName = "should show the city and nickname as Windscribe does")]
        public void ShouldShowTheCityAndNicknameAsWindscribeDoes()
        {
            Assert.Equal("Toronto - The 6", new CatalogEntry("Toronto", "The 6").Title);
            Assert.Equal("The 6", new CatalogEntry("", "The 6").Title);
        }
    }
}
