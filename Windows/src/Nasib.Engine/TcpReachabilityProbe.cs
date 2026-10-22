using System.Net;
using System.Net.Sockets;

namespace Nasib.Engine;

/// <summary>
/// Opens a TCP connection and closes it straight away; an address that accepts it is not blackholed.
/// </summary>
public sealed class TcpReachabilityProbe : IReachabilityProbe
{
    public async Task<bool> AnswersAsync(string address, int port, TimeSpan timeout, CancellationToken cancellationToken)
    {
        if (!IPAddress.TryParse(address, out var parsedAddress))
        {
            return false;
        }
        using var client = new TcpClient(parsedAddress.AddressFamily);
        using var timeoutSource = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeoutSource.CancelAfter(timeout);
        try
        {
            await client.ConnectAsync(parsedAddress, port, timeoutSource.Token);
            return true;
        }
        catch (SocketException)
        {
            return false;
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            return false;
        }
    }
}
