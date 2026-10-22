using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class TunnelControllerTests
{
    private static (TunnelController Controller, FakeWindscribeCli Cli, FakeBlockedSiteProbe Probe, FakeWaiter Waiter) Create()
    {
        var cli = new FakeWindscribeCli();
        var probe = new FakeBlockedSiteProbe(cli);
        var waiter = new FakeWaiter();
        return (new TunnelController(cli, probe, waiter, EngineTimings.Default), cli, probe, waiter);
    }

    public class DisconnectAndWaitAsync
    {
        [Fact(DisplayName = "should return at once when the tunnel goes down")]
        public async Task ShouldReturnAtOnceWhenTheTunnelGoesDown()
        {
            var (controller, cli, _, waiter) = Create();
            cli.State = FakeTunnelState.Connected;

            var settled = await controller.DisconnectAndWaitAsync(CancellationToken.None);

            Assert.True(settled);
            Assert.Equal(TimeSpan.Zero, waiter.TotalWaited);
        }

        [Fact(DisplayName = "should treat a sticky error as down")]
        public async Task ShouldTreatAStickyErrorAsDown()
        {
            var (controller, cli, _, waiter) = Create();
            cli.State = FakeTunnelState.Error;
            cli.ErrorIsSticky = true;

            var settled = await controller.DisconnectAndWaitAsync(CancellationToken.None);

            Assert.True(settled);
            Assert.Equal(TimeSpan.Zero, waiter.TotalWaited);
        }

        [Fact(DisplayName = "should give up after the disconnect timeout")]
        public async Task ShouldGiveUpAfterTheDisconnectTimeout()
        {
            var waiter = new FakeWaiter();
            var controller = new TunnelController(new StuckConnectedCli(), new FakeBlockedSiteProbe(new FakeWindscribeCli()), waiter, EngineTimings.Default);

            var settled = await controller.DisconnectAndWaitAsync(CancellationToken.None);

            Assert.False(settled);
            Assert.Equal(EngineTimings.Default.DisconnectTimeout, waiter.TotalWaited);
        }
    }

    public class ConnectAsync
    {
        [Fact(DisplayName = "should disconnect first when a tunnel is up")]
        public async Task ShouldDisconnectFirstWhenATunnelIsUp()
        {
            var (controller, cli, _, _) = Create();
            cli.State = FakeTunnelState.Connected;

            await controller.ConnectAsync("Khalifa", "ikev2", CancellationToken.None);

            Assert.Equal(1, cli.DisconnectCalls);
            Assert.Equal(FakeTunnelState.Connected, cli.State);
        }

        [Fact(DisplayName = "should not disconnect when the CLI reports an error")]
        public async Task ShouldNotDisconnectWhenTheCliReportsAnError()
        {
            var (controller, cli, _, _) = Create();
            cli.State = FakeTunnelState.Error;

            await controller.ConnectAsync("Khalifa", "ikev2", CancellationToken.None);

            Assert.Equal(0, cli.DisconnectCalls);
        }

        [Fact(DisplayName = "should stop polling once connected and then settle")]
        public async Task ShouldStopPollingOnceConnectedAndThenSettle()
        {
            var (controller, _, _, waiter) = Create();

            await controller.ConnectAsync("Khalifa", "ikev2", CancellationToken.None);

            Assert.Equal(EngineTimings.Default.StatusPollInterval + EngineTimings.Default.SettleDelay, waiter.TotalWaited);
        }

        [Fact(DisplayName = "should poll for the connect timeout when the tunnel never comes up")]
        public async Task ShouldPollForTheConnectTimeoutWhenTheTunnelNeverComesUp()
        {
            var (controller, cli, _, waiter) = Create();
            cli.PairConnects = (_, _) => false;

            await controller.ConnectAsync("Khalifa", "ikev2", CancellationToken.None);

            Assert.Equal(EngineTimings.Default.ConnectTimeout + EngineTimings.Default.SettleDelay, waiter.TotalWaited);
        }

        [Fact(DisplayName = "should pass the nickname and protocol to the CLI")]
        public async Task ShouldPassTheNicknameAndProtocolToTheCli()
        {
            var (controller, cli, _, _) = Create();

            await controller.ConnectAsync("Stolen Cheese", "wstunnel", CancellationToken.None);

            Assert.Equal([("Stolen Cheese", "wstunnel")], cli.ConnectCalls);
        }
    }

    public class WorksAsync
    {
        [Fact(DisplayName = "should not probe when the tunnel is down")]
        public async Task ShouldNotProbeWhenTheTunnelIsDown()
        {
            var (controller, _, probe, _) = Create();

            var works = await controller.WorksAsync(CancellationToken.None);

            Assert.False(works);
            Assert.Equal(0, probe.ProbeCalls);
        }

        [Fact(DisplayName = "should work only when the blocked site loads")]
        public async Task ShouldWorkOnlyWhenTheBlockedSiteLoads()
        {
            var (controller, cli, probe, _) = Create();
            cli.State = FakeTunnelState.Connected;
            probe.LoadsThrough = _ => false;

            Assert.False(await controller.WorksAsync(CancellationToken.None));

            probe.LoadsThrough = _ => true;

            Assert.True(await controller.WorksAsync(CancellationToken.None));
        }
    }

    public class ReadStatusAsync
    {
        [Fact(DisplayName = "should parse what the CLI prints")]
        public async Task ShouldParseWhatTheCliPrints()
        {
            var (controller, cli, _, _) = Create();
            cli.State = FakeTunnelState.Connected;
            cli.ConnectedLocation = "Amsterdam - Stolen Cheese";

            var status = await controller.ReadStatusAsync(CancellationToken.None);

            Assert.Equal(new TunnelStatus(ConnectState.Connected, "Amsterdam - Stolen Cheese", "IKEv2", null), status);
        }
    }

    /// <summary>
    /// A CLI that keeps reporting Connected whatever it is told.
    /// </summary>
    private sealed class StuckConnectedCli : IWindscribeCli
    {
        public Task<string> ReadStatusAsync(CancellationToken cancellationToken) =>
            Task.FromResult("Connect state: Connected: Dubai - Khalifa\nProtocol: IKEv2:500\n");

        public Task ConnectAsync(string nickname, string protocol, CancellationToken cancellationToken) => Task.CompletedTask;

        public Task DisconnectAsync(CancellationToken cancellationToken) => Task.CompletedTask;
    }
}
