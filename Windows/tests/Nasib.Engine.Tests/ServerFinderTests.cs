using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class ServerFinderTests
{
    public class FindAsync
    {
        // Regression for issue #2: Find Now while connected must not burn an attempt on the last-good pair.
        [Fact(DisplayName = "should log one attempt when searching while connected")]
        public async Task ShouldLogOneAttemptWhenSearchingWhileConnected()
        {
            using var harness = new EngineHarness();
            harness.Cli.State = FakeTunnelState.Connected;
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);

            var found = await harness.Finder.FindAsync();

            Assert.True(found);
            Assert.Equal(["Khalifa,ikev2,ok"], harness.LoggedAttempts());
        }

        // Baseline: from a disconnected state the first attempt succeeds, as before the fix.
        [Fact(DisplayName = "should log one attempt when searching while disconnected")]
        public async Task ShouldLogOneAttemptWhenSearchingWhileDisconnected()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);

            var found = await harness.Finder.FindAsync();

            Assert.True(found);
            Assert.Equal(["Khalifa,ikev2,ok"], harness.LoggedAttempts());
        }

        [Fact(DisplayName = "should not add a disconnect wait when the CLI keeps reporting an error")]
        public async Task ShouldNotAddADisconnectWaitWhenTheCliKeepsReportingAnError()
        {
            using var harness = new EngineHarness();
            harness.Cli.State = FakeTunnelState.Error;
            harness.Cli.ErrorIsSticky = true;
            harness.Cli.PairConnects = (_, _) => false;
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], [EngineHarness.Protocol]));

            await harness.Finder.FindAsync();

            // One failed attempt costs the connect timeout plus the settle delay, and nothing for the teardown.
            var oneFailedAttempt = EngineTimings.Default.ConnectTimeout + EngineTimings.Default.SettleDelay;
            Assert.Equal(oneFailedAttempt, harness.Waiter.TotalWaited);
        }

        [Fact(DisplayName = "should stop at the first pair that works")]
        public async Task ShouldStopAtTheFirstPairThatWorks()
        {
            using var harness = new EngineHarness();
            harness.Cli.PairConnects = (_, protocol) => protocol == "tcp";
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp", "udp"]));

            var found = await harness.Finder.FindAsync();

            Assert.True(found);
            Assert.Equal(["Khalifa,ikev2,fail", "Khalifa,tcp,ok"], harness.LoggedAttempts());
            Assert.Equal(new ServerPair("Khalifa", "tcp"), harness.AttemptLog.ReadLastGood());
        }

        [Fact(DisplayName = "should count a connect only when the blocked site loads")]
        public async Task ShouldCountAConnectOnlyWhenTheBlockedSiteLoads()
        {
            using var harness = new EngineHarness();
            harness.BlockedSiteProbe.LoadsThrough = _ => false;
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], [EngineHarness.Protocol]));

            var found = await harness.Finder.FindAsync();

            Assert.False(found);
            Assert.Equal(["Khalifa,ikev2,fail"], harness.LoggedAttempts());
            Assert.IsType<NothingWorkedEvent>(harness.Events[^1]);
        }

        [Fact(DisplayName = "should try every datacenter when no address answers")]
        public async Task ShouldTryEveryDatacenterWhenNoAddressAnswers()
        {
            var seed = new List<Datacenter> { new("Sofia", "Vitoshka", ["150.241.139.14"]) };
            using var harness = new EngineHarness(seed, new FakeReachabilityProbe(new HashSet<string>()));
            harness.Cli.PairConnects = (_, _) => false;
            harness.SaveConfiguration(new EngineConfiguration([], [EngineHarness.Protocol]));

            await harness.Finder.FindAsync();

            Assert.Contains(harness.Events, engineEvent => engineEvent.Text == "no address answered; trying all locations");
            Assert.Equal(["Vitoshka,ikev2,fail", "Khalifa,ikev2,fail"], harness.LoggedAttempts());
        }

        [Fact(DisplayName = "should skip datacenters where no address answers")]
        public async Task ShouldSkipDatacentersWhereNoAddressAnswers()
        {
            var seed = new List<Datacenter> { new("Sofia", "Vitoshka", ["150.241.139.14"]) };
            using var harness = new EngineHarness(seed, new FakeReachabilityProbe(new HashSet<string> { EngineHarness.Address }));
            harness.Cli.PairConnects = (_, _) => false;
            harness.SaveConfiguration(new EngineConfiguration([], [EngineHarness.Protocol]));

            await harness.Finder.FindAsync();

            Assert.Equal(["Khalifa,ikev2,fail"], harness.LoggedAttempts());
        }

        [Fact(DisplayName = "should try datacenters with the best recent success rate first")]
        public async Task ShouldTryDatacentersWithTheBestRecentSuccessRateFirst()
        {
            var seed = new List<Datacenter> { new("Sofia", "Vitoshka", ["150.241.139.14"]) };
            using var harness = new EngineHarness(seed);
            harness.AttemptLog.Record("Vitoshka", "ikev2", succeeded: false);
            harness.AttemptLog.Record("Khalifa", "ikev2", succeeded: true);
            harness.SaveConfiguration(new EngineConfiguration([], [EngineHarness.Protocol]));

            await harness.Finder.FindAsync();

            Assert.Contains(harness.Events, engineEvent => engineEvent.Text == "location order (by recent success): Khalifa, Vitoshka");
            Assert.Equal(new TryingEvent("Khalifa", "ikev2"), harness.Events.OfType<TryingEvent>().First());
        }

        [Fact(DisplayName = "should try protocols in the order set in Settings, whatever worked recently")]
        public async Task ShouldTryProtocolsInTheOrderSetInSettingsWhateverWorkedRecently()
        {
            using var harness = new EngineHarness();
            harness.AttemptLog.Record("Khalifa", "wireguard", succeeded: true);
            harness.AttemptLog.Record("Khalifa", "wireguard", succeeded: true);
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "wireguard"]));

            await harness.Finder.FindAsync();

            Assert.Equal(("Khalifa", "ikev2"), harness.Cli.ConnectCalls[0]);
            Assert.Contains(harness.Events, engineEvent => engineEvent.Text == "protocol order (from Settings): ikev2, wireguard");
            Assert.Contains(harness.Events, engineEvent => engineEvent.Text == "location order (by recent success): Khalifa");
        }

        [Fact(DisplayName = "should refuse to start while another search is running")]
        public async Task ShouldRefuseToStartWhileAnotherSearchIsRunning()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);
            var secondSearch = Task.FromResult(true);
            harness.Cli.BeforeConnect = (_, _) => secondSearch = harness.Finder.FindAsync();

            await harness.Finder.FindAsync();

            Assert.False(await secondSearch);
            Assert.Contains(harness.Events, engineEvent => engineEvent.Text == "search already running");
        }

        [Fact(DisplayName = "should report a failed attempt after logging it")]
        public async Task ShouldReportAFailedAttemptAfterLoggingIt()
        {
            using var harness = new EngineHarness();
            harness.Cli.PairConnects = (_, protocol) => protocol == "tcp";
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp"]));

            await harness.Finder.FindAsync();

            var pairEvents = harness.Events.Where(engineEvent => engineEvent is TryingEvent or FailedEvent or ConnectedEvent);
            Assert.Equal(
                ["trying: Khalifa / ikev2", "failed: Khalifa / ikev2", "trying: Khalifa / tcp", "connected: Khalifa / tcp"],
                pairEvents.Select(engineEvent => engineEvent.Text));
        }

        [Fact(DisplayName = "should stop promptly and stay disconnected when cancelled mid-attempt")]
        public async Task ShouldStopPromptlyAndStayDisconnectedWhenCancelledMidAttempt()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp"]));
            using var cancellation = new CancellationTokenSource();

            // Cancelled right as the tunnel starts coming up, so the stop has a live tunnel to tear down.
            harness.Cli.BeforeConnect = (_, _) => cancellation.Cancel();

            var found = await harness.Finder.FindAsync(cancellation.Token);

            Assert.False(found);
            Assert.Single(harness.Cli.ConnectCalls);
            Assert.Equal(FakeTunnelState.Disconnected, harness.Cli.State);
            Assert.Empty(harness.LoggedAttempts());
            Assert.IsType<SearchStoppedEvent>(harness.Events[^1]);
            Assert.True(harness.Waiter.TotalWaited < EngineTimings.Default.ConnectTimeout);
        }

        [Fact(DisplayName = "should stop at once when cancelled during the blocked-site check")]
        public async Task ShouldStopAtOnceWhenCancelledDuringTheBlockedSiteCheck()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);
            using var cancellation = new CancellationTokenSource();
            harness.BlockedSiteProbe.HangsAfter = cancellation.Cancel;

            var search = harness.Finder.FindAsync(cancellation.Token);
            var finished = await Task.WhenAny(search, Task.Delay(TimeSpan.FromSeconds(2)));

            Assert.Same(search, finished);
            Assert.False(await search);
            Assert.Empty(harness.LoggedAttempts());
            Assert.Equal(FakeTunnelState.Disconnected, harness.Cli.State);
            Assert.IsType<SearchStoppedEvent>(harness.Events[^1]);
        }

        [Fact(DisplayName = "should not reconnect the last good pair when stopped")]
        public async Task ShouldNotReconnectTheLastGoodPairWhenStopped()
        {
            using var harness = new EngineHarness();
            harness.Cli.PairConnects = (_, _) => false;
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp", "udp"]));
            using var cancellation = new CancellationTokenSource();
            harness.Cli.BeforeConnect = (_, protocol) =>
            {
                if (protocol == "tcp")
                {
                    harness.WriteLastGood(EngineHarness.Nickname, "wireguard");
                    cancellation.Cancel();
                }
            };

            await harness.Finder.FindAsync(cancellation.Token);

            Assert.Equal([("Khalifa", "ikev2"), ("Khalifa", "tcp")], harness.Cli.ConnectCalls);
            Assert.Equal(["Khalifa,ikev2,fail"], harness.LoggedAttempts());
            Assert.DoesNotContain(harness.Events, engineEvent => engineEvent is SurveyStoppedEvent);
        }

        [Fact(DisplayName = "should report when no datacenter matches the cities")]
        public async Task ShouldReportWhenNoDatacenterMatchesTheCities()
        {
            using var harness = new EngineHarness();
            harness.SaveConfiguration(new EngineConfiguration(["Reykjavik"], [EngineHarness.Protocol]));

            var found = await harness.Finder.FindAsync();

            Assert.False(found);
            Assert.Empty(harness.Cli.ConnectCalls);
        }
    }

    public class WatchAsync
    {
        [Fact(DisplayName = "should do nothing when the tunnel works")]
        public async Task ShouldDoNothingWhenTheTunnelWorks()
        {
            using var harness = new EngineHarness();
            harness.Cli.State = FakeTunnelState.Connected;

            var healthy = await harness.Finder.WatchAsync();

            Assert.True(healthy);
            Assert.Empty(harness.Cli.ConnectCalls);
            Assert.Empty(harness.Events);
        }

        [Fact(DisplayName = "should search when the tunnel is down")]
        public async Task ShouldSearchWhenTheTunnelIsDown()
        {
            using var harness = new EngineHarness();
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], [EngineHarness.Protocol]));

            var found = await harness.Finder.WatchAsync();

            Assert.True(found);
            Assert.IsType<TunnelDownEvent>(harness.Events[0]);
            Assert.Equal(["Khalifa,ikev2,ok"], harness.LoggedAttempts());
        }

        [Fact(DisplayName = "should search when the tunnel is up but the blocked site does not load")]
        public async Task ShouldSearchWhenTheTunnelIsUpButTheBlockedSiteDoesNotLoad()
        {
            using var harness = new EngineHarness();
            harness.Cli.State = FakeTunnelState.Connected;
            harness.Cli.ConnectedLocation = "Atlanta - Magic City";
            harness.BlockedSiteProbe.LoadsThrough = location => location == EngineHarness.Nickname;
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], [EngineHarness.Protocol]));

            await harness.Finder.WatchAsync();

            Assert.IsType<TunnelDownEvent>(harness.Events[0]);
            Assert.Equal(["Khalifa,ikev2,ok"], harness.LoggedAttempts());
        }

        [Fact(DisplayName = "should stop like a find when cancelled")]
        public async Task ShouldStopLikeAFindWhenCancelled()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, EngineHarness.Protocol);
            using var cancellation = new CancellationTokenSource();
            harness.Cli.BeforeConnect = (_, _) => cancellation.Cancel();

            var healthy = await harness.Finder.WatchAsync(cancellation.Token);

            Assert.False(healthy);
            Assert.Single(harness.Cli.ConnectCalls);
            Assert.Equal(FakeTunnelState.Disconnected, harness.Cli.State);
            Assert.IsType<SearchStoppedEvent>(harness.Events[^1]);
        }
    }

    public class SurveyAsync
    {
        [Fact(DisplayName = "should try every pair without stopping at a success")]
        public async Task ShouldTryEveryPairWithoutStoppingAtASuccess()
        {
            using var harness = new EngineHarness();
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp"]));

            await harness.Finder.SurveyAsync();

            Assert.Equal(["Khalifa,ikev2,ok", "Khalifa,tcp,ok", "Khalifa,tcp,ok"], harness.LoggedAttempts());
            Assert.Contains(harness.Events, engineEvent => engineEvent is SurveyFinishedEvent);
        }

        [Fact(DisplayName = "should reconnect the last good pair when cancelled")]
        public async Task ShouldReconnectTheLastGoodPairWhenCancelled()
        {
            using var harness = new EngineHarness();
            harness.WriteLastGood(EngineHarness.Nickname, "wireguard");
            harness.Cli.PairConnects = (_, protocol) => protocol == "wireguard";
            harness.SaveConfiguration(new EngineConfiguration([EngineHarness.City], ["ikev2", "tcp"]));
            using var cancellation = new CancellationTokenSource();
            harness.Cli.BeforeConnect = (_, _) => cancellation.Cancel();

            var reconnected = await harness.Finder.SurveyAsync(cancellation.Token);

            Assert.True(reconnected);
            Assert.Contains(harness.Events, engineEvent => engineEvent is SurveyStoppedEvent);
            Assert.Equal(("Khalifa", "wireguard"), harness.Cli.ConnectCalls[^1]);
            Assert.Equal(FakeTunnelState.Connected, harness.Cli.State);
        }
    }
}
