namespace Nasib.Engine;

/// <summary>
/// Something the engine did, with the line the macOS engine prints for it.
/// </summary>
public abstract record EngineEvent
{
    private const string PairSeparator = " / ";

    public abstract string Text { get; }

    protected static string Pair(string nickname, string protocol) => nickname + PairSeparator + protocol;
}

/// <summary>
/// An attempt on one datacenter and protocol has started.
/// </summary>
public sealed record TryingEvent(string Nickname, string Protocol) : EngineEvent
{
    public override string Text => "trying: " + Pair(Nickname, Protocol);
}

/// <summary>
/// A blocked site loaded through the tunnel.
/// </summary>
public sealed record ConnectedEvent(string Nickname, string Protocol) : EngineEvent
{
    public override string Text => "connected: " + Pair(Nickname, Protocol);
}

/// <summary>
/// A search tried every pair and none worked.
/// </summary>
public sealed record NothingWorkedEvent : EngineEvent
{
    public override string Text => "nothing worked";
}

/// <summary>
/// Watch found the tunnel down or not carrying traffic, and is about to search.
/// </summary>
public sealed record TunnelDownEvent : EngineEvent
{
    public override string Text => "tunnel down";
}

/// <summary>
/// A survey tried every pair.
/// </summary>
public sealed record SurveyFinishedEvent : EngineEvent
{
    public override string Text => "survey finished";
}

/// <summary>
/// A survey was cancelled before it finished.
/// </summary>
public sealed record SurveyStoppedEvent : EngineEvent
{
    public override string Text => "survey stopped";
}

/// <summary>
/// Any other progress line, such as the order the search will try things in.
/// </summary>
public sealed record MessageEvent(string Message) : EngineEvent
{
    public override string Text => Message;
}
