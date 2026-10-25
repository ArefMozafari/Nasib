namespace Nasib.App;

/// <summary>
/// One short sentence for every button and toggle, worded the same as on macOS.
/// </summary>
public static class ToolTipText
{
    public const string FindNow = "Search for a working server and connect to the first one that loads a blocked site.";
    public const string StopFind = "Stop searching and disconnect.";
    public const string StartSurvey = "Try every server on every protocol and port and record the results, without stopping at the first success.";
    public const string StopSurvey = "Stop the survey and reconnect to the last server that worked.";
    public const string ShowLog = "Open the list of every connection attempt.";
    public const string AutoReconnect = "Check the tunnel every 3 seconds and a blocked site every minute, and search for a new server when either fails.";
    public const string LaunchAtLogin = "Start Nasib in the background when you sign in.";
    public const string Save = "Use these locations and protocols from the next search on.";
    public const string ResetToDefaults = "Restore the default locations, ranking and protocol order. Nothing changes until you save.";
    public const string RankBySuccess = "Try the checked locations in the order that worked best lately, instead of exactly your order.";
    public const string MoveProtocolUp = "Try this protocol earlier.";
    public const string MoveProtocolDown = "Try this protocol later.";
    public const string MoveLocationUp = "Try this location earlier.";
    public const string MoveLocationDown = "Try this location later.";
    public const string UpdateToRelease = "Open the release page to download the new version.";
}
