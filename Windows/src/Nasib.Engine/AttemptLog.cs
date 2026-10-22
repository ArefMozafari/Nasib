using System.Globalization;

namespace Nasib.Engine;

/// <summary>
/// One line of log.csv: when, which datacenter, which protocol, and whether a blocked site loaded.
/// </summary>
public sealed record AttemptRecord(string Timestamp, string Nickname, string Protocol, bool Succeeded);

/// <summary>
/// A datacenter and protocol that carried traffic.
/// </summary>
public sealed record ServerPair(string Nickname, string Protocol);

/// <summary>
/// The attempt history in log.csv and the last pair that worked in last_good, in the same formats as on macOS.
/// </summary>
public sealed class AttemptLog
{
    private const string TimestampFormat = "yyyy-MM-ddTHH:mm:ss";
    private const char FieldSeparator = ',';
    private const char PairSeparator = '|';
    private const string SucceededValue = "ok";
    private const string FailedValue = "fail";
    private const int FieldCount = 4;

    private readonly StatePaths paths;
    private readonly TimeProvider timeProvider;

    public AttemptLog(StatePaths paths, TimeProvider timeProvider)
    {
        this.paths = paths;
        this.timeProvider = timeProvider;
    }

    public void Record(string nickname, string protocol, bool succeeded)
    {
        paths.EnsureExists();
        var timestamp = timeProvider.GetLocalNow().ToString(TimestampFormat, CultureInfo.InvariantCulture);
        var result = succeeded ? SucceededValue : FailedValue;
        var line = string.Join(FieldSeparator, timestamp, nickname, protocol, result);
        File.AppendAllText(paths.LogFile, line + "\n");
    }

    /// <summary>
    /// Returns the last lines of the log, oldest first, skipping any that are malformed.
    /// </summary>
    public IReadOnlyList<AttemptRecord> ReadRecent(int count)
    {
        if (!File.Exists(paths.LogFile))
        {
            return [];
        }
        return File.ReadLines(paths.LogFile)
            .TakeLast(count)
            .Select(ParseLine)
            .OfType<AttemptRecord>()
            .ToList();
    }

    public ServerPair? ReadLastGood()
    {
        if (!File.Exists(paths.LastGoodFile))
        {
            return null;
        }
        var parts = File.ReadAllText(paths.LastGoodFile).Trim().Split(PairSeparator);
        return parts.Length == 2 && parts[0].Length > 0 && parts[1].Length > 0
            ? new ServerPair(parts[0], parts[1])
            : null;
    }

    public void WriteLastGood(ServerPair pair)
    {
        paths.EnsureExists();
        File.WriteAllText(paths.LastGoodFile, pair.Nickname + PairSeparator + pair.Protocol + "\n");
    }

    private static AttemptRecord? ParseLine(string line)
    {
        var fields = line.Split(FieldSeparator);
        if (fields.Length != FieldCount)
        {
            return null;
        }
        return new AttemptRecord(fields[0], fields[1], fields[2], fields[3] == SucceededValue);
    }
}
