namespace Nasib.Engine.Tests;

public class TunnelStatusTests
{
    public class Parse
    {
        [Fact(DisplayName = "should parse a connected status with the protocol port stripped")]
        public void ShouldParseAConnectedStatusWithTheProtocolPortStripped()
        {
            const string output = """
                Internet connectivity: available
                Login state: Logged in
                Connect state: Connected: Atlanta - Magic City
                Protocol: IKEv2:500
                VPN IP: 198.51.100.20
                """;

            var status = TunnelStatus.Parse(output);

            Assert.Equal(new TunnelStatus(ConnectState.Connected, "Atlanta - Magic City", "IKEv2", null), status);
            Assert.True(status.IsConnected);
            Assert.False(status.IsDown);
        }

        [Fact(DisplayName = "should parse a disconnected status")]
        public void ShouldParseADisconnectedStatus()
        {
            var status = TunnelStatus.Parse("Firewall state: Off\r\nConnect state: Disconnected\r\nPublic IP: 203.0.113.10\r\n");

            Assert.Equal(ConnectState.Disconnected, status.State);
            Assert.True(status.IsDown);
            Assert.False(status.IsConnected);
        }

        [Fact(DisplayName = "should count an error as down and not connected")]
        public void ShouldCountAnErrorAsDownAndNotConnected()
        {
            var status = TunnelStatus.Parse("Connect state: Error: 11\n");

            Assert.Equal(new TunnelStatus(ConnectState.Error, null, null, "11"), status);
            Assert.True(status.IsDown);
            Assert.False(status.IsConnected);
        }

        [Theory(DisplayName = "should treat transitional states as neither up nor down")]
        [InlineData("Connect state: Connecting", ConnectState.Connecting)]
        [InlineData("Connect state: Disconnecting", ConnectState.Disconnecting)]
        public void ShouldTreatTransitionalStatesAsNeitherUpNorDown(string output, ConnectState expected)
        {
            var status = TunnelStatus.Parse(output);

            Assert.Equal(expected, status.State);
            Assert.False(status.IsConnected);
            Assert.False(status.IsDown);
        }

        [Fact(DisplayName = "should report unknown for empty output")]
        public void ShouldReportUnknownForEmptyOutput()
        {
            var status = TunnelStatus.Parse(string.Empty);

            Assert.Equal(TunnelStatus.Unknown, status);
            Assert.False(status.IsDown);
        }
    }
}
