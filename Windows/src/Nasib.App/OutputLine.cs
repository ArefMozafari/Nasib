namespace Nasib.App;

/// <summary>
/// One line of engine output as the Status tab shows it: its text, its color, and whether it gets the
/// alternating background.
/// </summary>
public sealed record OutputLine(string Text, OutputTone Tone, bool IsStriped);
