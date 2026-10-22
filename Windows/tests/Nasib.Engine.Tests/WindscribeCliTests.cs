namespace Nasib.Engine.Tests;

public class WindscribeCliTests
{
    public class ConnectArguments
    {
        [Fact(DisplayName = "should connect without blocking by nickname and protocol")]
        public void ShouldConnectWithoutBlockingByNicknameAndProtocol()
        {
            Assert.Equal(["connect", "-n", "Stolen Cheese", "ikev2"], WindscribeCli.ConnectArguments("Stolen Cheese", "ikev2"));
        }
    }

    public class StatusArguments
    {
        [Fact(DisplayName = "should ask for the status")]
        public void ShouldAskForTheStatus()
        {
            Assert.Equal(["status"], WindscribeCli.StatusArguments());
        }
    }

    public class DisconnectArguments
    {
        [Fact(DisplayName = "should disconnect and wait for it")]
        public void ShouldDisconnectAndWaitForIt()
        {
            Assert.Equal(["disconnect"], WindscribeCli.DisconnectArguments());
        }
    }

    public class ReadStatusAsync
    {
        [Fact(DisplayName = "should return nothing when the CLI is not installed")]
        public async Task ShouldReturnNothingWhenTheCliIsNotInstalled()
        {
            var cli = new WindscribeCli(Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"), "windscribe-cli.exe"));

            Assert.False(cli.IsInstalled);
            Assert.Equal(string.Empty, await cli.ReadStatusAsync(CancellationToken.None));
        }
    }

    public class DefaultExecutablePath
    {
        [Fact(DisplayName = "should point at the CLI in the Windscribe install folder")]
        public void ShouldPointAtTheCliInTheWindscribeInstallFolder()
        {
            Assert.EndsWith(Path.Combine("Windscribe", "windscribe-cli.exe"), WindscribeCli.DefaultExecutablePath());
        }
    }
}
