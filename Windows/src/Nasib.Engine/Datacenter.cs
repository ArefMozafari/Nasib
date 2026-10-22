namespace Nasib.Engine;

/// <summary>
/// One Windscribe datacenter: its city, the nickname the CLI connects by, and every address seen for it.
/// </summary>
public sealed record Datacenter(string City, string Nickname, IReadOnlyList<string> Addresses)
{
    private const string LocationNameSeparator = " - ";

    /// <summary>
    /// The name Windscribe shows, such as "Amsterdam - Stolen Cheese".
    /// </summary>
    public string LocationName => City + LocationNameSeparator + Nickname;

    /// <summary>
    /// Splits "City - Nickname" at the first separator; null when there is none.
    /// </summary>
    public static Datacenter? FromLocationName(string locationName, IEnumerable<string> addresses)
    {
        var separatorIndex = locationName.IndexOf(LocationNameSeparator, StringComparison.Ordinal);
        if (separatorIndex <= 0)
        {
            return null;
        }
        var city = locationName[..separatorIndex];
        var nickname = locationName[(separatorIndex + LocationNameSeparator.Length)..];
        return nickname.Length == 0 ? null : new Datacenter(city, nickname, addresses.ToList());
    }

    /// <summary>
    /// Returns this datacenter with the given addresses added after its own, without duplicates.
    /// </summary>
    public Datacenter WithAddresses(IEnumerable<string> addresses)
    {
        return this with { Addresses = Addresses.Concat(addresses).Distinct(StringComparer.Ordinal).ToList() };
    }
}
