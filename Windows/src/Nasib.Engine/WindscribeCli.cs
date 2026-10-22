using System.Diagnostics;
using System.Text;

namespace Nasib.Engine;

/// <summary>
/// Runs the windscribe-cli.exe that ships with the Windscribe app, without a console window.
/// </summary>
public sealed class WindscribeCli : IWindscribeCli
{
    private const string StatusCommand = "status";
    private const string ConnectCommand = "connect";
    private const string DisconnectCommand = "disconnect";
    private const string NonBlockingOption = "-n";

    /// <summary>
    /// A blocking disconnect normally takes a few seconds; this only guards against a CLI that never exits.
    /// </summary>
    private static readonly TimeSpan CommandTimeout = TimeSpan.FromSeconds(60);

    public WindscribeCli(string executablePath)
    {
        ExecutablePath = executablePath;
    }

    public string ExecutablePath { get; }

    public bool IsInstalled => File.Exists(ExecutablePath);

    /// <summary>
    /// Where the Windscribe installer puts the CLI.
    /// </summary>
    public static string DefaultExecutablePath()
    {
        var programFiles = Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles);
        return Path.Combine(programFiles, "Windscribe", "windscribe-cli.exe");
    }

    public Task<string> ReadStatusAsync(CancellationToken cancellationToken)
    {
        return RunAsync(StatusArguments(), cancellationToken);
    }

    public Task ConnectAsync(string nickname, string protocol, CancellationToken cancellationToken)
    {
        return RunAsync(ConnectArguments(nickname, protocol), cancellationToken);
    }

    public Task DisconnectAsync(CancellationToken cancellationToken)
    {
        return RunAsync(DisconnectArguments(), cancellationToken);
    }

    internal static IReadOnlyList<string> StatusArguments() => [StatusCommand];

    internal static IReadOnlyList<string> ConnectArguments(string nickname, string protocol) =>
        [ConnectCommand, NonBlockingOption, nickname, protocol];

    internal static IReadOnlyList<string> DisconnectArguments() => [DisconnectCommand];

    /// <summary>
    /// Runs the CLI and returns its standard output and error together; empty when it cannot be started.
    /// </summary>
    private async Task<string> RunAsync(IReadOnlyList<string> arguments, CancellationToken cancellationToken)
    {
        var startInfo = new ProcessStartInfo(ExecutablePath)
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };
        foreach (var argument in arguments)
        {
            startInfo.ArgumentList.Add(argument);
        }

        using var process = new Process { StartInfo = startInfo };
        try
        {
            process.Start();
        }
        catch (Exception exception) when (exception is System.ComponentModel.Win32Exception or InvalidOperationException)
        {
            return string.Empty;
        }

        using var timeoutSource = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeoutSource.CancelAfter(CommandTimeout);
        try
        {
            var outputTask = process.StandardOutput.ReadToEndAsync(timeoutSource.Token);
            var errorTask = process.StandardError.ReadToEndAsync(timeoutSource.Token);
            await process.WaitForExitAsync(timeoutSource.Token);
            return await outputTask + await errorTask;
        }
        catch (OperationCanceledException)
        {
            process.Kill(entireProcessTree: true);
            cancellationToken.ThrowIfCancellationRequested();
            return string.Empty;
        }
    }
}
