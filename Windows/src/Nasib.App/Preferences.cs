using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nasib.App;

/// <summary>
/// App-only choices that the engine does not read, kept next to the engine's files.
/// </summary>
public sealed class Preferences
{
    private const string FileName = "preferences.json";

    private static readonly JsonSerializerOptions SerializerOptions = new() { WriteIndented = true };

    private readonly string path;

    private Preferences(string path, bool autoReconnect, string? notifiedUpdateVersion)
    {
        this.path = path;
        AutoReconnect = autoReconnect;
        NotifiedUpdateVersion = notifiedUpdateVersion;
    }

    /// <summary>
    /// On unless the user turned it off.
    /// </summary>
    public bool AutoReconnect { get; private set; }

    /// <summary>
    /// The newer version the user was already told about, so a restart doesn't notify again.
    /// </summary>
    public string? NotifiedUpdateVersion { get; private set; }

    public static Preferences Load(StatePaths paths)
    {
        var path = Path.Combine(paths.RootDirectory, FileName);
        try
        {
            var stored = JsonSerializer.Deserialize<StoredPreferences>(File.ReadAllText(path), SerializerOptions);
            return new Preferences(path, stored?.AutoReconnect ?? true, stored?.NotifiedUpdateVersion);
        }
        catch (Exception exception) when (exception is IOException or JsonException or UnauthorizedAccessException)
        {
            return new Preferences(path, autoReconnect: true, notifiedUpdateVersion: null);
        }
    }

    public void SetAutoReconnect(bool enabled)
    {
        AutoReconnect = enabled;
        Save();
    }

    public void SetNotifiedUpdateVersion(string version)
    {
        NotifiedUpdateVersion = version;
        Save();
    }

    private void Save()
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var stored = new StoredPreferences { AutoReconnect = AutoReconnect, NotifiedUpdateVersion = NotifiedUpdateVersion };
            File.WriteAllText(path, JsonSerializer.Serialize(stored, SerializerOptions));
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            // The choices still hold for this run; they fall back to the defaults next time.
        }
    }

    private sealed class StoredPreferences
    {
        [JsonPropertyName("autoReconnect")]
        public bool? AutoReconnect { get; set; }

        [JsonPropertyName("notifiedUpdateVersion")]
        public string? NotifiedUpdateVersion { get; set; }
    }
}
