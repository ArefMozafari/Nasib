namespace Nasib.App.Tests;

public class SettingsViewModelTests
{
    public class SaveProblem
    {
        [Fact(DisplayName = "should refuse to save without a location")]
        public void ShouldRefuseToSaveWithoutALocation()
        {
            Assert.Equal("Check at least one location.", SettingsViewModel.SaveProblem(locationCount: 0, protocolCount: 3));
        }

        [Fact(DisplayName = "should refuse to save without a protocol")]
        public void ShouldRefuseToSaveWithoutAProtocol()
        {
            Assert.Equal("Check at least one protocol.", SettingsViewModel.SaveProblem(locationCount: 2, protocolCount: 0));
        }

        [Fact(DisplayName = "should save when the locations are not loaded")]
        public void ShouldSaveWhenTheLocationsAreNotLoaded()
        {
            Assert.Null(SettingsViewModel.SaveProblem(locationCount: null, protocolCount: 1));
        }

        [Fact(DisplayName = "should save with a location and a protocol")]
        public void ShouldSaveWithALocationAndAProtocol()
        {
            Assert.Null(SettingsViewModel.SaveProblem(locationCount: 1, protocolCount: 1));
        }
    }
}
