using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Threading;

namespace Nasib.App;

/// <summary>
/// Runs nasib-engine for every search and mirrors its progress for the window and the tray.
/// Every search can be stopped: a find or watch stops and disconnects, a survey stops and reconnects.
/// </summary>
public sealed class EngineController : ObservableObject, IDisposable
{
    private const int RecentLineLimit = 200;

    /// <summary>
    /// How often the blocked-site check runs; it costs a request through the tunnel.
    /// </summary>
    public static readonly TimeSpan WatchInterval = TimeSpan.FromSeconds(60);

    /// <summary>
    /// Reading the CLI status costs no traffic, so it runs far more often than the blocked-site check.
    /// </summary>
    public static readonly TimeSpan StatusInterval = TimeSpan.FromSeconds(3);

    private const string WatchArgument = "--watch";
    private const string SurveyArgument = "--survey";
    private const string StatusArgument = "--status";
    private const string CatalogArgument = "--catalog";

    private const string FindNowTitle = "Find Now";
    private const string StopFindTitle = "Stop";
    private const string StopFindMenuTitle = "Stop Search";
    private const string StartSurveyTitle = "Start Survey";
    private const string StopSurveyTitle = "Stop Survey";
    private const string EngineNotStartedLine = "could not start the engine";

    private const string ConnectedNotificationTitle = "Connected";
    private const string NothingWorkedNotificationTitle = "No working server";
    private const string NothingWorkedNotificationBody = "Every datacenter and protocol failed.";
    private const string TunnelDownNotificationTitle = "Tunnel down";
    private const string TunnelDownNotificationBody = "Searching for a working server…";

    private readonly string enginePath;
    private readonly Preferences preferences;
    private readonly StatePaths paths;
    private readonly Dispatcher dispatcher;
    private readonly DispatcherTimer watchTimer;
    private readonly DispatcherTimer statusTimer;

    private ConnectionState state = ConnectionState.Unknown;
    private EngineProcess? running;
    private bool isFinding;
    private bool isSurveying;
    private bool isReadingStatus;
    private bool cliMissing;
    private string cliPath = string.Empty;
    private long linesShown;

    public EngineController(string enginePath, Preferences preferences, StatePaths paths)
    {
        this.enginePath = enginePath;
        this.preferences = preferences;
        this.paths = paths;
        EngineMissing = !File.Exists(enginePath);
        dispatcher = Dispatcher.CurrentDispatcher;
        watchTimer = new DispatcherTimer { Interval = WatchInterval };
        watchTimer.Tick += (_, _) => WatchTick();
        statusTimer = new DispatcherTimer { Interval = StatusInterval };
        statusTimer.Tick += (_, _) => _ = RefreshStatusAsync();
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
    /// True during a find, from Find Now or from an auto-reconnect check that found the tunnel down. A healthy
    /// check runs the engine for a second or two but never searches, so the button doesn't flicker.
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

    /// <summary>
    /// True while the engine runs a find, watch or survey.
    /// </summary>
    public bool IsBusy => running is not null;

    /// <summary>
    /// True when nasib-engine.exe isn't next to Nasib.exe, so nothing can run.
    /// </summary>
    public bool EngineMissing { get; }

    public string EnginePath => enginePath;

    /// <summary>
    /// True once the engine reported that windscribe-cli isn't installed where expected.
    /// </summary>
    public bool CliMissing
    {
        get => cliMissing;
        private set
        {
            if (SetProperty(ref cliMissing, value))
            {
                OnSearchStateChanged();
            }
        }
    }

    public string CliPath
    {
        get => cliPath;
        private set => SetProperty(ref cliPath, value);
    }

    /// <summary>
    /// Find Now doubles as Stop while a find runs; a survey has its own Stop button.
    /// </summary>
    public bool CanUseFindButton => !EngineMissing && !CliMissing && !IsSurveying;

    public bool CanUseSurveyButton => !EngineMissing && !CliMissing && !IsFinding;

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
    /// Whether the line with this number gets the alternating background. Lines are numbered from the first one
    /// shown since launch, so dropping the oldest line doesn't flip every row's stripe.
    /// </summary>
    public static bool IsStriped(long lineNumber) => lineNumber % 2 == 1;

    /// <summary>
    /// Starts the once-a-minute watch and the three-second status read, and reads the current tunnel.
    /// </summary>
    public void Start()
    {
        watchTimer.Start();
        statusTimer.Start();
        _ = RefreshStatusAsync();
    }

    /// <summary>
    /// Starts a find, or stops the one that is running.
    /// </summary>
    public void ToggleFind()
    {
        if (IsFinding)
        {
            running?.Stop();
            return;
        }
        if (CanUseFindButton && running is null && Run([]))
        {
            IsFinding = true;
        }
    }

    /// <summary>
    /// Starts a survey, or stops the one that is running.
    /// </summary>
    public void ToggleSurvey()
    {
        if (IsSurveying)
        {
            running?.Stop();
            return;
        }
        if (CanUseSurveyButton && running is null && Run([SurveyArgument]))
        {
            IsSurveying = true;
        }
    }

    /// <summary>
    /// Every datacenter the engine knows, for Settings; null when the engine can't be read.
    /// </summary>
    public async Task<EngineCatalog?> ReadCatalogAsync()
    {
        var line = await EngineProcess.RunOnceAsync(enginePath, [CatalogArgument], CancellationToken.None);
        return EngineCatalog.Decode(line);
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

    /// <summary>
    /// Closes the running engine's input, which stops it; it finishes its clean-up after the app has gone,
    /// so a survey still reconnects the last pair that worked.
    /// </summary>
    public void Dispose()
    {
        watchTimer.Stop();
        statusTimer.Stop();
        running?.Dispose();
    }

    /// <summary>
    /// Runs the engine's watch, which exits at once when the tunnel is healthy.
    /// </summary>
    private void WatchTick()
    {
        if (AutoReconnect && running is null && !EngineMissing && !CliMissing)
        {
            Run([WatchArgument]);
        }
    }

    /// <summary>
    /// Starts the engine with the given arguments; false when it couldn't be started.
    /// </summary>
    private bool Run(IReadOnlyList<string> arguments)
    {
        var process = new EngineProcess(enginePath, arguments);
        process.LineReceived += line => dispatcher.BeginInvoke(() => Handle(line));
        process.Exited += _ => dispatcher.BeginInvoke(() => OnExited(process));
        if (!process.Start())
        {
            Append(EngineNotStartedLine, OutputTone.Error);
            return false;
        }
        running = process;
        OnPropertyChanged(nameof(IsBusy));
        return true;
    }

    private void OnExited(EngineProcess process)
    {
        if (!ReferenceEquals(process, running))
        {
            return;
        }
        process.Dispose();
        running = null;
        OnPropertyChanged(nameof(IsBusy));
        IsFinding = false;
        IsSurveying = false;
        _ = RefreshStatusAsync();
    }

    /// <summary>
    /// Reads the current tunnel through `nasib-engine --status`, at most one read at a time and never while
    /// the engine runs a search. When a tunnel shown as connected has just dropped and auto-reconnect is on,
    /// the watch starts at once instead of at the next minute's check.
    /// </summary>
    private async Task RefreshStatusAsync()
    {
        if (EngineMissing)
        {
            State = ConnectionState.Down;
            return;
        }
        if (isReadingStatus || IsBusy)
        {
            return;
        }
        isReadingStatus = true;
        string line;
        try
        {
            line = await EngineProcess.RunOnceAsync(enginePath, [StatusArgument], CancellationToken.None);
        }
        finally
        {
            isReadingStatus = false;
        }

        // A search that started during the read owns the state now; this result is already out of date.
        if (IsBusy)
        {
            return;
        }
        if (EngineOutputParser.Parse(line) is CliMissingEvent missing)
        {
            CliPath = missing.Path;
            CliMissing = true;
            State = ConnectionState.Down;
            return;
        }
        CliMissing = false;
        var previous = State;
        State = EngineOutputParser.ParseStatus(line) is { } status
            ? new ConnectionState(ConnectionKind.Connected, status.Location, status.Protocol)
            : ConnectionState.Down;
        if (ConnectionState.TunnelDropped(previous, State) && AutoReconnect)
        {
            WatchTick();
        }
    }

    private void OnSearchStateChanged()
    {
        OnPropertyChanged(nameof(CanUseFindButton));
        OnPropertyChanged(nameof(CanUseSurveyButton));
        OnPropertyChanged(nameof(FindButtonTitle));
        OnPropertyChanged(nameof(FindMenuItemTitle));
        OnPropertyChanged(nameof(FindButtonToolTip));
        OnPropertyChanged(nameof(SurveyButtonTitle));
        OnPropertyChanged(nameof(SurveyButtonToolTip));
    }

    private void Handle(string line)
    {
        var engineEvent = EngineOutputParser.Parse(line);
        Append(line, engineEvent.Tone);
        switch (engineEvent)
        {
            case TryingEvent trying:
                State = new ConnectionState(ConnectionKind.Searching, trying.Location, trying.Protocol);
                break;
            case ConnectedEvent connected:
                State = new ConnectionState(ConnectionKind.Connected, connected.Location, connected.Protocol);
                NotificationRequested?.Invoke(ConnectedNotificationTitle, State.Title);
                break;
            case NothingWorkedEvent:
                State = ConnectionState.Down;
                NotificationRequested?.Invoke(NothingWorkedNotificationTitle, NothingWorkedNotificationBody);
                break;
            case TunnelDownEvent:
                State = ConnectionState.Down;
                IsFinding = true;
                NotificationRequested?.Invoke(TunnelDownNotificationTitle, TunnelDownNotificationBody);
                break;
            case SearchStoppedEvent:
                State = ConnectionState.Down;
                break;
            case CliMissingEvent missing:
                CliPath = missing.Path;
                CliMissing = true;
                State = ConnectionState.Down;
                break;
        }
    }

    private void Append(string text, OutputTone tone)
    {
        RecentLines.Add(new OutputLine(text, tone, IsStriped(linesShown)));
        linesShown++;
        while (RecentLines.Count > RecentLineLimit)
        {
            RecentLines.RemoveAt(0);
        }
    }
}
