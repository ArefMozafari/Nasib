//! What `windscribe-cli status` reports, for example:
//! `Connect state: Connected: Amsterdam - Stolen Cheese` and `Protocol: IKEv2:500`.

const CONNECT_STATE_PREFIX: &str = "Connect state: ";
const PROTOCOL_PREFIX: &str = "Protocol: ";
const DETAIL_SEPARATOR: &str = ": ";
const PROTOCOL_PORT_SEPARATOR: char = ':';
const PAIR_SEPARATOR: &str = " / ";
const CONNECTED_LINE_PREFIX: &str = "connected: ";
const DOWN_LINE: &str = "down";

/// The "Connect state" line of `windscribe-cli status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectState {
    Unknown,
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelStatus {
    pub state: ConnectState,
    /// The location as the CLI shows it, such as "Atlanta - Magic City", when connected.
    pub location: Option<String>,
    /// The protocol as the CLI names it, without the port, such as "IKEv2".
    pub protocol: Option<String>,
}

/// Splits "Connected: Amsterdam - Stolen Cheese" or "Error: 11" into the state and the text after it.
fn parse_connect_state(value: &str) -> (ConnectState, Option<&str>) {
    let (word, detail) = match value.split_once(DETAIL_SEPARATOR) {
        Some((word, detail)) => (word, Some(detail)),
        None => (value, None),
    };
    let state = match word {
        "Connected" => ConnectState::Connected,
        "Disconnected" => ConnectState::Disconnected,
        "Connecting" => ConnectState::Connecting,
        "Disconnecting" => ConnectState::Disconnecting,
        "Error" => ConnectState::Error,
        _ => ConnectState::Unknown,
    };
    (state, detail)
}

impl TunnelStatus {
    pub fn parse(output: &str) -> TunnelStatus {
        let mut status = TunnelStatus {
            state: ConnectState::Unknown,
            location: None,
            protocol: None,
        };
        for line in output.lines().map(str::trim) {
            if let Some(value) = line.strip_prefix(CONNECT_STATE_PREFIX) {
                let (state, detail) = parse_connect_state(value);
                status.state = state;
                status.location = if state == ConnectState::Connected {
                    detail.map(String::from)
                } else {
                    None
                };
            } else if let Some(value) = line.strip_prefix(PROTOCOL_PREFIX) {
                status.protocol = value
                    .split(PROTOCOL_PORT_SEPARATOR)
                    .next()
                    .map(String::from);
            }
        }
        status
    }

    pub fn is_connected(&self) -> bool {
        self.state == ConnectState::Connected
    }

    /// True when no tunnel is up or on its way: Disconnected, or an error the CLI keeps reporting until
    /// the next connect.
    pub fn is_down(&self) -> bool {
        matches!(self.state, ConnectState::Disconnected | ConnectState::Error)
    }

    /// The line `--status` prints: the CLI's own location and protocol when fully connected, `down` otherwise.
    pub fn status_line(&self) -> String {
        match (self.is_connected(), &self.location, &self.protocol) {
            (true, Some(location), Some(protocol)) => {
                format!("{CONNECTED_LINE_PREFIX}{location}{PAIR_SEPARATOR}{protocol}")
            }
            _ => DOWN_LINE.to_string(),
        }
    }
}
