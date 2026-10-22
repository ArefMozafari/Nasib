namespace Nasib.App;

/// <summary>
/// One protocol in the Settings list: its name and whether the engine should try it.
/// </summary>
public sealed class ProtocolRow : ObservableObject
{
    private bool enabled;

    public ProtocolRow(string name, bool enabled)
    {
        Name = name;
        this.enabled = enabled;
    }

    public string Name { get; }

    public bool Enabled
    {
        get => enabled;
        set => SetProperty(ref enabled, value);
    }
}
