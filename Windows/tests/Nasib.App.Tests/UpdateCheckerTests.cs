namespace Nasib.App.Tests;

public class UpdateCheckerTests
{
    private const string ReleasePage = "https://github.com/ArefMozafari/Nasib/releases/tag/v0.3.0";

    public class IsNewer
    {
        [Fact(DisplayName = "should treat a higher version as newer")]
        public void ShouldTreatAHigherVersionAsNewer()
        {
            Assert.True(UpdateChecker.IsNewer("0.3.0", "0.2.0"));
            Assert.True(UpdateChecker.IsNewer("1.0.0", "0.9.9"));
        }

        [Fact(DisplayName = "should compare components as numbers, not text")]
        public void ShouldCompareComponentsAsNumbersNotText()
        {
            Assert.True(UpdateChecker.IsNewer("0.10.0", "0.9.0"));
        }

        [Fact(DisplayName = "should not treat the same or an older version as newer")]
        public void ShouldNotTreatTheSameOrAnOlderVersionAsNewer()
        {
            Assert.False(UpdateChecker.IsNewer("0.2.0", "0.2.0"));
            Assert.False(UpdateChecker.IsNewer("0.1.1", "0.2.0"));
        }

        [Fact(DisplayName = "should treat missing components as zero")]
        public void ShouldTreatMissingComponentsAsZero()
        {
            Assert.False(UpdateChecker.IsNewer("0.2", "0.2.0"));
            Assert.True(UpdateChecker.IsNewer("0.2.1", "0.2"));
        }
    }

    public class ParseRelease
    {
        [Fact(DisplayName = "should read the version without its prefix and the page from the latest release")]
        public void ShouldReadTheVersionWithoutItsPrefixAndThePageFromTheLatestRelease()
        {
            var json = $$"""{"tag_name": "v0.3.0", "html_url": "{{ReleasePage}}", "name": "Nasib 0.3.0"}""";

            var release = UpdateChecker.ParseRelease(json);

            Assert.Equal(new UpdateChecker.Release("0.3.0", new Uri(ReleasePage)), release);
        }

        [Theory(DisplayName = "should return nothing for a response without a release")]
        [InlineData("""{"message": "Not Found"}""")]
        [InlineData("not json")]
        [InlineData("""{"tag_name": "v0.3.0"}""")]
        [InlineData("""{"tag_name": "v0.3.0", "html_url": "not a url"}""")]
        public void ShouldReturnNothingForAResponseWithoutARelease(string response)
        {
            Assert.Null(UpdateChecker.ParseRelease(response));
        }
    }

    public class CurrentVersion
    {
        [Fact(DisplayName = "should read the app version without build metadata")]
        public void ShouldReadTheAppVersionWithoutBuildMetadata()
        {
            Assert.Matches(@"^\d+\.\d+\.\d+$", UpdateChecker.CurrentVersion);
        }
    }

    public class Titles
    {
        [Fact(DisplayName = "should name the version on the button and the menu item")]
        public void ShouldNameTheVersionOnTheButtonAndTheMenuItem()
        {
            var release = new UpdateChecker.Release("0.3.0", new Uri(ReleasePage));

            Assert.Equal("Update to 0.3.0", UpdateChecker.ButtonTitle(release));
            Assert.Equal("Update to 0.3.0…", UpdateChecker.MenuItemTitle(release));
        }
    }
}
