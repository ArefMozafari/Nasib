using System;
using System.Collections.Specialized;
using System.Security;
using System.Windows;
using System.Windows.Controls;

namespace Nasib.App;

/// <summary>
/// The Status tab: the current server, the actions, the two toggles and the engine's recent output.
/// </summary>
public partial class StatusPanel : UserControl
{
    // Set while the checkbox is being made to match the registry, so that change is not written back.
    private bool isSyncingLaunchAtLogin;

    public StatusPanel()
    {
        InitializeComponent();
        SyncLaunchAtLogin();
        DataContextChanged += OnDataContextChanged;
    }

    private EngineController Controller => (EngineController)DataContext;

    private void OnDataContextChanged(object sender, DependencyPropertyChangedEventArgs eventArguments)
    {
        if (eventArguments.NewValue is EngineController controller)
        {
            controller.RecentLines.CollectionChanged += OnRecentLinesChanged;
        }
    }

    /// <summary>
    /// Keeps the newest line in view.
    /// </summary>
    private void OnRecentLinesChanged(object? sender, NotifyCollectionChangedEventArgs eventArguments)
    {
        OutputScroller.ScrollToEnd();
    }

    private void OnFindNowClick(object sender, RoutedEventArgs eventArguments)
    {
        Controller.FindNow();
    }

    private void OnSurveyClick(object sender, RoutedEventArgs eventArguments)
    {
        Controller.ToggleSurvey();
    }

    private void OnShowLogClick(object sender, RoutedEventArgs eventArguments)
    {
        Controller.ShowLog();
    }

    /// <summary>
    /// Handles Checked and Unchecked rather than Click, so keyboard and screen-reader toggles apply too.
    /// </summary>
    private void OnLaunchAtLoginChanged(object sender, RoutedEventArgs eventArguments)
    {
        if (isSyncingLaunchAtLogin)
        {
            return;
        }
        try
        {
            LoginItem.SetEnabled(LaunchAtLogin.IsChecked == true);
            LoginItemError.Visibility = Visibility.Collapsed;
        }
        catch (Exception exception) when (exception is UnauthorizedAccessException or SecurityException or InvalidOperationException)
        {
            LoginItemError.Text = exception.Message;
            LoginItemError.Visibility = Visibility.Visible;
        }
        SyncLaunchAtLogin();
    }

    private void SyncLaunchAtLogin()
    {
        isSyncingLaunchAtLogin = true;
        LaunchAtLogin.IsChecked = LoginItem.IsEnabled;
        isSyncingLaunchAtLogin = false;
    }
}
