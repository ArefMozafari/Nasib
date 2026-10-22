using System.Net;
using System.Net.Sockets;
using System.Text;

namespace Nasib.Engine.Tests;

public class HttpBlockedSiteProbeTests
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(2);

    /// <summary>
    /// Answers one HTTP request on the loopback address with the given status line, or never answers.
    /// </summary>
    private sealed class OneShotServer : IDisposable
    {
        private readonly TcpListener listener = new(IPAddress.Loopback, 0);
        private readonly Task serving;

        public OneShotServer(string? statusLine)
        {
            listener.Start();
            Address = new Uri($"http://127.0.0.1:{((IPEndPoint)listener.LocalEndpoint).Port}/generate_204");
            serving = ServeAsync(statusLine);
        }

        public Uri Address { get; }

        private async Task ServeAsync(string? statusLine)
        {
            try
            {
                using var client = await listener.AcceptTcpClientAsync();
                using var stream = client.GetStream();
                using var reader = new StreamReader(stream, Encoding.ASCII, leaveOpen: true);

                // The request ends at the first empty line, since a GET carries no body.
                while (!string.IsNullOrEmpty(await reader.ReadLineAsync()))
                {
                }
                if (statusLine is null)
                {
                    await Task.Delay(Timeout * 2);
                    return;
                }
                var response = $"HTTP/1.1 {statusLine}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                await stream.WriteAsync(Encoding.ASCII.GetBytes(response));
            }
            catch (Exception exception) when (exception is SocketException or ObjectDisposedException or IOException)
            {
                // The listener stops when the test ends; nothing is left to serve.
            }
        }

        public void Dispose()
        {
            listener.Stop();
            serving.Wait(Timeout * 3);
        }
    }

    public class LoadsAsync
    {
        [Fact(DisplayName = "should count a 204 answer as loaded")]
        public async Task ShouldCountA204AnswerAsLoaded()
        {
            using var server = new OneShotServer("204 No Content");

            Assert.True(await new HttpBlockedSiteProbe(server.Address, Timeout).LoadsAsync(CancellationToken.None));
        }

        [Fact(DisplayName = "should not count any other answer, such as a block page")]
        public async Task ShouldNotCountAnyOtherAnswerSuchAsABlockPage()
        {
            using var server = new OneShotServer("200 OK");

            Assert.False(await new HttpBlockedSiteProbe(server.Address, Timeout).LoadsAsync(CancellationToken.None));
        }

        [Fact(DisplayName = "should not count a request that times out")]
        public async Task ShouldNotCountARequestThatTimesOut()
        {
            using var server = new OneShotServer(statusLine: null);

            Assert.False(await new HttpBlockedSiteProbe(server.Address, TimeSpan.FromMilliseconds(300)).LoadsAsync(CancellationToken.None));
        }
    }
}
