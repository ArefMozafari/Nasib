using System.Net;
using System.Net.Sockets;

namespace Nasib.Engine.Tests;

public class TcpReachabilityProbeTests
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(2);

    public class AnswersAsync
    {
        [Fact(DisplayName = "should answer when something listens on the port")]
        public async Task ShouldAnswerWhenSomethingListensOnThePort()
        {
            var listener = new TcpListener(IPAddress.Loopback, 0);
            listener.Start();
            try
            {
                var port = ((IPEndPoint)listener.LocalEndpoint).Port;

                Assert.True(await new TcpReachabilityProbe().AnswersAsync("127.0.0.1", port, Timeout, CancellationToken.None));
            }
            finally
            {
                listener.Stop();
            }
        }

        [Fact(DisplayName = "should not answer when nothing listens on the port")]
        public async Task ShouldNotAnswerWhenNothingListensOnThePort()
        {
            var listener = new TcpListener(IPAddress.Loopback, 0);
            listener.Start();
            var port = ((IPEndPoint)listener.LocalEndpoint).Port;
            listener.Stop();

            Assert.False(await new TcpReachabilityProbe().AnswersAsync("127.0.0.1", port, Timeout, CancellationToken.None));
        }

        [Fact(DisplayName = "should not answer for something that is not an address")]
        public async Task ShouldNotAnswerForSomethingThatIsNotAnAddress()
        {
            Assert.False(await new TcpReachabilityProbe().AnswersAsync("Stolen Cheese", ReachabilityFilter.ReachPort, Timeout, CancellationToken.None));
        }
    }
}
