namespace Nasib.App.Tests;

public class ConnectionStateTests
{
    private static readonly ConnectionState Connected = new(ConnectionKind.Connected, "Zurich - Altstadt", "IKEv2");
    private static readonly ConnectionState Searching = new(ConnectionKind.Searching, "Altstadt", "ikev2");

    public class TunnelDropped
    {
        [Fact(DisplayName = "should detect a drop when a connected tunnel goes down")]
        public void ShouldDetectADropWhenAConnectedTunnelGoesDown()
        {
            Assert.True(ConnectionState.TunnelDropped(Connected, ConnectionState.Down));
        }

        [Fact(DisplayName = "should not detect a drop while still connected")]
        public void ShouldNotDetectADropWhileStillConnected()
        {
            Assert.False(ConnectionState.TunnelDropped(Connected, Connected));
        }

        [Fact(DisplayName = "should not detect a drop when the tunnel was never connected")]
        public void ShouldNotDetectADropWhenTheTunnelWasNeverConnected()
        {
            Assert.False(ConnectionState.TunnelDropped(ConnectionState.Unknown, ConnectionState.Down));
            Assert.False(ConnectionState.TunnelDropped(ConnectionState.Down, ConnectionState.Down));
        }

        [Fact(DisplayName = "should not detect a drop after a stopped search")]
        public void ShouldNotDetectADropAfterAStoppedSearch()
        {
            Assert.False(ConnectionState.TunnelDropped(Searching, ConnectionState.Down));
        }
    }
}
