using System;
using System.Linq;
using System.Windows;
using Nasib.Engine;

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
        var timings = EngineTimings.Default;
        var cli = new WindscribeCli(WindscribeCli.DefaultExecutablePath());
        var tunnel = new TunnelController(
            cli,
            new HttpBlockedSiteProbe(HttpBlockedSiteProbe.DefaultAddress, timings.ProbeTimeout),
            new TaskWaiter(),
            timings);
        var finder = new ServerFinder(
            paths,
            new DatacenterCatalog(SeedDatacenters.Load(), ClientLogParser.DefaultLogDirectory()),
            tunnel,
            new ReachabilityFilter(new TcpReachabilityProbe(), timings),
            new AttemptLog(paths, TimeProvider.System));

        var preferences = Preferences.Load(paths);
        controller = new EngineController(finder, tunnel, preferences, paths, cli.ExecutablePath, cli.IsInstalled);
        updateChecker = new UpdateChecker(preferences);
        mainWindow = new MainWindow(controller, updateChecker, new SettingsViewModel(paths));
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
