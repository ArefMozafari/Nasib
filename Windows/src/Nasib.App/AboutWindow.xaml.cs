using System.Reflection;
using System.Windows;

namespace Nasib.App;

/// <summary>
/// Shows the icon, version and copyright, read from the assembly so they follow Directory.Build.props.
/// </summary>
public partial class AboutWindow : Window
{
    private const char BuildMetadataSeparator = '+';

    public AboutWindow()
    {
        InitializeComponent();
        var assembly = Assembly.GetExecutingAssembly();
        var version = assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? string.Empty;
        VersionText.Text = "Version " + version.Split(BuildMetadataSeparator)[0];
        CopyrightText.Text = assembly.GetCustomAttribute<AssemblyCopyrightAttribute>()?.Copyright ?? string.Empty;
    }

    private void OnOkClick(object sender, RoutedEventArgs eventArguments)
    {
        Close();
    }
}
