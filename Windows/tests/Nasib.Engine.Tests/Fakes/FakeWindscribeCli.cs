namespace Nasib.Engine.Tests.Fakes;

internal enum FakeTunnelState
{
    Disconnected,
    Connected,
    Error,
}

/// <summary>
/// Mimics the real Windscribe app: a connect issued while a tunnel is up tears the old tunnel down and drops
/// the new request, leaving nothing connected, and a failed connect can leave a sticky "Error: 11" behind.
/// </summary>
internal sealed class FakeWindscribeCli : IWindscribeCli
{
    public const string ErrorCode = "11";

    public FakeTunnelState State { get; set; } = FakeTunnelState.Disconnected;

    public string ConnectedLocation { get; set; } = "Dubai - Khalifa";

    /// <summary>
    /// Decides whether a connect to the pair brings the tunnel up.
    /// </summary>
    public Func<string, string, bool> PairConnects { get; set; } = (_, _) => true;

    /// <summary>
    /// When true, a failed connect leaves the error state, and a disconnect does not clear it.
    /// </summary>
    public bool ErrorIsSticky { get; set; }

    /// <summary>
    /// Called before each connect, so a test can cancel a search part way through.
    /// </summary>
    public Action<string, string>? BeforeConnect { get; set; }

    public List<(string Nickname, string Protocol)> ConnectCalls { get; } = [];

    public int DisconnectCalls { get; private set; }

    public Task<string> ReadStatusAsync(CancellationToken cancellationToken)
    {
        var output = State switch
        {
            FakeTunnelState.Connected => $"Connect state: Connected: {ConnectedLocation}\nProtocol: IKEv2:500\n",
            FakeTunnelState.Error => $"Connect state: Error: {ErrorCode}\n",
            _ => "Connect state: Disconnected\n",
        };
        return Task.FromResult(output);
    }

    public Task ConnectAsync(string nickname, string protocol, CancellationToken cancellationToken)
    {
        BeforeConnect?.Invoke(nickname, protocol);
        ConnectCalls.Add((nickname, protocol));
        if (State == FakeTunnelState.Connected)
        {
            State = FakeTunnelState.Disconnected;
        }
        else if (PairConnects(nickname, protocol))
        {
            State = FakeTunnelState.Connected;
            ConnectedLocation = nickname;
        }
        else
        {
            State = ErrorIsSticky ? FakeTunnelState.Error : FakeTunnelState.Disconnected;
        }
        return Task.CompletedTask;
    }

    public Task DisconnectAsync(CancellationToken cancellationToken)
    {
        DisconnectCalls++;
        if (!(State == FakeTunnelState.Error && ErrorIsSticky))
        {
            State = FakeTunnelState.Disconnected;
        }
        return Task.CompletedTask;
    }
}
