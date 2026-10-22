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
/// </summary>
public sealed class EngineController : ObservableObject, IDisposable
{
    private const int RecentLineLimit = 200;
    private static readonly TimeSpan WatchInterval = TimeSpan.FromSeconds(60);

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
    private bool isBusy;
    private bool isSurveying;
    private CancellationTokenSource? surveyCancellation;

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
        watchTimer.Tick += (_, _) => WatchTick();
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

    public bool IsBusy
    {
        get => isBusy;
        private set
        {
            if (SetProperty(ref isBusy, value))
            {
                OnPropertyChanged(nameof(CanStartSearch));
                OnPropertyChanged(nameof(CanUseSurveyButton));
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
                OnPropertyChanged(nameof(SurveyButtonTitle));
                OnPropertyChanged(nameof(CanUseSurveyButton));
            }
        }
    }

    public bool CliMissing { get; }

    public string CliPath { get; }

    public bool CanStartSearch => !IsBusy && !CliMissing;

    public bool CanUseSurveyButton => IsSurveying || CanStartSearch;

    public string SurveyButtonTitle => IsSurveying ? "Stop Survey" : "Start Survey";

    public ObservableCollection<string> RecentLines { get; } = [];

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

    public void FindNow()
    {
        _ = RunAsync(cancellationToken => finder.FindAsync(cancellationToken), CancellationToken.None);
    }

    public void ToggleSurvey()
    {
        if (IsSurveying)
        {
            surveyCancellation?.Cancel();
            return;
        }
        if (!CanStartSearch)
        {
            return;
        }
        surveyCancellation = new CancellationTokenSource();
        IsSurveying = true;
        _ = RunAsync(cancellationToken => finder.SurveyAsync(cancellationToken), surveyCancellation.Token);
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
        surveyCancellation?.Cancel();
        surveyCancellation?.Dispose();
    }

    /// <summary>
    /// Runs Watch, which returns at once when the tunnel works.
    /// </summary>
    private void WatchTick()
    {
        if (AutoReconnect && CanStartSearch)
        {
            _ = RunAsync(cancellationToken => finder.WatchAsync(cancellationToken), CancellationToken.None);
        }
    }

    /// <summary>
    /// Runs one search on a worker thread; refuses while another is in progress.
    /// </summary>
    private async Task RunAsync(Func<CancellationToken, Task<bool>> search, CancellationToken cancellationToken)
    {
        if (IsBusy || CliMissing)
        {
            return;
        }
        IsBusy = true;
        try
        {
            await Task.Run(() => search(cancellationToken), CancellationToken.None);
        }
        catch (Exception exception)
        {
            Append("engine error: " + exception.Message);
        }
        finally
        {
            IsBusy = false;
            IsSurveying = false;
            surveyCancellation?.Dispose();
            surveyCancellation = null;
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

    private void Handle(EngineEvent engineEvent)
    {
        Append(engineEvent.Text);
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
        }
    }

    private void Append(string line)
    {
        RecentLines.Add(line);
        while (RecentLines.Count > RecentLineLimit)
        {
            RecentLines.RemoveAt(0);
        }
    }
}
