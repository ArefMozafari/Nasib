namespace Nasib.Engine;

/// <summary>
/// The three windscribe-cli commands the engine uses. Tests replace it with a fake.
/// </summary>
public interface IWindscribeCli
{
    Task<string> ReadStatusAsync(CancellationToken cancellationToken);

    /// <summary>
    /// Starts a connect and returns at once; the caller polls the status to see how it went.
    /// </summary>
    Task ConnectAsync(string nickname, string protocol, CancellationToken cancellationToken);

    Task DisconnectAsync(CancellationToken cancellationToken);
}
