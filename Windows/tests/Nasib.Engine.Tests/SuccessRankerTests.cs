namespace Nasib.Engine.Tests;

public class SuccessRankerTests
{
    public class Rank
    {
        private static AttemptRecord Attempt(string nickname, string protocol, bool succeeded) =>
            new("2026-10-22T14:00:00", nickname, protocol, succeeded);

        [Fact(DisplayName = "should put the best success rate first")]
        public void ShouldPutTheBestSuccessRateFirst()
        {
            var attempts = new[]
            {
                Attempt("Lygos", "ikev2", false),
                Attempt("Khalifa", "ikev2", true),
                Attempt("Khalifa", "ikev2", false),
                Attempt("Fika", "ikev2", true),
            };

            var ranked = SuccessRanker.Rank(["Lygos", "Khalifa", "Fika"], attempts, attempt => attempt.Nickname);

            Assert.Equal(["Fika", "Khalifa", "Lygos"], ranked);
        }

        [Fact(DisplayName = "should keep the given order for ties and names never tried")]
        public void ShouldKeepTheGivenOrderForTiesAndNamesNeverTried()
        {
            var attempts = new[] { Attempt("Lygos", "tcp", false), Attempt("Lygos", "udp", false) };

            var ranked = SuccessRanker.Rank(["wireguard", "ikev2", "tcp", "udp"], attempts, attempt => attempt.Protocol);

            Assert.Equal(["wireguard", "ikev2", "tcp", "udp"], ranked);
        }

        [Fact(DisplayName = "should rank by protocol when given the protocol key")]
        public void ShouldRankByProtocolWhenGivenTheProtocolKey()
        {
            var attempts = new[] { Attempt("Lygos", "wstunnel", true), Attempt("Khalifa", "wireguard", false) };

            var ranked = SuccessRanker.Rank(["wireguard", "ikev2", "wstunnel"], attempts, attempt => attempt.Protocol);

            Assert.Equal(["wstunnel", "wireguard", "ikev2"], ranked);
        }
    }
}
