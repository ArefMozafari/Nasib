//! One Windscribe datacenter: its city, the nickname the CLI connects by, and every address seen for it.

const LOCATION_NAME_SEPARATOR: &str = " - ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Datacenter {
    pub city: String,
    pub nickname: String,
    pub addresses: Vec<String>,
}

impl Datacenter {
    /// Splits "City - Nickname" at the first separator; `None` when either part is missing.
    pub fn from_location_name(location_name: &str, addresses: Vec<String>) -> Option<Datacenter> {
        let (city, nickname) = location_name.split_once(LOCATION_NAME_SEPARATOR)?;
        if city.is_empty() || nickname.is_empty() {
            return None;
        }
        Some(Datacenter {
            city: city.to_string(),
            nickname: nickname.to_string(),
            addresses,
        })
    }

    /// The name Windscribe shows, such as "Amsterdam - Stolen Cheese".
    pub fn location_name(&self) -> String {
        format!("{}{LOCATION_NAME_SEPARATOR}{}", self.city, self.nickname)
    }

    /// Returns this datacenter with the given addresses added after its own, without duplicates.
    pub fn with_addresses(&self, more: &[String]) -> Datacenter {
        let mut addresses = self.addresses.clone();
        for address in more {
            if !addresses.contains(address) {
                addresses.push(address.clone());
            }
        }
        Datacenter {
            addresses,
            ..self.clone()
        }
    }
}
