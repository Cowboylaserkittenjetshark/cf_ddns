use super::{FetchV4, FetchV6};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Disabled {}

#[typetag::serde]
impl FetchV4 for Disabled {
    fn fetch_v4(&self) -> Result<Option<std::net::Ipv4Addr>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

#[typetag::serde]
impl FetchV6 for Disabled {
    fn fetch_v6(&self) -> Result<Option<std::net::Ipv6Addr>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}
