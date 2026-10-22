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
            var attempts = new[] { Attempt("Fika", "tcp", false), Attempt("Lygos", "udp", false) };

            var ranked = SuccessRanker.Rank(["Khalifa", "Lygos", "Vitoshka", "Fika"], attempts, attempt => attempt.Nickname);

            Assert.Equal(["Khalifa", "Lygos", "Vitoshka", "Fika"], ranked);
        }

        [Fact(DisplayName = "should count every protocol tried on a datacenter towards its rate")]
        public void ShouldCountEveryProtocolTriedOnADatacenterTowardsItsRate()
        {
            var attempts = new[]
            {
                Attempt("Lygos", "wstunnel", true),
                Attempt("Lygos", "wireguard", true),
                Attempt("Khalifa", "wireguard", true),
                Attempt("Khalifa", "ikev2", false),
            };

            var ranked = SuccessRanker.Rank(["Khalifa", "Lygos"], attempts, attempt => attempt.Nickname);

            Assert.Equal(["Lygos", "Khalifa"], ranked);
        }
    }
}
