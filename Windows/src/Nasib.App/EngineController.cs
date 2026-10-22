using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Threading;
using Nasib.Engine;

namespace Nasib.App;

/// <summary>
/// Runs the engine off the UI thread and mirrors its progress for the window and the tray.
/// Every search can be stopped: a find or watch stops and disconnects, a survey stops and reconnects.
/// </summary>
public sealed class EngineController : ObservableObject, IDisposable
{
    private const int RecentLineLimit = 200;
    private static readonly TimeSpan WatchInterval = TimeSpan.FromSeconds(60);

    private const string FindNowTitle = "Find Now";
    private const string StopFindTitle = "Stop";
    private const string StopFindMenuTitle = "Stop Search";
    private const string StartSurveyTitle = "Start Survey";
    private const string StopSurveyTitle = "Stop Survey";

    private const string ConnectedNotificationTitle = "Connected";
    private const string NothingWorkedNotificationTitle = "No working server";
    private const string NothingWorkedNotificationBody = "Every datacenter and protocol failed.";
    private const string TunnelDownNotificationTitle = "Tunnel down";
    private const string TunnelDownNotificationBody = "Searching for a working server…";

    private readonly ServerFinder finder;
    private readonly TunnelController tunnel;
    private readonly Preferences preferences;
    private readonly StatePaths paths;
    private readonly Dispatcher dispatcher;
    private readonly DispatcherTimer watchTimer;

    private ConnectionState state = ConnectionState.Unknown;
    private bool isFinding;
    private bool isSurveying;
    private bool isCheckingTunnel;
    private CancellationTokenSource? searchCancellation;

    public EngineController(ServerFinder finder, TunnelController tunnel, Preferences preferences, StatePaths paths, string cliPath, bool cliInstalled)
    {
        this.finder = finder;
        this.tunnel = tunnel;
        this.preferences = preferences;
        this.paths = paths;
        CliPath = cliPath;
        CliMissing = !cliInstalled;
        dispatcher = Dispatcher.CurrentDispatcher;
        finder.EventRaised += engineEvent => dispatcher.BeginInvoke(() => Handle(engineEvent));
        watchTimer = new DispatcherTimer { Interval = WatchInterval };
        watchTimer.Tick += (_, _) => _ = WatchTickAsync();
    }

    /// <summary>
    /// Raised on the UI thread with a title and a body.
    /// </summary>
    public event Action<string, string>? NotificationRequested;

    public ConnectionState State
    {
        get => state;
        private set => SetProperty(ref state, value);
    }

    /// <summary>
    /// True while a find runs, whether from Find Now or from auto-reconnect.
    /// </summary>
    public bool IsFinding
    {
        get => isFinding;
        private set
        {
            if (SetProperty(ref isFinding, value))
            {
                OnSearchStateChanged();
            }
        }
    }

    public bool IsSurveying
    {
        get => isSurveying;
        private set
        {
            if (SetProperty(ref isSurveying, value))
            {
                OnSearchStateChanged();
            }
        }
    }

    public bool IsBusy => IsFinding || IsSurveying;

    public bool CliMissing { get; }

    public string CliPath { get; }

    /// <summary>
    /// Find Now doubles as Stop while a find runs; a survey has its own Stop button.
    /// </summary>
    public bool CanUseFindButton => !CliMissing && !IsSurveying;

    public bool CanUseSurveyButton => !CliMissing && !IsFinding;

    public string FindButtonTitle => IsFinding ? StopFindTitle : FindNowTitle;

    public string FindMenuItemTitle => IsFinding ? StopFindMenuTitle : FindNowTitle;

    public string FindButtonToolTip => IsFinding ? ToolTipText.StopFind : ToolTipText.FindNow;

    public string SurveyButtonTitle => IsSurveying ? StopSurveyTitle : StartSurveyTitle;

    public string SurveyButtonToolTip => IsSurveying ? ToolTipText.StopSurvey : ToolTipText.StartSurvey;

    public ObservableCollection<OutputLine> RecentLines { get; } = [];

    public bool AutoReconnect
    {
        get => preferences.AutoReconnect;
        set
        {
            if (value != preferences.AutoReconnect)
            {
                preferences.SetAutoReconnect(value);
                OnPropertyChanged();
            }
        }
    }

    /// <summary>
    /// Starts the once-a-minute watch and reads the current tunnel.
    /// </summary>
    public void Start()
    {
        watchTimer.Start();
        _ = RefreshStatusAsync();
    }

    /// <summary>
    /// Starts a find, or stops the one that is running.
    /// </summary>
    public void ToggleFind()
    {
        if (IsFinding)
        {
            searchCancellation?.Cancel();
            return;
        }
        if (CanUseFindButton)
        {
            _ = RunFindAsync(cancellationToken => finder.FindAsync(cancellationToken));
        }
    }

    /// <summary>
    /// Starts a survey, or stops the one that is running.
    /// </summary>
    public void ToggleSurvey()
    {
        if (IsSurveying)
        {
            searchCancellation?.Cancel();
            return;
        }
        if (!CanUseSurveyButton)
        {
            return;
        }
        IsSurveying = true;
        _ = RunAsync(cancellationToken => finder.SurveyAsync(cancellationToken));
    }

    /// <summary>
    /// Opens log.csv in whatever app handles .csv files, creating it first if no attempt was made yet.
    /// </summary>
    public void ShowLog()
    {
        paths.EnsureExists();
        if (!File.Exists(paths.LogFile))
        {
            File.WriteAllText(paths.LogFile, string.Empty);
        }
        Process.Start(new ProcessStartInfo(paths.LogFile) { UseShellExecute = true });
    }

    public void Dispose()
    {
        watchTimer.Stop();
        searchCancellation?.Cancel();
        searchCancellation?.Dispose();
    }

    /// <summary>
    /// Checks the tunnel quietly and starts a watch only when it stopped working, so a healthy minute
    /// never turns Find Now into Stop.
    /// </summary>
    private async Task WatchTickAsync()
    {
        if (!AutoReconnect || IsBusy || CliMissing || isCheckingTunnel)
        {
            return;
        }
        isCheckingTunnel = true;
        bool works;
        try
        {
            works = await Task.Run(() => tunnel.WorksAsync(CancellationToken.None));
        }
        finally
        {
            isCheckingTunnel = false;
        }
        if (!works && AutoReconnect && !IsBusy)
        {
            await RunFindAsync(cancellationToken => finder.WatchAsync(cancellationToken));
        }
    }

    private Task RunFindAsync(Func<CancellationToken, Task<bool>> search)
    {
        IsFinding = true;
        return RunAsync(search);
    }

    /// <summary>
    /// Runs one search on a worker thread with a fresh cancellation, then reads the tunnel again.
    /// </summary>
    private async Task RunAsync(Func<CancellationToken, Task<bool>> search)
    {
        searchCancellation = new CancellationTokenSource();
        var cancellationToken = searchCancellation.Token;
        try
        {
            await Task.Run(() => search(cancellationToken), CancellationToken.None);
        }
        catch (Exception exception)
        {
            Append(new OutputLine("engine error: " + exception.Message, OutputKind.Other));
        }
        finally
        {
            searchCancellation.Dispose();
            searchCancellation = null;
            IsFinding = false;
            IsSurveying = false;
        }
        await RefreshStatusAsync();
    }

    /// <summary>
    /// Reads the current tunnel from the CLI without touching it.
    /// </summary>
    private async Task RefreshStatusAsync()
    {
        if (CliMissing)
        {
            State = ConnectionState.Down;
            return;
        }
        var status = await Task.Run(() => tunnel.ReadStatusAsync(CancellationToken.None));
        if (IsBusy)
        {
            return;
        }
        State = status.IsConnected
            ? new ConnectionState(ConnectionKind.Connected, status.Location, status.Protocol)
            : ConnectionState.Down;
    }

    private void OnSearchStateChanged()
    {
        OnPropertyChanged(nameof(IsBusy));
        OnPropertyChanged(nameof(CanUseFindButton));
        OnPropertyChanged(nameof(CanUseSurveyButton));
        OnPropertyChanged(nameof(FindButtonTitle));
        OnPropertyChanged(nameof(FindMenuItemTitle));
        OnPropertyChanged(nameof(FindButtonToolTip));
        OnPropertyChanged(nameof(SurveyButtonTitle));
        OnPropertyChanged(nameof(SurveyButtonToolTip));
    }

    private void Handle(EngineEvent engineEvent)
    {
        Append(OutputLine.From(engineEvent));
        switch (engineEvent)
        {
            case TryingEvent trying:
                State = new ConnectionState(ConnectionKind.Searching, trying.Nickname, trying.Protocol);
                break;
            case ConnectedEvent connected:
                State = new ConnectionState(ConnectionKind.Connected, connected.Nickname, connected.Protocol);
                NotificationRequested?.Invoke(ConnectedNotificationTitle, State.Title);
                break;
            case NothingWorkedEvent:
                State = ConnectionState.Down;
                NotificationRequested?.Invoke(NothingWorkedNotificationTitle, NothingWorkedNotificationBody);
                break;
            case TunnelDownEvent:
                State = ConnectionState.Down;
                NotificationRequested?.Invoke(TunnelDownNotificationTitle, TunnelDownNotificationBody);
                break;
            case SearchStoppedEvent:
                State = ConnectionState.Down;
                break;
        }
    }

    private void Append(OutputLine line)
    {
        RecentLines.Add(line);
        while (RecentLines.Count > RecentLineLimit)
        {
            RecentLines.RemoveAt(0);
        }
    }
}
