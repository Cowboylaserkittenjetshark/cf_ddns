use reqwest::blocking::Client;
use std::error::Error;
use std::net::{
    IpAddr::{self, V4, V6},
    Ipv4Addr, Ipv6Addr,
};

use super::{FetchV4, FetchV6};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Ipify;

#[typetag::serde]
impl FetchV4 for Ipify {
    fn fetch_v4(&self) -> Result<Option<Ipv4Addr>, Box<dyn Error>> {
        let ip = fetch()?;
        match ip {
            V4(addr) => Ok(Some(addr)),
            V6(..) => Ok(None),
        }
    }
}

#[typetag::serde]
impl FetchV6 for Ipify {
    fn fetch_v6(&self) -> Result<Option<Ipv6Addr>, Box<dyn Error>> {
        let ip = fetch()?;
        match ip {
            V6(addr) => Ok(Some(addr)),
            V4(..) => Ok(None),
        }
    }
}

fn fetch() -> Result<IpAddr, Box<dyn Error>> {
    let url = "https://api64.ipify.org?format=json";
    let response: Response = Client::new().get(url).send()?.json()?;
    Ok(response.ip)
}

#[derive(Deserialize, Serialize)]
struct Response {
    ip: IpAddr,
}
