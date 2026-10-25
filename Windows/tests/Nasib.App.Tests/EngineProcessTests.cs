using System.IO;

namespace Nasib.App.Tests;

/// <summary>
/// Runs EngineProcess against a stub engine: cmd.exe printing lines, then waiting for one line on its input.
/// </summary>
public class EngineProcessTests
{
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(10);

    private static string CommandProcessor => Path.Combine(Environment.SystemDirectory, "cmd.exe");

    /// <summary>
    /// Prints two engine lines, waits for a line on standard input, echoes it, and exits with code 3.
    /// Delayed expansion reads the variable after set /p fills it.
    /// </summary>
    private static readonly string[] StubThatWaitsForStop =
        ["/v:on", "/c", "echo trying: The 6 / tcp:443& echo failed: The 6 / tcp:443& set /p received=& echo received !received!& exit 3"];

    public class Start
    {
        [Fact(DisplayName = "should pass every line and then the exit code")]
        public async Task ShouldPassEveryLineAndThenTheExitCode()
        {
            var lines = new List<string>();
            var exited = new TaskCompletionSource<int>();
            using var process = new EngineProcess(CommandProcessor, StubThatWaitsForStop);
            process.LineReceived += line => { lock (lines) { lines.Add(line); } };
            process.Exited += code => exited.TrySetResult(code);

            Assert.True(process.Start());
            await Task.Delay(TimeSpan.FromMilliseconds(300));
            process.Stop();
            var code = await exited.Task.WaitAsync(Patience);

            Assert.Equal(3, code);
            Assert.Equal(["trying: The 6 / tcp:443", "failed: The 6 / tcp:443", "received stop"], lines);
        }

        [Fact(DisplayName = "should not start an engine that is not there")]
        public void ShouldNotStartAnEngineThatIsNotThere()
        {
            using var process = new EngineProcess(Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"), "nasib-engine.exe"), []);

            Assert.False(process.Start());
        }
    }

    public class Dispose
    {
        // Closing the engine's input is how it learns the app quit, so it can finish its clean-up on its own.
        [Fact(DisplayName = "should close the engine's input")]
        public async Task ShouldCloseTheEnginesInput()
        {
            var exited = new TaskCompletionSource<int>();
            var process = new EngineProcess(CommandProcessor, StubThatWaitsForStop);
            process.Exited += code => exited.TrySetResult(code);
            Assert.True(process.Start());

            process.Dispose();

            Assert.Equal(3, await exited.Task.WaitAsync(Patience));
        }
    }

    public class RunOnceAsync
    {
        [Fact(DisplayName = "should return what a one-shot mode prints, trimmed")]
        public async Task ShouldReturnWhatAOneShotModePrintsTrimmed()
        {
            var output = await EngineProcess.RunOnceAsync(CommandProcessor, ["/c", "echo connected: Dubai - Khalifa / IKEv2 "], CancellationToken.None);

            Assert.Equal("connected: Dubai - Khalifa / IKEv2", output);
        }

        [Fact(DisplayName = "should return nothing when the engine is not there")]
        public async Task ShouldReturnNothingWhenTheEngineIsNotThere()
        {
            var output = await EngineProcess.RunOnceAsync(Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"), "nasib-engine.exe"), ["--status"], CancellationToken.None);

            Assert.Equal(string.Empty, output);
        }
    }
}
