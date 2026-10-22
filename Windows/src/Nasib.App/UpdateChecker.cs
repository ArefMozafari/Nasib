using System;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using System.Windows.Threading;

namespace Nasib.App;

/// <summary>
/// Asks GitHub for the latest release at launch and once a day, and offers it when it is newer than this build.
/// Updating stays manual: the app opens the release page and the user downloads the new zip.
/// </summary>
public sealed class UpdateChecker : ObservableObject, IDisposable
{
    /// <summary>
    /// A release newer than this build: its version and the page it can be downloaded from.
    /// </summary>
    public sealed record Release(string Version, Uri PageAddress);

    private static readonly Uri LatestReleaseAddress = new("https://api.github.com/repos/ArefMozafari/Nasib/releases/latest");
    private static readonly TimeSpan CheckInterval = TimeSpan.FromHours(24);

    /// <summary>
    /// A filtered network can hang a request for a long time; the next daily check retries anyway.
    /// </summary>
    private static readonly TimeSpan RequestTimeout = TimeSpan.FromSeconds(20);

    private const string ProductName = "Nasib";
    private const string GitHubJsonMediaType = "application/vnd.github+json";
    private const string TagNameProperty = "tag_name";
    private const string PageAddressProperty = "html_url";
    private const string TagPrefix = "v";
    private const char VersionSeparator = '.';
    private const char BuildMetadataSeparator = '+';
    private const string NotificationTitle = "Update available";

    private readonly Preferences preferences;
    private readonly DispatcherTimer timer;
    private Release? available;

    public UpdateChecker(Preferences preferences)
    {
        this.preferences = preferences;
        timer = new DispatcherTimer { Interval = CheckInterval };
        timer.Tick += (_, _) => _ = CheckAsync();
    }

    /// <summary>
    /// Raised on the UI thread once per new version, with a title, a body and the page to open on click.
    /// </summary>
    public event Action<string, string, Uri>? NotificationRequested;

    /// <summary>
    /// This build's version as set in Directory.Build.props, without the commit it was built from.
    /// </summary>
    public static string CurrentVersion { get; } =
        (Assembly.GetExecutingAssembly().GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? string.Empty)
            .Split(BuildMetadataSeparator)[0];

    public Release? Available
    {
        get => available;
        private set
        {
            if (SetProperty(ref available, value))
            {
                OnPropertyChanged(nameof(HasUpdate));
                OnPropertyChanged(nameof(UpdateButtonTitle));
                OnPropertyChanged(nameof(UpdateMenuItemTitle));
            }
        }
    }

    public bool HasUpdate => Available is not null;

    public string UpdateButtonTitle => Available is null ? string.Empty : ButtonTitle(Available);

    public string UpdateMenuItemTitle => Available is null ? string.Empty : MenuItemTitle(Available);

    public static string ButtonTitle(Release release) => "Update to " + release.Version;

    public static string MenuItemTitle(Release release) => ButtonTitle(release) + "…";

    /// <summary>
    /// Compares dotted version numbers component by component; a missing component counts as zero.
    /// </summary>
    public static bool IsNewer(string candidate, string current)
    {
        var candidateParts = ParseVersion(candidate);
        var currentParts = ParseVersion(current);
        for (var index = 0; index < Math.Max(candidateParts.Length, currentParts.Length); index++)
        {
            var candidatePart = index < candidateParts.Length ? candidateParts[index] : 0;
            var currentPart = index < currentParts.Length ? currentParts[index] : 0;
            if (candidatePart != currentPart)
            {
                return candidatePart > currentPart;
            }
        }
        return false;
    }

    /// <summary>
    /// Splits "0.10.2" into its numbers; a part that isn't a number counts as zero.
    /// </summary>
    private static int[] ParseVersion(string version)
    {
        return version
            .Split(VersionSeparator)
            .Select(part => int.TryParse(part, NumberStyles.None, CultureInfo.InvariantCulture, out var number) ? number : 0)
            .ToArray();
    }

    /// <summary>
    /// Reads the version and release page from GitHub's latest-release response; null for anything else.
    /// </summary>
    public static Release? ParseRelease(string json)
    {
        try
        {
            using var document = JsonDocument.Parse(json);
            var root = document.RootElement;
            if (root.ValueKind != JsonValueKind.Object
                || !root.TryGetProperty(TagNameProperty, out var tagName) || tagName.ValueKind != JsonValueKind.String
                || !root.TryGetProperty(PageAddressProperty, out var page) || page.ValueKind != JsonValueKind.String
                || !Uri.TryCreate(page.GetString(), UriKind.Absolute, out var pageAddress))
            {
                return null;
            }
            var tag = tagName.GetString()!;
            var version = tag.StartsWith(TagPrefix, StringComparison.Ordinal) ? tag[TagPrefix.Length..] : tag;
            return new Release(version, pageAddress);
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>
    /// Checks now and then once a day.
    /// </summary>
    public void Start()
    {
        timer.Start();
        _ = CheckAsync();
    }

    public void OpenReleasePage()
    {
        if (Available is not null)
        {
            Process.Start(new ProcessStartInfo(Available.PageAddress.AbsoluteUri) { UseShellExecute = true });
        }
    }

    public void Dispose()
    {
        timer.Stop();
    }

    /// <summary>
    /// Stays quiet on failure: a filtered network often can't reach GitHub, and the next check retries.
    /// </summary>
    private async Task CheckAsync()
    {
        var release = await Task.Run(FetchLatestAsync);
        if (release is null || !IsNewer(release.Version, CurrentVersion))
        {
            return;
        }
        Available = release;
        NotifyOnce(release);
    }

    private static async Task<Release?> FetchLatestAsync()
    {
        try
        {
            using var client = new HttpClient { Timeout = RequestTimeout };

            // GitHub's API refuses requests without a User-Agent, which HttpClient doesn't send by default.
            client.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue(ProductName, CurrentVersion));
            client.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue(GitHubJsonMediaType));
            using var response = await client.GetAsync(LatestReleaseAddress);
            if (!response.IsSuccessStatusCode)
            {
                return null;
            }
            return ParseRelease(await response.Content.ReadAsStringAsync());
        }
        catch (Exception exception) when (exception is HttpRequestException or TaskCanceledException)
        {
            return null;
        }
    }

    /// <summary>
    /// Posts one notification per new version, remembered across restarts.
    /// </summary>
    private void NotifyOnce(Release release)
    {
        if (preferences.NotifiedUpdateVersion == release.Version)
        {
            return;
        }
        preferences.SetNotifiedUpdateVersion(release.Version);
        NotificationRequested?.Invoke(NotificationTitle, $"Nasib {release.Version} is ready to download.", release.PageAddress);
    }
}
