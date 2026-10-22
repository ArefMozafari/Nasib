using System.Text.Json;

namespace Nasib.Engine;

/// <summary>
/// The datacenter list bundled with the app. Windscribe on Windows logs a datacenter's addresses only
/// when it connects to it, so the search starts from this list and the log adds to it.
/// </summary>
public static class SeedDatacenters
{
    private const string ResourceName = "Nasib.Engine.SeedDatacenters.json";

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    public static IReadOnlyList<Datacenter> Load()
    {
        using var stream = typeof(SeedDatacenters).Assembly.GetManifestResourceStream(ResourceName)
            ?? throw new InvalidOperationException($"The resource {ResourceName} is missing from the engine assembly.");
        var entries = JsonSerializer.Deserialize<List<SeedEntry>>(stream, SerializerOptions) ?? [];
        return entries.Select(entry => new Datacenter(entry.City, entry.Nickname, entry.Addresses)).ToList();
    }

    private sealed record SeedEntry(string City, string Nickname, List<string> Addresses);
}
