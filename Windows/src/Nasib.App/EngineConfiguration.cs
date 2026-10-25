using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nasib.App;

/// <summary>
/// Which datacenters to search and which protocols to try, in order, saved as config.json for the engine to
/// read. Every Nasib app writes the same format; Engine/README.md describes it.
/// </summary>
public sealed record EngineConfiguration(
    IReadOnlyList<string> Cities,
    IReadOnlyList<string> ProtocolOrder,
    IReadOnlyList<string>? Locations = null,
    bool RankBySuccess = true)
{
    public static IReadOnlyList<string> AllProtocols { get; } =
        ["wireguard", "ikev2", "tcp", "stealth", "wstunnel", "udp"];

    public static IReadOnlyList<string> DefaultCities { get; } =
    [
        "Istanbul", "Dubai", "Sofia", "Stockholm", "Amsterdam", "Frankfurt", "Zurich",
        "Toronto", "Montreal", "Winnipeg", "New York", "Chicago", "Atlanta",
        "Bratislava", "Helsinki", "Brussels",
    ];

    /// <summary>
    /// The engine's default locations, in the same order: the 20 datacenters Nasib searched before every
    /// datacenter was bundled.
    /// </summary>
    public static IReadOnlyList<string> DefaultLocations { get; } =
    [
        "Lygos", "Khalifa", "Vitoshka", "Nevski", "Fika", "Djurgarden", "Stolen Cheese", "Wurstchen",
        "Sausage Party", "Altstadt", "The 6", "Bagel Poutine", "The Forks", "Inside Job", "Green Mill",
        "Magic City", "Devin Castle", "Sauna", "Tram", "Guildhouse",
    ];

    /// <summary>
    /// No saved locations, so the engine carries over the default cities, as an untouched install does.
    /// </summary>
    public static EngineConfiguration Default { get; } = new(DefaultCities, AllProtocols);

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    /// <summary>
    /// Reads the saved file, or returns the defaults when it is missing, unreadable or has no protocol order.
    /// A file saved before locations existed loads with locations unset and ranking on.
    /// </summary>
    public static EngineConfiguration Load(string path)
    {
        try
        {
            var stored = JsonSerializer.Deserialize<StoredConfiguration>(File.ReadAllText(path), SerializerOptions);
            if (stored?.ProtocolOrder is null)
            {
                return Default;
            }
            return new EngineConfiguration(
                stored.Cities ?? DefaultCities,
                KnownProtocols(stored.ProtocolOrder),
                stored.Locations,
                stored.RankBySuccess ?? true);
        }
        catch (Exception exception) when (exception is IOException or JsonException or UnauthorizedAccessException)
        {
            return Default;
        }
    }

    /// <summary>
    /// Writes the file. Unset locations are left out, so the engine keeps carrying the cities over.
    /// </summary>
    public void Save(string path)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var stored = new StoredConfiguration
        {
            Locations = Locations?.ToList(),
            RankBySuccess = RankBySuccess,
            ProtocolOrder = [.. ProtocolOrder],
            Cities = [.. Cities],
        };
        File.WriteAllText(path, JsonSerializer.Serialize(stored, SerializerOptions));
    }

    /// <summary>
    /// True when the city is one of the given ones, ignoring case; every city is when none are given.
    /// </summary>
    public static bool IncludesCity(string city, IReadOnlyList<string> cities)
    {
        return cities.Count == 0 || cities.Contains(city, StringComparer.OrdinalIgnoreCase);
    }

    /// <summary>
    /// Keeps only protocols the engine knows, each once, in the order given.
    /// </summary>
    private static List<string> KnownProtocols(IEnumerable<string> protocols)
    {
        return protocols.Where(AllProtocols.Contains).Distinct().ToList();
    }

    private sealed class StoredConfiguration
    {
        [JsonPropertyName("locations")]
        public List<string>? Locations { get; set; }

        [JsonPropertyName("rankBySuccess")]
        public bool? RankBySuccess { get; set; }

        [JsonPropertyName("protocolOrder")]
        public List<string>? ProtocolOrder { get; set; }

        [JsonPropertyName("cities")]
        public List<string>? Cities { get; set; }
    }
}
