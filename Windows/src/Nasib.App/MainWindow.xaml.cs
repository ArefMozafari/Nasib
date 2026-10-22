using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;

namespace Nasib.App;

/// <summary>
/// The Status and Settings tabs. Closing the window hides it to the tray; only Quit Nasib exits.
/// </summary>
public partial class MainWindow : Window
{
    private const int StatusTabIndex = 0;

    private bool closeAllowed;

    public MainWindow(EngineController controller, SettingsViewModel settings)
    {
        InitializeComponent();
        StatusPanel.DataContext = controller;
        SettingsPanel.DataContext = settings;
    }

    public void ShowAndActivate()
    {
        Show();
        if (WindowState == WindowState.Minimized)
        {
            WindowState = WindowState.Normal;
        }
        Activate();
    }

    /// <summary>
    /// Lets the next close through, for Quit Nasib.
    /// </summary>
    public void AllowClose()
    {
        closeAllowed = true;
    }

    protected override void OnClosing(CancelEventArgs eventArguments)
    {
        if (!closeAllowed)
        {
            eventArguments.Cancel = true;
            Hide();
        }
        base.OnClosing(eventArguments);
    }

    private void OnTabSelectionChanged(object sender, SelectionChangedEventArgs eventArguments)
    {
        if (!ReferenceEquals(eventArguments.OriginalSource, Tabs) || StatusPanel is null)
        {
            return;
        }
        var statusSelected = Tabs.SelectedIndex == StatusTabIndex;
        StatusPanel.Visibility = statusSelected ? Visibility.Visible : Visibility.Hidden;
        SettingsPanel.Visibility = statusSelected ? Visibility.Hidden : Visibility.Visible;
    }
}
