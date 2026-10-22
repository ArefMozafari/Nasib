using System.Windows;
using System.Windows.Controls;

namespace Nasib.App;

/// <summary>
/// The Settings tab: the cities to search and the protocols to try, in order.
/// </summary>
public partial class SettingsPanel : UserControl
{
    private const int UpOffset = -1;
    private const int DownOffset = 1;

    public SettingsPanel()
    {
        InitializeComponent();
    }

    private SettingsViewModel Settings => (SettingsViewModel)DataContext;

    private void OnMoveUpClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.Move((ProtocolRow)((Button)sender).Tag, UpOffset);
    }

    private void OnMoveDownClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.Move((ProtocolRow)((Button)sender).Tag, DownOffset);
    }

    private void OnResetClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.ResetToDefaults();
    }

    private void OnSaveClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.Save();
    }
}
