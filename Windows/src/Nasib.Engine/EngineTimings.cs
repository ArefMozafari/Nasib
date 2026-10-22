namespace Nasib.Engine;

/// <summary>
/// How long the engine waits at each step. The defaults match Engine/ws-find.sh on macOS.
/// </summary>
public sealed record EngineTimings
{
    public static EngineTimings Default { get; } = new();

    /// <summary>
    /// How long the blocked-site request may take before the tunnel counts as not working.
    /// </summary>
    public TimeSpan ProbeTimeout { get; init; } = TimeSpan.FromSeconds(8);

    /// <summary>
    /// The CLI itself waits about 42 seconds per failed attempt; a working pair answers within this.
    /// </summary>
    public TimeSpan ConnectTimeout { get; init; } = TimeSpan.FromSeconds(10);

    /// <summary>
    /// The CLI can report Connected and drop seconds later, so the engine waits this long before trusting it.
    /// </summary>
    public TimeSpan SettleDelay { get; init; } = TimeSpan.FromSeconds(2);

    /// <summary>
    /// The longest the engine waits for a tunnel to finish tearing down.
    /// </summary>
    public TimeSpan DisconnectTimeout { get; init; } = TimeSpan.FromSeconds(15);

    /// <summary>
    /// How often the engine reads the CLI status while it waits for a state change.
    /// </summary>
    public TimeSpan StatusPollInterval { get; init; } = TimeSpan.FromSeconds(1);

    /// <summary>
    /// How long one TCP knock on a datacenter address may take.
    /// </summary>
    public TimeSpan ReachTimeout { get; init; } = TimeSpan.FromSeconds(2);
}
