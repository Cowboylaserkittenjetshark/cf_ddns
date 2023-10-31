pub mod ipify;
pub mod nest;

use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest returned an error connecting to a fetcher API: {source}")]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("API returned HTTP error {code}")]
    HttpError { code: u16 },
}

#[typetag::serde(tag = "fetcher")]
pub trait Fetch {
    fn fetch(&self) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), Error>;
}
