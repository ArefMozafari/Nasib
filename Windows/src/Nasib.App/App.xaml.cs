using System;
using System.Linq;
using System.Windows;

namespace Nasib.App;

/// <summary>
/// Starts Nasib in the tray, wires the engine to the window and the tray, and keeps it to one instance.
/// </summary>
public partial class App : Application
{
    /// <summary>
    /// The Run key starts Nasib with this argument, so signing in does not open the window.
    /// </summary>
    public const string BackgroundArgument = "--background";

    private SingleInstance? singleInstance;
    private EngineController? controller;
    private UpdateChecker? updateChecker;
    private TrayIcon? trayIcon;
    private MainWindow? mainWindow;
    private AboutWindow? aboutWindow;

    protected override void OnStartup(StartupEventArgs eventArguments)
    {
        base.OnStartup(eventArguments);
        singleInstance = SingleInstance.TryClaim(() => Dispatcher.BeginInvoke(ShowMainWindow));
        if (singleInstance is null)
        {
            SingleInstance.AskRunningInstanceToShowItsWindow();
            Shutdown();
            return;
        }

        var paths = StatePaths.ForCurrentUser();
        var preferences = Preferences.Load(paths);
        controller = new EngineController(EngineProcess.DefaultExecutablePath, preferences, paths);
        updateChecker = new UpdateChecker(preferences);
        mainWindow = new MainWindow(controller, updateChecker, new SettingsViewModel(paths, controller.ReadCatalogAsync));
        trayIcon = new TrayIcon(controller, updateChecker, mainWindow, ShowMainWindow, ShowAbout, Quit);
        controller.NotificationRequested += trayIcon.ShowNotification;
        updateChecker.NotificationRequested += trayIcon.ShowNotification;
        controller.Start();
        updateChecker.Start();

        if (!eventArguments.Args.Contains(BackgroundArgument))
        {
            ShowMainWindow();
        }
    }

    protected override void OnExit(ExitEventArgs eventArguments)
    {
        trayIcon?.Dispose();
        updateChecker?.Dispose();
        controller?.Dispose();
        singleInstance?.Dispose();
        base.OnExit(eventArguments);
    }

    private void ShowMainWindow()
    {
        mainWindow?.ShowAndActivate();
    }

    private void ShowAbout()
    {
        if (aboutWindow is null)
        {
            aboutWindow = new AboutWindow();
            aboutWindow.Closed += (_, _) => aboutWindow = null;
        }
        aboutWindow.Show();
        aboutWindow.Activate();
    }

    private void Quit()
    {
        mainWindow?.AllowClose();
        Shutdown();
    }
}
