using System;
using System.Collections.Generic;
using System.Linq;

namespace Nasib.App;

/// <summary>
/// One datacenter in a Settings list, with a check box that adds it to the order or takes it out.
/// </summary>
public sealed class LocationRow : ObservableObject
{
    private readonly Action<string> toggle;
    private bool isChecked;
    private bool canMove;

    public LocationRow(CatalogEntry entry, bool isChecked, Action<string> toggle)
    {
        Entry = entry;
        this.isChecked = isChecked;
        this.toggle = toggle;
    }

    public CatalogEntry Entry { get; }

    public string Title => Entry.Title;

    /// <summary>
    /// Checking or unchecking it changes the order; the order then updates every row's check mark.
    /// </summary>
    public bool IsChecked
    {
        get => isChecked;
        set
        {
            if (value != isChecked)
            {
                toggle(Entry.Nickname);
            }
        }
    }

    /// <summary>
    /// False while the order is filtered, since a filtered row's place in the order is ambiguous.
    /// </summary>
    public bool CanMove
    {
        get => canMove;
        set => SetProperty(ref canMove, value);
    }

    /// <summary>
    /// Shows the order's current state without changing it.
    /// </summary>
    public void ShowChecked(bool value)
    {
        SetProperty(ref isChecked, value, nameof(IsChecked));
    }
}

/// <summary>
/// One of Windscribe's regions in the Add locations list, with the rows its search shows.
/// </summary>
public sealed class RegionGroup : ObservableObject
{
    private const string SelectAllTitle = "Select all";
    private const string DeselectAllTitle = "Deselect all";

    private bool allSelected;

    public RegionGroup(string name, IReadOnlyList<LocationRow> rows)
    {
        Name = name;
        Rows = rows;
    }

    public string Name { get; }

    public IReadOnlyList<LocationRow> Rows { get; }

    public IEnumerable<string> Nicknames => Rows.Select(row => row.Entry.Nickname);

    /// <summary>
    /// The region's one link checks every datacenter it shows, or unchecks them once all are checked.
    /// </summary>
    public bool AllSelected
    {
        get => allSelected;
        private set
        {
            if (SetProperty(ref allSelected, value))
            {
                OnPropertyChanged(nameof(LinkTitle));
            }
        }
    }

    public string LinkTitle => AllSelected ? DeselectAllTitle : SelectAllTitle;

    public void Refresh(LocationSelection selection)
    {
        foreach (var row in Rows)
        {
            row.ShowChecked(selection.IsSelected(row.Entry.Nickname));
        }
        AllSelected = selection.AreAllSelected(Nicknames);
    }
}
