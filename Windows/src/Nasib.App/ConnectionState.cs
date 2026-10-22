using Nasib.App.Drawing;

namespace Nasib.App;

public enum ConnectionKind
{
    Unknown,
    Down,
    Searching,
    Connected,
}

/// <summary>
/// What the window and the tray show: checking, down, trying a pair, or connected to one.
/// </summary>
public sealed record ConnectionState(ConnectionKind Kind, string? Location = null, string? Protocol = null)
{
    private const string PairSeparator = " / ";

    public static ConnectionState Unknown { get; } = new(ConnectionKind.Unknown);

    public static ConnectionState Down { get; } = new(ConnectionKind.Down);

    /// <summary>
    /// True when a tunnel shown as connected has gone down. A stopped search or a tunnel that was never up
    /// is not a drop, so it doesn't start a reconnect the user didn't ask for.
    /// </summary>
    public static bool TunnelDropped(ConnectionState previous, ConnectionState current)
    {
        return previous.Kind == ConnectionKind.Connected && current.Kind == ConnectionKind.Down;
    }

    public string Title => Kind switch
    {
        ConnectionKind.Unknown => "Checking…",
        ConnectionKind.Down => "Not connected",
        ConnectionKind.Searching => "Trying " + Pair,
        _ => Pair,
    };

    /// <summary>
    /// Unknown shows as searching until the first status read lands.
    /// </summary>
    public GlyphState Glyph => Kind switch
    {
        ConnectionKind.Connected => GlyphState.Connected,
        ConnectionKind.Down => GlyphState.Down,
        _ => GlyphState.Searching,
    };

    private string Pair => Protocol is null ? Location ?? string.Empty : Location + PairSeparator + Protocol;
}
