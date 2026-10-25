using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace Nasib.App;

/// <summary>
/// Runs nasib-engine.exe without a console window, passes on every line it prints, and stops it with a
/// "stop" line on its standard input. Closing the input, as disposing does, also stops it, so an engine
/// started by an app that quits still finishes its clean-up on its own.
/// </summary>
public sealed class EngineProcess : IDisposable
{
    private const string EngineFileName = "nasib-engine.exe";
    private const string StopLine = "stop";

    /// <summary>
    /// The engine writes UTF-8 and reads it; no byte-order mark, since it would arrive as part of the first line.
    /// </summary>
    private static readonly Encoding EngineEncoding = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false);

    /// <summary>
    /// A one-shot mode such as --status answers in about a second; this only guards against one that hangs.
    /// </summary>
    private static readonly TimeSpan OneShotTimeout = TimeSpan.FromSeconds(30);

    private readonly string executablePath;
    private readonly IReadOnlyList<string> arguments;
    private Process? process;

    public EngineProcess(string executablePath, IReadOnlyList<string> arguments)
    {
        this.executablePath = executablePath;
        this.arguments = arguments;
    }

    /// <summary>
    /// Raised on a worker thread for each line the engine prints, on standard output or standard error.
    /// </summary>
    public event Action<string>? LineReceived;

    /// <summary>
    /// Raised on a worker thread once the engine has exited and every line has been passed on.
    /// </summary>
    public event Action<int>? Exited;

    /// <summary>
    /// The engine shipped next to Nasib.exe.
    /// </summary>
    public static string DefaultExecutablePath => Path.Combine(AppContext.BaseDirectory, EngineFileName);

    /// <summary>
    /// Starts the engine; false when it can't be started, such as when the file is missing.
    /// </summary>
    public bool Start()
    {
        var started = new Process { StartInfo = CreateStartInfo(executablePath, arguments, redirectInput: true), EnableRaisingEvents = true };
        started.OutputDataReceived += (_, eventArguments) => PassOn(eventArguments.Data);
        started.ErrorDataReceived += (_, eventArguments) => PassOn(eventArguments.Data);
        started.Exited += (_, _) => OnExited(started);
        try
        {
            if (!started.Start())
            {
                started.Dispose();
                return false;
            }
        }
        catch (Exception exception) when (exception is Win32Exception or InvalidOperationException)
        {
            started.Dispose();
            return false;
        }
        started.BeginOutputReadLine();
        started.BeginErrorReadLine();
        process = started;
        return true;
    }

    /// <summary>
    /// Asks the engine to stop. A stopped find disconnects; a stopped survey reconnects the last pair that worked.
    /// </summary>
    public void Stop()
    {
        try
        {
            process?.StandardInput.WriteLine(StopLine);
            process?.StandardInput.Flush();
        }
        catch (Exception exception) when (exception is IOException or InvalidOperationException or ObjectDisposedException)
        {
            // The engine has already exited, so there is nothing left to stop.
        }
    }

    /// <summary>
    /// Runs a one-shot mode such as --status or --catalog and returns what it prints, trimmed; empty when
    /// the engine can't run.
    /// </summary>
    public static async Task<string> RunOnceAsync(string executablePath, IReadOnlyList<string> arguments, CancellationToken cancellationToken)
    {
        using var oneShot = new Process { StartInfo = CreateStartInfo(executablePath, arguments, redirectInput: false) };
        try
        {
            if (!oneShot.Start())
            {
                return string.Empty;
            }
        }
        catch (Exception exception) when (exception is Win32Exception or InvalidOperationException)
        {
            return string.Empty;
        }

        using var timeoutSource = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeoutSource.CancelAfter(OneShotTimeout);
        try
        {
            var output = oneShot.StandardOutput.ReadToEndAsync(timeoutSource.Token);
            var errors = oneShot.StandardError.ReadToEndAsync(timeoutSource.Token);
            await oneShot.WaitForExitAsync(timeoutSource.Token);
            await errors;
            return (await output).Trim();
        }
        catch (OperationCanceledException)
        {
            oneShot.Kill(entireProcessTree: true);
            cancellationToken.ThrowIfCancellationRequested();
            return string.Empty;
        }
    }

    /// <summary>
    /// Closes the engine's input, which stops a running search, and lets it finish on its own.
    /// </summary>
    public void Dispose()
    {
        try
        {
            process?.StandardInput.Close();
        }
        catch (Exception exception) when (exception is IOException or InvalidOperationException)
        {
            // The engine has already exited and closed its end.
        }
    }

    private static ProcessStartInfo CreateStartInfo(string executablePath, IReadOnlyList<string> arguments, bool redirectInput)
    {
        var startInfo = new ProcessStartInfo(executablePath)
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = redirectInput,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = EngineEncoding,
            StandardErrorEncoding = EngineEncoding,
        };
        if (redirectInput)
        {
            startInfo.StandardInputEncoding = EngineEncoding;
        }
        foreach (var argument in arguments)
        {
            startInfo.ArgumentList.Add(argument);
        }
        return startInfo;
    }

    private void PassOn(string? line)
    {
        // A null line marks the end of the stream.
        if (line is not null)
        {
            LineReceived?.Invoke(line);
        }
    }

    /// <summary>
    /// Waits for the output to drain before reporting the exit, so no line arrives after Exited.
    /// </summary>
    private void OnExited(Process exited)
    {
        exited.WaitForExit();
        var code = exited.ExitCode;
        exited.Dispose();
        Exited?.Invoke(code);
    }
}
