using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class StatePathsTests
{
    public class Files
    {
        [Fact(DisplayName = "should keep the macOS file names under the root folder")]
        public void ShouldKeepTheMacOsFileNamesUnderTheRootFolder()
        {
            var paths = new StatePaths(Path.Combine("C:", "Nasib"));

            Assert.Equal(Path.Combine("C:", "Nasib", "log.csv"), paths.LogFile);
            Assert.Equal(Path.Combine("C:", "Nasib", "last_good"), paths.LastGoodFile);
            Assert.Equal(Path.Combine("C:", "Nasib", "config.json"), paths.ConfigurationFile);
        }
    }

    public class ForCurrentUser
    {
        [Fact(DisplayName = "should live in the local application data folder")]
        public void ShouldLiveInTheLocalApplicationDataFolder()
        {
            var localApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

            Assert.Equal(Path.Combine(localApplicationData, "Nasib"), StatePaths.ForCurrentUser().RootDirectory);
        }
    }

    public class EnsureExists
    {
        [Fact(DisplayName = "should create the root folder")]
        public void ShouldCreateTheRootFolder()
        {
            using var directory = new TemporaryDirectory();
            var paths = new StatePaths(directory.Combine("Nasib"));

            paths.EnsureExists();

            Assert.True(Directory.Exists(paths.RootDirectory));
        }
    }
}
