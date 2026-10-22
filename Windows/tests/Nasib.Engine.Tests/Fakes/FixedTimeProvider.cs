namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// A clock stopped at one moment, in UTC, so log timestamps are predictable.
/// </summary>
internal sealed class FixedTimeProvider : TimeProvider
{
    private readonly DateTimeOffset now;

    public FixedTimeProvider(DateTimeOffset now)
    {
        this.now = now;
    }

    public override TimeZoneInfo LocalTimeZone => TimeZoneInfo.Utc;

    public override DateTimeOffset GetUtcNow() => now;
}
