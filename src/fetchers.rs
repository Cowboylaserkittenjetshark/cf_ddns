mod disabled;
mod ipify;
mod nest;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::net;

#[derive(Serialize, Deserialize)]
pub struct Fetchers {
    v4: Box<dyn FetchV4>,
    v6: Box<dyn FetchV6>,
}

pub struct IpSet {
    pub v4_addr: Option<net::Ipv4Addr>,
    pub v6_addr: Option<net::Ipv6Addr>,
}

impl Fetchers {
    pub fn fetch(&self) -> Result<IpSet, Box<dyn std::error::Error>> {
        let client = Client::new();
        Ok(IpSet {
            v4_addr: self.v4.fetch_v4(&client)?,
            v6_addr: self.v6.fetch_v6(&client)?,
        })
    }
}

#[typetag::serde(tag = "type")]
trait FetchV4 {
    fn fetch_v4(
        &self,
        client: &Client,
    ) -> Result<Option<std::net::Ipv4Addr>, Box<dyn std::error::Error>>;
}

#[typetag::serde(tag = "type")]
trait FetchV6 {
    fn fetch_v6(
        &self,
        client: &Client,
    ) -> Result<Option<std::net::Ipv6Addr>, Box<dyn std::error::Error>>;
}
