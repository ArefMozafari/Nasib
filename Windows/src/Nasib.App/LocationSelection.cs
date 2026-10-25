using System;
using System.Collections.Generic;
using System.Linq;

namespace Nasib.App;

/// <summary>
/// The locations checked in Settings, in the user's order. It starts from what the engine would search for
/// the saved settings, so opening Settings never changes the search by itself.
/// </summary>
public sealed class LocationSelection
{
    private readonly EngineCatalog catalog;
    private readonly List<CatalogEntry> ordered;

    private LocationSelection(EngineCatalog catalog, IEnumerable<CatalogEntry> ordered)
    {
        this.catalog = catalog;
        this.ordered = ordered.ToList();
    }

    /// <summary>
    /// Raised after every change to the order.
    /// </summary>
    public event Action? Changed;

    public IReadOnlyList<CatalogEntry> Ordered => ordered;

    public IReadOnlyList<string> Nicknames => ordered.Select(entry => entry.Nickname).ToList();

    /// <summary>
    /// An empty order would mean every datacenter to the engine, so Settings doesn't save one.
    /// </summary>
    public bool CanSave => ordered.Count > 0;

    /// <summary>
    /// What the engine searches without saved locations: the default locations in the given cities, then
    /// every other datacenter the Windscribe log knows in those cities.
    /// </summary>
    private static List<CatalogEntry> CarriedOver(IReadOnlyList<string> cities, EngineCatalog catalog)
    {
        var entries = catalog.Entries;
        var defaults = EngineConfiguration.DefaultLocations
            .Select(nickname => entries.FirstOrDefault(entry => entry.Nickname == nickname))
            .OfType<CatalogEntry>()
            .Where(entry => EngineConfiguration.IncludesCity(entry.City, cities))
            .ToList();
        var logged = entries
            .Where(entry => entry.Logged && EngineConfiguration.IncludesCity(entry.City, cities) && !defaults.Contains(entry));
        return [.. defaults, .. logged];
    }

    /// <summary>
    /// Null when only cities are saved and the catalog couldn't be read: the cities can't be turned into
    /// locations then, and Settings must leave them as they are.
    /// </summary>
    public static LocationSelection? Create(EngineConfiguration saved, EngineCatalog catalog)
    {
        var entries = catalog.Entries;
        if (saved.Locations is null)
        {
            return catalog.Regions.Count == 0 ? null : new LocationSelection(catalog, CarriedOver(saved.Cities, catalog));
        }
        if (saved.Locations.Count == 0)
        {
            return new LocationSelection(catalog, entries);
        }

        // Without a catalog, the saved order stays editable under the nicknames alone.
        var ordered = saved.Locations
            .Select(nickname => entries.FirstOrDefault(entry => entry.Nickname == nickname)
                ?? (catalog.Regions.Count == 0 ? new CatalogEntry(string.Empty, nickname) : null))
            .OfType<CatalogEntry>();
        return new LocationSelection(catalog, ordered);
    }

    public bool IsSelected(string nickname)
    {
        return ordered.Any(entry => entry.Nickname == nickname);
    }

    public bool AreAllSelected(IEnumerable<string> nicknames)
    {
        return nicknames.All(IsSelected);
    }

    /// <summary>
    /// Appends the datacenter to the order, or removes it when it's already there.
    /// </summary>
    public void Toggle(string nickname)
    {
        ToggleQuietly(nickname);
        Changed?.Invoke();
    }

    /// <summary>
    /// Appends the given datacenters that aren't checked yet, in the order given.
    /// </summary>
    public void Select(IEnumerable<string> nicknames)
    {
        foreach (var nickname in nicknames.Where(nickname => !IsSelected(nickname)).ToList())
        {
            ToggleQuietly(nickname);
        }
        Changed?.Invoke();
    }

    public void Deselect(IEnumerable<string> nicknames)
    {
        var removed = nicknames.ToHashSet();
        ordered.RemoveAll(entry => removed.Contains(entry.Nickname));
        Changed?.Invoke();
    }

    public void DeselectAll()
    {
        ordered.Clear();
        Changed?.Invoke();
    }

    /// <summary>
    /// Moves one row to another position; a move past either end is ignored.
    /// </summary>
    public void Move(int from, int to)
    {
        if (from < 0 || from >= ordered.Count || to < 0 || to >= ordered.Count)
        {
            return;
        }
        var entry = ordered[from];
        ordered.RemoveAt(from);
        ordered.Insert(to, entry);
        Changed?.Invoke();
    }

    /// <summary>
    /// Restores what an untouched install searches.
    /// </summary>
    public void ResetToDefaults()
    {
        ordered.Clear();
        ordered.AddRange(CarriedOver(EngineConfiguration.DefaultCities, catalog));
        Changed?.Invoke();
    }

    private void ToggleQuietly(string nickname)
    {
        var index = ordered.FindIndex(entry => entry.Nickname == nickname);
        if (index >= 0)
        {
            ordered.RemoveAt(index);
        }
        else if (catalog.Entries.FirstOrDefault(entry => entry.Nickname == nickname) is { } entry)
        {
            ordered.Add(entry);
        }
    }
}
