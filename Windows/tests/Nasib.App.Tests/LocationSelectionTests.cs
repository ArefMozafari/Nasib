namespace Nasib.App.Tests;

public class LocationSelectionTests
{
    private static readonly EngineCatalog Catalog = new(
    [
        new CatalogRegion("Turkey", [new CatalogEntry("Istanbul", "Lygos")]),
        new CatalogRegion("United Arab Emirates", [new CatalogEntry("Dubai", "Khalifa")]),
        new CatalogRegion("Canada East",
        [
            new CatalogEntry("Toronto", "The 6"),
            new CatalogEntry("Toronto", "Comfort Zone", Logged: true),
            new CatalogEntry("Toronto", "Peak"),
        ]),
        new CatalogRegion("Other", [new CatalogEntry("Toronto", "Drake", Logged: true)]),
    ]);

    private static readonly EngineCatalog EmptyCatalog = new([]);

    private static EngineConfiguration Saved(IReadOnlyList<string>? locations = null, IReadOnlyList<string>? cities = null) =>
        EngineConfiguration.Default with { Locations = locations, Cities = cities ?? EngineConfiguration.DefaultCities };

    private static LocationSelection Selection(params string[] locations) => LocationSelection.Create(Saved(locations), Catalog)!;

    public class Create
    {
        [Fact(DisplayName = "should start from the saved locations in order")]
        public void ShouldStartFromTheSavedLocationsInOrder()
        {
            Assert.Equal(["The 6", "Lygos"], Selection("The 6", "Lygos").Nicknames);
        }

        [Fact(DisplayName = "should start from the default locations, then the logged ones, in the saved cities")]
        public void ShouldStartFromTheDefaultLocationsThenTheLoggedOnesInTheSavedCities()
        {
            var selection = LocationSelection.Create(Saved(cities: ["Toronto", "Dubai"]), Catalog);

            Assert.Equal(["Khalifa", "The 6", "Comfort Zone", "Drake"], selection!.Nicknames);
        }

        [Fact(DisplayName = "should start from every location when the saved list is empty")]
        public void ShouldStartFromEveryLocationWhenTheSavedListIsEmpty()
        {
            var selection = LocationSelection.Create(Saved(locations: []), Catalog);

            Assert.Equal(["Lygos", "Khalifa", "The 6", "Comfort Zone", "Peak", "Drake"], selection!.Nicknames);
        }

        [Fact(DisplayName = "should keep the saved locations when the catalog cannot be read")]
        public void ShouldKeepTheSavedLocationsWhenTheCatalogCannotBeRead()
        {
            var selection = LocationSelection.Create(Saved(locations: ["The 6", "Lygos"]), EmptyCatalog);

            Assert.Equal(["The 6", "Lygos"], selection!.Nicknames);
        }

        // Without the catalog, the old cities can't be turned into locations, so Settings must not write any.
        [Fact(DisplayName = "should not start without a catalog when only cities are saved")]
        public void ShouldNotStartWithoutACatalogWhenOnlyCitiesAreSaved()
        {
            Assert.Null(LocationSelection.Create(Saved(cities: ["Toronto"]), EmptyCatalog));
        }
    }

    public class Toggle
    {
        [Fact(DisplayName = "should append when toggled on")]
        public void ShouldAppendWhenToggledOn()
        {
            var selection = Selection("The 6");

            selection.Toggle("Peak");

            Assert.Equal(["The 6", "Peak"], selection.Nicknames);
            Assert.True(selection.IsSelected("Peak"));
        }

        [Fact(DisplayName = "should remove when toggled off")]
        public void ShouldRemoveWhenToggledOff()
        {
            var selection = Selection("The 6", "Lygos");

            selection.Toggle("The 6");

            Assert.Equal(["Lygos"], selection.Nicknames);
            Assert.False(selection.IsSelected("The 6"));
        }
    }

    public class Move
    {
        [Fact(DisplayName = "should move a row")]
        public void ShouldMoveARow()
        {
            var selection = Selection("The 6", "Lygos", "Khalifa");

            selection.Move(2, 0);

            Assert.Equal(["Khalifa", "The 6", "Lygos"], selection.Nicknames);
        }

        [Fact(DisplayName = "should ignore a move past either end")]
        public void ShouldIgnoreAMovePastEitherEnd()
        {
            var selection = Selection("The 6", "Lygos");

            selection.Move(0, -1);
            selection.Move(1, 2);

            Assert.Equal(["The 6", "Lygos"], selection.Nicknames);
        }
    }

    public class ResetToDefaults
    {
        [Fact(DisplayName = "should reset to what an untouched install searches")]
        public void ShouldResetToWhatAnUntouchedInstallSearches()
        {
            var selection = Selection("Peak");

            selection.ResetToDefaults();

            Assert.Equal(["Lygos", "Khalifa", "The 6", "Comfort Zone", "Drake"], selection.Nicknames);
        }
    }

    public class Select
    {
        [Fact(DisplayName = "should append the given locations after the order, in their order")]
        public void ShouldAppendTheGivenLocationsAfterTheOrderInTheirOrder()
        {
            var selection = Selection("Comfort Zone");

            selection.Select(["The 6", "Comfort Zone", "Peak"]);

            Assert.Equal(["Comfort Zone", "The 6", "Peak"], selection.Nicknames);
            Assert.True(selection.AreAllSelected(["The 6", "Comfort Zone", "Peak"]));
        }
    }

    public class Deselect
    {
        [Fact(DisplayName = "should deselect only the given locations")]
        public void ShouldDeselectOnlyTheGivenLocations()
        {
            var selection = Selection("Lygos", "The 6", "Peak");

            selection.Deselect(["The 6", "Peak", "Drake"]);

            Assert.Equal(["Lygos"], selection.Nicknames);
            Assert.False(selection.AreAllSelected(["Lygos", "The 6"]));
        }

        [Fact(DisplayName = "should deselect everything and refuse to save")]
        public void ShouldDeselectEverythingAndRefuseToSave()
        {
            var selection = Selection("Lygos", "The 6");

            selection.DeselectAll();

            Assert.Empty(selection.Nicknames);
            Assert.False(selection.CanSave);
        }
    }

    public class Changed
    {
        [Fact(DisplayName = "should tell listeners about every change")]
        public void ShouldTellListenersAboutEveryChange()
        {
            var selection = Selection("Lygos");
            var changes = 0;
            selection.Changed += () => changes++;

            selection.Toggle("Peak");
            selection.Move(1, 0);
            selection.DeselectAll();

            Assert.Equal(3, changes);
        }
    }
}
