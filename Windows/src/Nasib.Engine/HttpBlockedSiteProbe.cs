using System.Net;

namespace Nasib.Engine;

/// <summary>
/// Requests a page that answers 204 No Content and counts only that answer. Windscribe reporting Connected
/// is not enough on a censored network; this is what proves traffic gets through.
/// </summary>
public sealed class HttpBlockedSiteProbe : IBlockedSiteProbe
{
    public static Uri DefaultAddress { get; } = new("https://www.youtube.com/generate_204");

    private readonly Uri address;
    private readonly TimeSpan timeout;

    public HttpBlockedSiteProbe(Uri address, TimeSpan timeout)
    {
        this.address = address;
        this.timeout = timeout;
    }

    public async Task<bool> LoadsAsync(CancellationToken cancellationToken)
    {
        // A fresh client per probe keeps a connection opened through the previous tunnel from being reused,
        // and the proxy stays off so a system proxy such as v2rayN cannot answer in the tunnel's place.
        using var handler = new SocketsHttpHandler { UseProxy = false };
        using var client = new HttpClient(handler) { Timeout = timeout };
        try
        {
            using var response = await client.GetAsync(address, HttpCompletionOption.ResponseHeadersRead, cancellationToken);
            return response.StatusCode == HttpStatusCode.NoContent;
        }
        catch (HttpRequestException)
        {
            return false;
        }
        catch (TaskCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            return false;
        }
    }
}
