namespace Nasib.Engine;

/// <summary>
/// The "Connect state" line of windscribe-cli status.
/// </summary>
public enum ConnectState
{
    Unknown,
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

/// <summary>
/// What windscribe-cli status reports, for example:
/// Connect state: Connected: Amsterdam - Stolen Cheese
/// Protocol: IKEv2:500
/// </summary>
public sealed record TunnelStatus(ConnectState State, string? Location, string? Protocol, string? ErrorCode)
{
    private const string ConnectStatePrefix = "Connect state: ";
    private const string ProtocolPrefix = "Protocol: ";
    private const string ConnectedWord = "Connected";
    private const string DisconnectedWord = "Disconnected";
    private const string ConnectingWord = "Connecting";
    private const string DisconnectingWord = "Disconnecting";
    private const string ErrorWord = "Error";
    private const string DetailSeparator = ": ";
    private const char ProtocolPortSeparator = ':';

    public static TunnelStatus Unknown { get; } = new(ConnectState.Unknown, null, null, null);

    public bool IsConnected => State == ConnectState.Connected;

    /// <summary>
    /// True when no tunnel is up or on its way: Disconnected, or an error the CLI keeps reporting until the next connect.
    /// </summary>
    public bool IsDown => State is ConnectState.Disconnected or ConnectState.Error;

    public static TunnelStatus Parse(string output)
    {
        var state = ConnectState.Unknown;
        string? location = null;
        string? protocol = null;
        string? errorCode = null;
        foreach (var rawLine in output.Split('\n'))
        {
            var line = rawLine.Trim();
            if (line.StartsWith(ConnectStatePrefix, StringComparison.Ordinal))
            {
                (state, var detail) = ParseConnectState(line[ConnectStatePrefix.Length..]);
                if (state == ConnectState.Connected)
                {
                    location = detail;
                }
                else if (state == ConnectState.Error)
                {
                    errorCode = detail;
                }
            }
            else if (line.StartsWith(ProtocolPrefix, StringComparison.Ordinal))
            {
                protocol = line[ProtocolPrefix.Length..].Split(ProtocolPortSeparator)[0];
            }
        }
        return new TunnelStatus(state, location, protocol, errorCode);
    }

    /// <summary>
    /// Splits "Connected: Amsterdam - Stolen Cheese" or "Error: 11" into the state and the text after it.
    /// </summary>
    private static (ConnectState State, string? Detail) ParseConnectState(string value)
    {
        var separatorIndex = value.IndexOf(DetailSeparator, StringComparison.Ordinal);
        var word = separatorIndex < 0 ? value : value[..separatorIndex];
        var detail = separatorIndex < 0 ? null : value[(separatorIndex + DetailSeparator.Length)..];
        var state = word switch
        {
            ConnectedWord => ConnectState.Connected,
            DisconnectedWord => ConnectState.Disconnected,
            ConnectingWord => ConnectState.Connecting,
            DisconnectingWord => ConnectState.Disconnecting,
            ErrorWord => ConnectState.Error,
            _ => ConnectState.Unknown,
        };
        return (state, detail);
    }
}
