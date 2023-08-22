mod disabled;
mod ipify;
mod nest;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fetchers {
    v4: Box<dyn FetchV4>,
    v6: Box<dyn FetchV6>,
}

impl Fetchers {
    pub fn fetch_v4(&self) -> Result<Option<std::net::Ipv4Addr>, Box<dyn std::error::Error>> {
        self.v4.fetch_v4()
    }
    pub fn fetch_v6(&self) -> Result<Option<std::net::Ipv6Addr>, Box<dyn std::error::Error>> {
        self.v6.fetch_v6()
    }
}

#[typetag::serde(tag = "type")]
trait FetchV4 {
    fn fetch_v4(&self) -> Result<Option<std::net::Ipv4Addr>, Box<dyn std::error::Error>>;
}

#[typetag::serde(tag = "type")]
trait FetchV6 {
    fn fetch_v6(&self) -> Result<Option<std::net::Ipv6Addr>, Box<dyn std::error::Error>>;
}
