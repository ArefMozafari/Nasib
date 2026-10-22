using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using Nasib.Engine;

namespace Nasib.App;

/// <summary>
/// Edits the cities and protocol order the engine reads from config.json.
/// </summary>
public sealed class SettingsViewModel : ObservableObject
{
    private const string SavedMessage = "Saved. The next search uses these settings.";

    private readonly StatePaths paths;
    private string citiesText = string.Empty;
    private string? message;

    public SettingsViewModel(StatePaths paths)
    {
        this.paths = paths;
        Apply(EngineConfiguration.Load(paths.ConfigurationFile));
    }

    public string CitiesText
    {
        get => citiesText;
        set => SetProperty(ref citiesText, value);
    }

    public ObservableCollection<ProtocolRow> ProtocolRows { get; } = [];

    public string? Message
    {
        get => message;
        private set => SetProperty(ref message, value);
    }

    public void Save()
    {
        try
        {
            EditedConfiguration().Save(paths.ConfigurationFile);
            Message = SavedMessage;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            Message = exception.Message;
        }
    }

    public void ResetToDefaults()
    {
        Apply(EngineConfiguration.Default);
        Message = null;
    }

    /// <summary>
    /// Moves a protocol up (negative offset) or down (positive offset); does nothing at either end.
    /// </summary>
    public void Move(ProtocolRow row, int offset)
    {
        var from = ProtocolRows.IndexOf(row);
        var to = from + offset;
        if (from < 0 || to < 0 || to >= ProtocolRows.Count)
        {
            return;
        }
        ProtocolRows.Move(from, to);
    }

    private EngineConfiguration EditedConfiguration()
    {
        var cities = CitiesText
            .Split('\n')
            .Select(line => line.Trim())
            .Where(line => line.Length > 0)
            .ToList();
        var protocols = ProtocolRows.Where(row => row.Enabled).Select(row => row.Name).ToList();
        return new EngineConfiguration(cities, protocols);
    }

    /// <summary>
    /// Enabled protocols first in their saved order, then the disabled ones in default order.
    /// </summary>
    private void Apply(EngineConfiguration configuration)
    {
        CitiesText = string.Join(Environment.NewLine, configuration.Cities);
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
