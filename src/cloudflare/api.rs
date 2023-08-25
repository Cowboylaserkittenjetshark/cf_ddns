pub mod dns;
mod zone;

use reqwest::blocking::Client;
use serde::Deserialize;
use thiserror::Error;
use zone::Zone;

#[derive(Deserialize)]
pub struct Response<T> {
    // #[serde(default = "Vec::new")]
    pub result: Option<Vec<T>>,
    pub errors: Vec<CloudflareError>,
    pub success: bool,
}

#[derive(Deserialize, Error, Debug)]
#[error("Cloudflare API Error: {message} (Code {code})")]
pub struct CloudflareError {
    code: i32,
    message: String,
}

pub fn get_zones(client: &Client) -> Result<Vec<Zone>, ListZonesError> {
    let url = "https://api.cloudflare.com/client/v4/zones";
    let response: Response<Zone> = serde_json::from_str(&client.get(url).send()?.text()?)?;
    if response.success {
        Ok(response
            .result
            .expect("Should always be some when `success` is `true`"))
    } else {
        Err(ListZonesError::Cloudflare(response.errors))
    }
}
#[derive(thiserror::Error, Debug)]
pub enum ListZonesError {
    #[error("cloudflare returned one or more errors: {0:#?}")]
    Cloudflare(Vec<CloudflareError>),
    #[error("error sending GET request to fetch zone list: {source}")]
    Send {
        #[from]
        source: reqwest::Error,
    },
    #[error("error deserializing response body: {source}")]
    Malformed {
        #[from]
        source: serde_json::Error,
    },
}
