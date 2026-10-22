using System.Text.Json;
using System.Text.RegularExpressions;

namespace Nasib.Engine;

/// <summary>
/// Reads datacenters out of the Windscribe app's client log. Each line is one JSON object, and the
/// lines that matter carry a "msg" such as:
/// MutableLocationInfo created:  "Amsterdam - Stolen Cheese" "203.0.113.1; 203.0.113.2; " ; Selected node: 1
/// </summary>
public static partial class ClientLogParser
{
    /// <summary>
    /// The current log and the one before it; Windscribe rotates on every start.
    /// </summary>
    public static IReadOnlyList<string> LogFileNames { get; } = ["client.1.log", "client.log"];

    private const string MessagePropertyName = "msg";
    private const char AddressSeparator = ';';

    [GeneratedRegex("""MutableLocationInfo created:\s+"(?<name>[^"]+)"\s+"(?<addresses>[0-9.;\s]*)""")]
    private static partial Regex LocationLinePattern();

    /// <summary>
    /// The folder Windscribe 2 writes its logs to under %LOCALAPPDATA%.
    /// </summary>
    public static string DefaultLogDirectory()
    {
        var localApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return Path.Combine(localApplicationData, "Windscribe", "Windscribe2");
    }

    /// <summary>
    /// Returns one datacenter per location line, or null for any other line.
    /// Lines that are not JSON are matched as they are, which covers older plain-text logs.
    /// </summary>
    public static Datacenter? ParseLine(string line)
    {
        var match = LocationLinePattern().Match(MessageOf(line));
        if (!match.Success)
        {
            return null;
        }
        var addresses = match.Groups["addresses"].Value
            .Split(AddressSeparator, StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        return Datacenter.FromLocationName(match.Groups["name"].Value, addresses);
    }

    /// <summary>
    /// Reads both log files in the folder and merges every address seen per nickname, in the order first seen.
    /// Missing or locked files are skipped.
    /// </summary>
    public static IReadOnlyList<Datacenter> ReadDirectory(string directory)
    {
        var merged = new List<Datacenter>();
        foreach (var fileName in LogFileNames)
        {
            foreach (var line in ReadLinesShared(Path.Combine(directory, fileName)))
            {
                var datacenter = ParseLine(line);
                if (datacenter is null)
                {
                    continue;
                }
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
        }
        return merged;
    }

    private static string MessageOf(string line)
    {
        try
        {
            using var document = JsonDocument.Parse(line);
            if (document.RootElement.ValueKind == JsonValueKind.Object
                && document.RootElement.TryGetProperty(MessagePropertyName, out var message)
                && message.ValueKind == JsonValueKind.String)
            {
                return message.GetString() ?? string.Empty;
            }
            return string.Empty;
        }
        catch (JsonException)
        {
            return line;
        }
    }

    /// <summary>
    /// Opens the file without locking it, since Windscribe keeps writing to it while it runs.
    /// </summary>
    private static IEnumerable<string> ReadLinesShared(string path)
    {
        if (!File.Exists(path))
        {
            return [];
        }
        try
        {
            using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete);
            using var reader = new StreamReader(stream);
            var lines = new List<string>();
            while (reader.ReadLine() is { } line)
            {
                lines.Add(line);
            }
            return lines;
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            return [];
        }
    }
}
