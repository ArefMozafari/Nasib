namespace Nasib.Engine;

/// <summary>
/// Orders datacenters or protocols by how often they worked recently, so the order follows current censorship.
/// </summary>
public static class SuccessRanker
{
    /// <summary>
    /// Only this many of the most recent attempts count towards a success rate.
    /// </summary>
    public const int RankWindowLines = 300;

    /// <summary>
    /// Rates are compared at this many decimal places, as the macOS engine does.
    /// </summary>
    private const int RateDecimalPlaces = 3;

    /// <summary>
    /// Returns the names best first. Names with equal rates, including ones never tried, keep the order they were given in.
    /// </summary>
    public static IReadOnlyList<string> Rank(
        IEnumerable<string> names,
        IEnumerable<AttemptRecord> recentAttempts,
        Func<AttemptRecord, string> keySelector)
    {
        var rates = recentAttempts
            .GroupBy(keySelector)
            .ToDictionary(
                group => group.Key,
                group => Math.Round((double)group.Count(attempt => attempt.Succeeded) / group.Count(), RateDecimalPlaces));
        return names
            .Select((name, index) => (Name: name, Index: index, Rate: rates.GetValueOrDefault(name)))
            .OrderByDescending(entry => entry.Rate)
            .ThenBy(entry => entry.Index)
            .Select(entry => entry.Name)
            .ToList();
    }
}
