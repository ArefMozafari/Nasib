//! Orders datacenters, and each protocol's ports, by how often they worked recently, so the search
//! follows what the network allows now. Protocols are not ranked; they keep the order set in Settings.

use std::collections::HashMap;

use crate::attempt_log::AttemptRecord;

/// Only this many of the most recent attempts count towards a success rate.
pub const RANK_WINDOW_LINES: usize = 300;

/// Rates are compared in thousandths, as ws-find.sh printed them with three decimal places.
const RATE_SCALE: f64 = 1000.0;

/// Returns the names best first. Names with equal rates, including ones never tried, keep the order
/// they were given in.
pub fn rank(names: &[String], recent: &[AttemptRecord]) -> Vec<String> {
    let mut totals: HashMap<&str, (u32, u32)> = HashMap::new();
    for attempt in recent {
        let entry = totals.entry(attempt.nickname.as_str()).or_default();
        entry.0 += u32::from(attempt.succeeded);
        entry.1 += 1;
    }
    let rate_of = |name: &str| -> i64 {
        totals
            .get(name)
            .map(|(succeeded, total)| {
                (f64::from(*succeeded) / f64::from(*total) * RATE_SCALE).round() as i64
            })
            .unwrap_or(0)
    };
    let mut ranked: Vec<(i64, &String)> = names.iter().map(|name| (rate_of(name), name)).collect();
    // A stable sort keeps the given order among equal rates.
    ranked.sort_by_key(|(rate, _)| std::cmp::Reverse(*rate));
    ranked.into_iter().map(|(_, name)| name.clone()).collect()
}

/// The ports to try for one protocol: the preferred one in the first pass, the others in the second.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortOrder {
    /// `None` when the CLI lists no ports, so Windscribe picks one.
    pub preferred: Option<u16>,
    pub others: Vec<u16>,
}

/// One port's record on this network.
struct PortHistory {
    port: u16,
    succeeded: u32,
    total: u32,
}

impl PortHistory {
    fn rate(&self) -> i64 {
        (f64::from(self.succeeded) / f64::from(self.total) * RATE_SCALE).round() as i64
    }
}

/// Orders one protocol's ports, given default first as the CLI lists them. The preferred port is the one
/// that has worked best here, or the default without any success. The others come in three groups: ports
/// that have worked, best first; ports never tried, in the given order; and ports that only failed, least
/// tried first, since an untried port is more promising than one that has only failed. Attempts logged
/// without a port count as the default port, which is what Windscribe used for them.
pub fn rank_ports(protocol: &str, ports: &[u16], recent: &[AttemptRecord]) -> PortOrder {
    let Some(&default_port) = ports.first() else {
        return PortOrder {
            preferred: None,
            others: Vec::new(),
        };
    };
    let mut histories: Vec<PortHistory> = ports
        .iter()
        .map(|&port| PortHistory {
            port,
            succeeded: 0,
            total: 0,
        })
        .collect();
    for attempt in recent.iter().filter(|attempt| attempt.protocol == protocol) {
        let port = attempt.port.unwrap_or(default_port);
        if let Some(history) = histories.iter_mut().find(|history| history.port == port) {
            history.succeeded += u32::from(attempt.succeeded);
            history.total += 1;
        }
    }
    let (mut worked, rest): (Vec<PortHistory>, Vec<PortHistory>) = histories
        .into_iter()
        .partition(|history| history.succeeded > 0);
    let (untried, mut failed): (Vec<PortHistory>, Vec<PortHistory>) =
        rest.into_iter().partition(|history| history.total == 0);
    // Stable sorts keep the given order among equals.
    worked.sort_by_key(|history| std::cmp::Reverse(history.rate()));
    failed.sort_by_key(|history| history.total);
    let preferred = worked
        .first()
        .map(|history| history.port)
        .unwrap_or(default_port);
    let others = worked
        .iter()
        .chain(&untried)
        .chain(&failed)
        .map(|history| history.port)
        .filter(|port| *port != preferred)
        .collect();
    PortOrder {
        preferred: Some(preferred),
        others,
    }
}
