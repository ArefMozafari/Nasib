using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using Nasib.Engine;

namespace Nasib.App;

/// <summary>
/// App-only choices that the engine does not read, kept next to the engine's files.
/// </summary>
public sealed class Preferences
{
    private const string FileName = "preferences.json";

    private static readonly JsonSerializerOptions SerializerOptions = new() { WriteIndented = true };

    private readonly string path;

    private Preferences(string path, bool autoReconnect)
    {
        this.path = path;
        AutoReconnect = autoReconnect;
    }

    /// <summary>
    /// On unless the user turned it off.
    /// </summary>
    public bool AutoReconnect { get; private set; }

    public static Preferences Load(StatePaths paths)
    {
        var path = Path.Combine(paths.RootDirectory, FileName);
        try
        {
            var stored = JsonSerializer.Deserialize<StoredPreferences>(File.ReadAllText(path), SerializerOptions);
            return new Preferences(path, stored?.AutoReconnect ?? true);
        }
        catch (Exception exception) when (exception is IOException or JsonException or UnauthorizedAccessException)
        {
            return new Preferences(path, autoReconnect: true);
        }
    }

    public void SetAutoReconnect(bool enabled)
    {
        AutoReconnect = enabled;
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, JsonSerializer.Serialize(new StoredPreferences { AutoReconnect = enabled }, SerializerOptions));
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            // The choice still holds for this run; it falls back to the default next time.
        }
    }

    private sealed class StoredPreferences
    {
        [JsonPropertyName("autoReconnect")]
        public bool? AutoReconnect { get; set; }
    }
}
