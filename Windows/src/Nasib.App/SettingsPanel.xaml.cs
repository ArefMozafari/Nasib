using System.Windows;
using System.Windows.Controls;
using System.Windows.Documents;

namespace Nasib.App;

/// <summary>
/// The Settings tab: the locations to search and the protocols to try, each in order.
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

    private void OnMoveProtocolUpClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.MoveProtocol((ProtocolRow)((Button)sender).Tag, UpOffset);
    }

    private void OnMoveProtocolDownClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.MoveProtocol((ProtocolRow)((Button)sender).Tag, DownOffset);
    }

    private void OnSelectAllProtocolsClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.SetAllProtocols(enabled: true);
    }

    private void OnDeselectAllProtocolsClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.SetAllProtocols(enabled: false);
    }

    private void OnMoveLocationUpClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.MoveLocation((LocationRow)((Button)sender).Tag, UpOffset);
    }

    private void OnMoveLocationDownClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.MoveLocation((LocationRow)((Button)sender).Tag, DownOffset);
    }

    private void OnDeselectShownOrderClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.DeselectShownOrder();
    }

    private void OnSelectShownCatalogClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.SetShownCatalog(selected: true);
    }

    private void OnDeselectShownCatalogClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.SetShownCatalog(selected: false);
    }

    private void OnToggleRegionClick(object sender, RoutedEventArgs eventArguments)
    {
        Settings.ToggleRegion((RegionGroup)((Hyperlink)sender).Tag);
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
