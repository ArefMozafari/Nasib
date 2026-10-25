using System;

namespace Nasib.App;

/// <summary>
/// How an output line is colored. A failed attempt is routine while searching, so only "nothing worked"
/// and a missing CLI get the error tone.
/// </summary>
public enum OutputTone
{
    Primary,
    Success,
    Muted,
    Warning,
    Error,
    Accent,
    Detail,

    /// <summary>
    /// The header that starts each stage of a search.
    /// </summary>
    Phase,
}

/// <summary>
/// One line of the engine's output, turned into something the app can react to.
/// </summary>
public abstract record EngineEvent
{
    public abstract OutputTone Tone { get; }
}

public sealed record TryingEvent(string Location, string Protocol) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Primary;
}

public sealed record ConnectedEvent(string Location, string Protocol) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Success;
}

public sealed record FailedEvent(string Location, string Protocol) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Muted;
}

public sealed record NothingWorkedEvent : EngineEvent
{
    public override OutputTone Tone => OutputTone.Error;
}

public sealed record SurveyFinishedEvent : EngineEvent
{
    public override OutputTone Tone => OutputTone.Accent;
}

public sealed record SurveyStoppedEvent : EngineEvent
{
    public override OutputTone Tone => OutputTone.Accent;
}

public sealed record SearchStoppedEvent : EngineEvent
{
    public override OutputTone Tone => OutputTone.Accent;
}

public sealed record TunnelDownEvent : EngineEvent
{
    public override OutputTone Tone => OutputTone.Warning;
}

public sealed record CliMissingEvent(string Path) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Error;
}

public sealed record PhaseEvent(string Stage) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Phase;
}

public sealed record OtherEvent(string Text) : EngineEvent
{
    public override OutputTone Tone => OutputTone.Detail;
}

/// <summary>
/// What `nasib-engine --status` reports: the CLI's own location and protocol names.
/// </summary>
public sealed record TunnelStatus(string Location, string Protocol);

/// <summary>
/// Reads the lines the engine prints, as listed under "Output contract" in Engine/README.md.
/// </summary>
public static class EngineOutputParser
{
    private const string TryingPrefix = "trying: ";
    private const string ConnectedPrefix = "connected: ";
    private const string FailedPrefix = "failed: ";
    private const string NothingWorkedLine = "nothing worked";
    private const string SurveyFinishedLine = "survey finished";
    private const string SurveyStoppedLine = "survey stopped";
    private const string SearchStoppedLine = "search stopped";
    private const string TunnelDownLine = "tunnel down";
    private const string CliMissingPrefix = "windscribe-cli missing: ";
    private const string PhasePrefix = "phase: ";
    private const string PairSeparator = " / ";

    /// <summary>
    /// Splits "Location / protocol" at the last separator, since a nickname may contain one itself.
    /// </summary>
    private static (string Location, string Protocol)? SplitPair(string text)
    {
        var separatorIndex = text.LastIndexOf(PairSeparator, StringComparison.Ordinal);
        if (separatorIndex < 0)
        {
            return null;
        }
        return (text[..separatorIndex], text[(separatorIndex + PairSeparator.Length)..]);
    }

    /// <summary>
    /// Returns the pair after the prefix, or null when the line doesn't start with it or has no pair.
    /// </summary>
    private static (string Location, string Protocol)? PairAfter(string prefix, string line)
    {
        return line.StartsWith(prefix, StringComparison.Ordinal) ? SplitPair(line[prefix.Length..]) : null;
    }

    public static EngineEvent Parse(string line)
    {
        if (PairAfter(TryingPrefix, line) is { } trying)
        {
            return new TryingEvent(trying.Location, trying.Protocol);
        }
        if (PairAfter(ConnectedPrefix, line) is { } connected)
        {
            return new ConnectedEvent(connected.Location, connected.Protocol);
        }
        if (PairAfter(FailedPrefix, line) is { } failed)
        {
            return new FailedEvent(failed.Location, failed.Protocol);
        }
        return line switch
        {
            NothingWorkedLine => new NothingWorkedEvent(),
            SurveyFinishedLine => new SurveyFinishedEvent(),
            SurveyStoppedLine => new SurveyStoppedEvent(),
            SearchStoppedLine => new SearchStoppedEvent(),
            TunnelDownLine => new TunnelDownEvent(),
            _ when line.StartsWith(CliMissingPrefix, StringComparison.Ordinal) => new CliMissingEvent(line[CliMissingPrefix.Length..]),
            _ when line.StartsWith(PhasePrefix, StringComparison.Ordinal) => new PhaseEvent(line[PhasePrefix.Length..]),
            _ => new OtherEvent(line),
        };
    }

    /// <summary>
    /// Reads the one line `nasib-engine --status` prints. Returns null for `down`, a missing CLI, or
    /// anything else that isn't a fully connected tunnel.
    /// </summary>
    public static TunnelStatus? ParseStatus(string line)
    {
        return PairAfter(ConnectedPrefix, line) is { } pair ? new TunnelStatus(pair.Location, pair.Protocol) : null;
    }
}
