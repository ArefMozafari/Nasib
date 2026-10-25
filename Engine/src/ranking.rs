//! Orders datacenters by how often they worked recently, so the search follows what the network allows
//! now. Protocols are not ranked; they keep the order set in Settings.

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
