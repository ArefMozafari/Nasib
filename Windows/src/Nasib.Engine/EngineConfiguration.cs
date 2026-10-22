using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nasib.Engine;

/// <summary>
/// Which cities to search and which protocols to try, in order, as saved from the Settings tab.
/// </summary>
public sealed record EngineConfiguration(IReadOnlyList<string> Cities, IReadOnlyList<string> ProtocolOrder)
{
    public static IReadOnlyList<string> AllProtocols { get; } =
        ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"];

    public static IReadOnlyList<string> DefaultCities { get; } =
    [
        "Istanbul", "Dubai", "Sofia", "Stockholm", "Amsterdam", "Frankfurt", "Zurich",
        "Toronto", "Montreal", "Winnipeg", "New York", "Chicago", "Atlanta",
        "Bratislava", "Helsinki", "Brussels",
    ];

    public static EngineConfiguration Default { get; } = new(DefaultCities, AllProtocols);

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
    };

    /// <summary>
    /// Reads the saved file, or returns the defaults when it is missing or unreadable.
    /// Protocols the engine does not know are dropped.
    /// </summary>
    public static EngineConfiguration Load(string path)
    {
        try
        {
            var stored = JsonSerializer.Deserialize<StoredConfiguration>(File.ReadAllText(path), SerializerOptions);
            if (stored?.Cities is null || stored.ProtocolOrder is null)
            {
                return Default;
            }
            var protocols = stored.ProtocolOrder.Where(AllProtocols.Contains).Distinct().ToList();
            return new EngineConfiguration(stored.Cities, protocols);
        }
        catch (Exception exception) when (exception is IOException or JsonException or UnauthorizedAccessException)
        {
            return Default;
        }
    }

    public void Save(string path)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var stored = new StoredConfiguration { Cities = [.. Cities], ProtocolOrder = [.. ProtocolOrder] };
        File.WriteAllText(path, JsonSerializer.Serialize(stored, SerializerOptions));
    }

    /// <summary>
    /// True when the city is searched: every city is when the list is empty.
    /// </summary>
    public bool IncludesCity(string city)
    {
        return Cities.Count == 0 || Cities.Contains(city, StringComparer.OrdinalIgnoreCase);
    }

    private sealed class StoredConfiguration
    {
        [JsonPropertyName("cities")]
        public List<string>? Cities { get; set; }

        [JsonPropertyName("protocolOrder")]
        public List<string>? ProtocolOrder { get; set; }
    }
}
