namespace Nasib.Engine;

/// <summary>
/// Checks whether a site that is blocked without the tunnel loads through it. Tests replace it with a fake.
/// </summary>
public interface IBlockedSiteProbe
{
    Task<bool> LoadsAsync(CancellationToken cancellationToken);
}
