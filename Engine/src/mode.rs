//! What the engine was asked to do, from its command-line arguments.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Search now, trying the last pair that worked first.
    Find,
    /// Exit if the current tunnel works, otherwise search.
    Watch,
    /// Try every pair, log each result, and never stop early.
    Survey,
    /// Print only which datacenters answer right now.
    Reach,
    /// Print the protocol order, and datacenters ordered by recent success.
    Rank,
    /// Print the datacenters and their addresses.
    Locations,
    /// Print the tunnel the CLI reports, without touching it.
    Status,
}

impl Mode {
    pub fn parse(arguments: &[String]) -> Result<Mode, String> {
        match arguments {
            [] => Ok(Mode::Find),
            [flag] => match flag.as_str() {
                "--watch" => Ok(Mode::Watch),
                "--survey" => Ok(Mode::Survey),
                "--reach" => Ok(Mode::Reach),
                "--rank" => Ok(Mode::Rank),
                "--locations" => Ok(Mode::Locations),
                "--status" => Ok(Mode::Status),
                other => Err(format!("unknown option: {other}")),
            },
            _ => Err("expected at most one option".to_string()),
        }
    }
}
