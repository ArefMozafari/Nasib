namespace Nasib.Engine;

/// <summary>
/// The datacenters to search: the bundled list with whatever the Windscribe log has added, limited to the chosen cities.
/// </summary>
public sealed class DatacenterCatalog
{
    private readonly IReadOnlyList<Datacenter> seed;
    private readonly string logDirectory;

    public DatacenterCatalog(IReadOnlyList<Datacenter> seed, string logDirectory)
    {
        this.seed = seed;
        this.logDirectory = logDirectory;
    }

    public IReadOnlyList<Datacenter> Load(EngineConfiguration configuration)
    {
        return Merge(seed, ClientLogParser.ReadDirectory(logDirectory), configuration);
    }

    /// <summary>
    /// Keeps the seed order, adds logged addresses to known nicknames, and appends nicknames only the log knows.
    /// </summary>
    internal static IReadOnlyList<Datacenter> Merge(
        IReadOnlyList<Datacenter> seed,
        IReadOnlyList<Datacenter> logged,
        EngineConfiguration configuration)
    {
        var merged = seed.ToList();
        foreach (var datacenter in logged)
        {
            var index = merged.FindIndex(existing => existing.Nickname == datacenter.Nickname);
            if (index < 0)
            {
                merged.Add(datacenter);
            }
            else
            {
                merged[index] = merged[index].WithAddresses(datacenter.Addresses);
            }
        }
        return merged.Where(datacenter => configuration.IncludesCity(datacenter.City)).ToList();
    }
}
