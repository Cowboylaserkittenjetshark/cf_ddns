use std::net::IpAddr::{self, V4, V6};

use super::FetchV4;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Nest {
    router_ip: IpAddr,
}

#[typetag::serde]
impl FetchV4 for Nest {
    fn fetch_v4(&self) -> Result<Option<std::net::Ipv4Addr>, Box<dyn std::error::Error>> {
        let url = format!("http://{}/api/v1/status", self.router_ip);
        let response: Response = Client::new().get(url).send()?.json()?;
        match response.wan.local_ip_address {
            V4(addr) => Ok(Some(addr)),
            V6(..) => Ok(None),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    wan: Wan,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Wan {
    local_ip_address: IpAddr,
}
