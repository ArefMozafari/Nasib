using Nasib.Engine;

namespace Nasib.App;

/// <summary>
/// What a line in the output pane reports, which decides its color.
/// </summary>
public enum OutputKind
{
    Trying,
    Connected,
    Failed,
    TunnelDown,
    NothingWorked,
    Stopped,
    Other,
}

/// <summary>
/// One line of engine output, as the Status tab shows it.
/// </summary>
public sealed record OutputLine(string Text, OutputKind Kind)
{
    /// <summary>
    /// A failed attempt is normal while searching, so only "nothing worked" is shown as an error.
    /// </summary>
    public static OutputLine From(EngineEvent engineEvent)
    {
        var kind = engineEvent switch
        {
            TryingEvent => OutputKind.Trying,
            ConnectedEvent => OutputKind.Connected,
            FailedEvent => OutputKind.Failed,
            TunnelDownEvent => OutputKind.TunnelDown,
            NothingWorkedEvent => OutputKind.NothingWorked,
            SearchStoppedEvent or SurveyStoppedEvent or SurveyFinishedEvent => OutputKind.Stopped,
            _ => OutputKind.Other,
        };
        return new OutputLine(engineEvent.Text, kind);
    }
}
