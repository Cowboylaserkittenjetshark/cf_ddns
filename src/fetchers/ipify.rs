use super::{Error, Fetch};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Serialize, Deserialize)]
struct Ipify;

#[derive(Deserialize)]
struct IpifyResponse {
    ip: IpAddr,
}

#[typetag::serde]
impl Fetch for Ipify {
    fn fetch(&self) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), Error> {
        let url = "https://api64.ipify.org?format=json";
        let response = blocking::get(url)?;
        if response.status().is_success() {
            let result: IpifyResponse = response.json()?;
            match result.ip {
                IpAddr::V4(addr) => Ok((Some(addr), None)),
                IpAddr::V6(addr) => Ok((None, Some(addr))),
            }
        } else {
            return Err(Error::HttpError {
                code: response.status().as_u16(),
            });
        }
    }
}
