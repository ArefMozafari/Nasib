//! Requests a page that answers 204 No Content and counts only that answer. Windscribe reporting
//! Connected is not enough on a censored network; this is what proves traffic gets through.

use std::time::Duration;

use ureq::Agent;

/// The page every app has always checked.
pub const DEFAULT_PROBE_URL: &str = "https://www.youtube.com/generate_204";
const NO_CONTENT_STATUS: u16 = 204;

pub trait BlockedSiteProbe: Send + Sync {
    /// True only when the page answers 204 within the probe's timeout.
    fn loads(&self) -> bool;
}

pub struct HttpBlockedSiteProbe {
    url: String,
    timeout: Duration,
}

impl HttpBlockedSiteProbe {
    pub fn new(url: String, timeout: Duration) -> HttpBlockedSiteProbe {
        HttpBlockedSiteProbe { url, timeout }
    }
}

impl BlockedSiteProbe for HttpBlockedSiteProbe {
    fn loads(&self) -> bool {
        // A fresh agent per probe keeps a connection opened through the previous tunnel from being reused,
        // and the proxy stays off so a system proxy such as v2rayN cannot answer in the tunnel's place.
        let agent: Agent = Agent::config_builder()
            .proxy(None)
            .timeout_global(Some(self.timeout))
            .http_status_as_error(false)
            .build()
            .into();
        agent
            .get(&self.url)
            .call()
            .is_ok_and(|response| response.status().as_u16() == NO_CONTENT_STATUS)
    }
}
