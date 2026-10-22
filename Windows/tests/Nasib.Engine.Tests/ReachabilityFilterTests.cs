using Nasib.Engine.Tests.Fakes;

namespace Nasib.Engine.Tests;

public class ReachabilityFilterTests
{
    public class ReachableAsync
    {
        private static readonly IReadOnlyList<Datacenter> Datacenters =
        [
            new("Istanbul", "Lygos", ["203.0.113.1", "203.0.113.2"]),
            new("Dubai", "Khalifa", ["203.0.113.3"]),
            new("Sofia", "Vitoshka", ["203.0.113.4"]),
        ];

        [Fact(DisplayName = "should keep datacenters with at least one answering address in the given order")]
        public async Task ShouldKeepDatacentersWithAtLeastOneAnsweringAddressInTheGivenOrder()
        {
            var probe = new FakeReachabilityProbe(new HashSet<string> { "203.0.113.2", "203.0.113.4" });
            var filter = new ReachabilityFilter(probe, EngineTimings.Default);

            var reachable = await filter.ReachableAsync(Datacenters, CancellationToken.None);

            Assert.Equal(["Lygos", "Vitoshka"], reachable.Select(datacenter => datacenter.Nickname));
        }

        [Fact(DisplayName = "should knock on every address on port 443")]
        public async Task ShouldKnockOnEveryAddressOnPort443()
        {
            var probe = new FakeReachabilityProbe();
            var filter = new ReachabilityFilter(probe, EngineTimings.Default);

            await filter.ReachableAsync(Datacenters, CancellationToken.None);

            Assert.Equal(4, probe.Knocks.Count);
            Assert.All(probe.Knocks, knock => Assert.Equal(ReachabilityFilter.ReachPort, knock.Port));
        }
    }
}
