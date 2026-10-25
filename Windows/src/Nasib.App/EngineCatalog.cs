using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Nasib.App;

/// <summary>
/// One datacenter in the catalog. Logged is true when the Windscribe log names it; settings from before
/// locations carry these over.
/// </summary>
public sealed record CatalogEntry(string City, string Nickname, bool Logged = false)
{
    private const string LocationNameSeparator = " - ";

    /// <summary>
    /// The name Windscribe shows, such as "Toronto - The 6".
    /// </summary>
    public string Title => City.Length == 0 ? Nickname : City + LocationNameSeparator + Nickname;
}

public sealed record CatalogRegion(string Name, IReadOnlyList<CatalogEntry> Datacenters);

/// <summary>
/// Every datacenter the engine knows, grouped by Windscribe's region names, as `nasib-engine --catalog`
/// prints it. Settings lists it so users can pick and order their locations.
/// </summary>
public sealed record EngineCatalog(IReadOnlyList<CatalogRegion> Regions)
{
    private static readonly JsonSerializerOptions SerializerOptions = new() { PropertyNameCaseInsensitive = true };

    public static EngineCatalog Empty { get; } = new([]);

    /// <summary>
    /// Every datacenter, in region order.
    /// </summary>
    public IReadOnlyList<CatalogEntry> Entries => Regions.SelectMany(region => region.Datacenters).ToList();

    /// <summary>
    /// Reads the one line `--catalog` prints; null for anything else.
    /// </summary>
    public static EngineCatalog? Decode(string line)
    {
        try
        {
            var stored = JsonSerializer.Deserialize<StoredCatalog>(line, SerializerOptions);
            if (stored?.Regions is null)
            {
                return null;
            }
            return new EngineCatalog(stored.Regions
                .Select(region => new CatalogRegion(
                    region.Name ?? string.Empty,
                    (region.Datacenters ?? [])
                        .Select(entry => new CatalogEntry(entry.City ?? string.Empty, entry.Nickname ?? string.Empty, entry.Logged ?? false))
                        .ToList()))
                .ToList());
        }
        catch (JsonException)
        {
            return null;
        }
    }

    private sealed class StoredCatalog
    {
        [JsonPropertyName("regions")]
        public List<StoredRegion>? Regions { get; set; }
    }

    private sealed class StoredRegion
    {
        [JsonPropertyName("name")]
        public string? Name { get; set; }

        [JsonPropertyName("datacenters")]
        public List<StoredEntry>? Datacenters { get; set; }
    }

    /// <summary>
    /// The engine leaves logged out unless it's true.
    /// </summary>
    private sealed class StoredEntry
    {
        [JsonPropertyName("city")]
        public string? City { get; set; }

        [JsonPropertyName("nickname")]
        public string? Nickname { get; set; }

        [JsonPropertyName("logged")]
        public bool? Logged { get; set; }
    }
}
