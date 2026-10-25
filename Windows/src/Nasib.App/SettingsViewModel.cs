using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;

namespace Nasib.App;

/// <summary>
/// Edits the locations, ranking and protocol order the engine reads from config.json.
/// </summary>
public sealed class SettingsViewModel : ObservableObject
{
    private const string SavedMessage = "Saved. The next search uses these settings.";
    private const string EmptyOrderMessage = "Check at least one location.";
    private const string NoProtocolMessage = "Check at least one protocol.";
    private const string UnreadableCatalogMessage = "Couldn't read the datacenter list.";
    private const string CitiesKeptMessage = "Couldn't read the datacenter list, so your saved cities stay as they are.";

    private readonly StatePaths paths;

    /// <summary>
    /// What was saved, kept so the cities older versions saved carry through a save untouched.
    /// </summary>
    private EngineConfiguration saved;

    private EngineCatalog? catalog;
    private LocationSelection? selection;
    private bool rankBySuccess;
    private string orderSearchText = string.Empty;
    private string catalogSearchText = string.Empty;
    private string? message;

    public SettingsViewModel(StatePaths paths, Func<Task<EngineCatalog?>> readCatalog)
    {
        this.paths = paths;
        saved = EngineConfiguration.Load(paths.ConfigurationFile);
        rankBySuccess = saved.RankBySuccess;
        ApplyProtocols(saved);
        _ = LoadCatalogAsync(readCatalog);
    }

    public ObservableCollection<ProtocolRow> ProtocolRows { get; } = [];

    /// <summary>
    /// The checked datacenters that match the order's search, in the user's order.
    /// </summary>
    public ObservableCollection<LocationRow> OrderRows { get; } = [];

    /// <summary>
    /// The regions with only the datacenters that match the catalog's search, in catalog order.
    /// </summary>
    public ObservableCollection<RegionGroup> CatalogGroups { get; } = [];

    /// <summary>
    /// True until the engine's catalog has been read.
    /// </summary>
    public bool IsLoadingLocations => catalog is null;

    /// <summary>
    /// True once there is an order to edit: the catalog was read, or saved locations stand on their own.
    /// </summary>
    public bool HasLocations => selection is not null;

    /// <summary>
    /// Why the Locations section is missing or incomplete, or null when it isn't.
    /// </summary>
    public string? LocationsProblem => catalog is null
        ? null
        : selection is null ? CitiesKeptMessage
        : catalog.Regions.Count == 0 ? UnreadableCatalogMessage
        : null;

    public string CheckedCountText => $"{selection?.Ordered.Count ?? 0} of {catalog?.Entries.Count ?? 0} checked";

    public bool RankBySuccess
    {
        get => rankBySuccess;
        set => SetProperty(ref rankBySuccess, value);
    }

    public string OrderSearchText
    {
        get => orderSearchText;
        set
        {
            if (SetProperty(ref orderSearchText, value))
            {
                RefreshOrderRows();
            }
        }
    }

    public string CatalogSearchText
    {
        get => catalogSearchText;
        set
        {
            if (SetProperty(ref catalogSearchText, value))
            {
                RebuildCatalogGroups();
            }
        }
    }

    public string? Message
    {
        get => message;
        private set => SetProperty(ref message, value);
    }

    /// <summary>
    /// Why the settings can't be saved, or null when they can. An empty location list would mean every
    /// datacenter to the engine, and an empty protocol list would mean nothing is ever tried. A null location
    /// count means the locations aren't being edited, so the saved ones are kept.
    /// </summary>
    public static string? SaveProblem(int? locationCount, int protocolCount)
    {
        if (locationCount == 0)
        {
            return EmptyOrderMessage;
        }
        return protocolCount == 0 ? NoProtocolMessage : null;
    }

    public void Save()
    {
        var protocols = ProtocolRows.Where(row => row.Enabled).Select(row => row.Name).ToList();
        if (SaveProblem(selection?.Ordered.Count, protocols.Count) is { } problem)
        {
            Message = problem;
            return;
        }
        try
        {
            var configuration = new EngineConfiguration(saved.Cities, protocols, selection?.Nicknames ?? saved.Locations, RankBySuccess);
            configuration.Save(paths.ConfigurationFile);
            saved = configuration;
            Message = SavedMessage;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            Message = exception.Message;
        }
    }

    public void ResetToDefaults()
    {
        selection?.ResetToDefaults();
        RankBySuccess = EngineConfiguration.Default.RankBySuccess;
        ApplyProtocols(EngineConfiguration.Default);
        Message = null;
    }

    /// <summary>
    /// Moves a protocol up (negative offset) or down (positive offset); does nothing at either end.
    /// </summary>
    public void MoveProtocol(ProtocolRow row, int offset)
    {
        var from = ProtocolRows.IndexOf(row);
        var to = from + offset;
        if (from < 0 || to < 0 || to >= ProtocolRows.Count)
        {
            return;
        }
        ProtocolRows.Move(from, to);
    }

    public void SetAllProtocols(bool enabled)
    {
        foreach (var row in ProtocolRows)
        {
            row.Enabled = enabled;
        }
    }

    /// <summary>
    /// Moves a location up (negative offset) or down (positive offset) in the order.
    /// </summary>
    public void MoveLocation(LocationRow row, int offset)
    {
        if (selection is null || !row.CanMove)
        {
            return;
        }
        var from = selection.Nicknames.ToList().IndexOf(row.Entry.Nickname);
        selection.Move(from, from + offset);
    }

    /// <summary>
    /// Unchecks what the order's search shows.
    /// </summary>
    public void DeselectShownOrder()
    {
        selection?.Deselect(OrderRows.Select(row => row.Entry.Nickname).ToList());
    }

    /// <summary>
    /// Checks or unchecks what the catalog's search shows.
    /// </summary>
    public void SetShownCatalog(bool selected)
    {
        var nicknames = CatalogGroups.SelectMany(group => group.Nicknames).ToList();
        if (selected)
        {
            selection?.Select(nicknames);
        }
        else
        {
            selection?.Deselect(nicknames);
        }
    }

    /// <summary>
    /// Checks every datacenter a region shows, or unchecks them once all are checked.
    /// </summary>
    public void ToggleRegion(RegionGroup group)
    {
        if (group.AllSelected)
        {
            selection?.Deselect(group.Nicknames.ToList());
        }
        else
        {
            selection?.Select(group.Nicknames.ToList());
        }
    }

    private static string Query(string text) => text.Trim();

    private static bool Matches(string text, string query) => text.Contains(query, StringComparison.CurrentCultureIgnoreCase);

    /// <summary>
    /// Reads the catalog without blocking the window. Without it, saved locations stay editable on their own.
    /// </summary>
    private async Task LoadCatalogAsync(Func<Task<EngineCatalog?>> readCatalog)
    {
        var loaded = await readCatalog();
        catalog = loaded ?? EngineCatalog.Empty;
        selection = LocationSelection.Create(saved, catalog);
        if (selection is not null)
        {
            selection.Changed += OnSelectionChanged;
        }
        RebuildCatalogGroups();
        RefreshOrderRows();
        OnPropertyChanged(nameof(IsLoadingLocations));
        OnPropertyChanged(nameof(HasLocations));
        OnPropertyChanged(nameof(LocationsProblem));
        OnPropertyChanged(nameof(CheckedCountText));
    }

    private void OnSelectionChanged()
    {
        RefreshOrderRows();
        foreach (var group in CatalogGroups)
        {
            group.Refresh(selection!);
        }
        OnPropertyChanged(nameof(CheckedCountText));
    }

    private void ToggleLocation(string nickname)
    {
        selection?.Toggle(nickname);
    }

    private void RefreshOrderRows()
    {
        OrderRows.Clear();
        if (selection is null)
        {
            return;
        }
        var query = Query(OrderSearchText);
        var canMove = query.Length == 0;
        foreach (var entry in selection.Ordered.Where(entry => query.Length == 0 || Matches(entry.Title, query)))
        {
            OrderRows.Add(new LocationRow(entry, isChecked: true, ToggleLocation) { CanMove = canMove });
        }
    }

    private void RebuildCatalogGroups()
    {
        CatalogGroups.Clear();
        if (catalog is null || selection is null)
        {
            return;
        }
        var query = Query(CatalogSearchText);
        foreach (var region in catalog.Regions)
        {
            IEnumerable<CatalogEntry> shown = query.Length == 0 || Matches(region.Name, query)
                ? region.Datacenters
                : region.Datacenters.Where(entry => Matches(entry.Title, query));
            var rows = shown.Select(entry => new LocationRow(entry, selection.IsSelected(entry.Nickname), ToggleLocation)).ToList();
            if (rows.Count == 0)
            {
                continue;
            }
            var group = new RegionGroup(region.Name, rows);
            group.Refresh(selection);
            CatalogGroups.Add(group);
        }
    }

    /// <summary>
    /// Enabled protocols first in their saved order, then the disabled ones in default order.
    /// </summary>
    private void ApplyProtocols(EngineConfiguration configuration)
    {
        ProtocolRows.Clear();
        foreach (var protocol in configuration.ProtocolOrder)
        {
            ProtocolRows.Add(new ProtocolRow(protocol, enabled: true));
        }
        foreach (var protocol in EngineConfiguration.AllProtocols.Except(configuration.ProtocolOrder))
        {
            ProtocolRows.Add(new ProtocolRow(protocol, enabled: false));
        }
    }
}
