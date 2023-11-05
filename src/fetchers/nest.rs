use super::{Error, Fetch};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Serialize, Deserialize)]
struct Nest {
    router_ip: Option<IpAddr>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouterResponse {
    wan: Wan,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Wan {
    local_ip_address: IpAddr,
}

#[typetag::serde]
impl Fetch for Nest {
    fn fetch(&self) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), Error> {
        let ip = match self.router_ip {
            Some(ip) => ip,
            None => match default_net::get_default_gateway() {
                Ok(gateway) => gateway.ip_addr,
                Err(e) => return Err(Error::Generic(e)),
            },
        };
        let url = format!("http://{}/api/v1/status", ip);
        let response = blocking::get(url)?;
        if response.status().is_success() {
            let result: RouterResponse = response.json()?;
            match result.wan.local_ip_address {
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