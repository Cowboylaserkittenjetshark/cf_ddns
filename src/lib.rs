use cloudflare::endpoints::{dns, zone};
use cloudflare::framework::Environment;
use cloudflare::framework::{auth::Credentials, HttpApiClient, HttpApiClientConfig};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tabled::Tabled;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Config {
    token: String,
    domains: Vec<String>,
    tag: String,
    fetcher: Box<dyn Fetch>,
    ipv4: bool,
    ipv6: bool,
}

pub fn run(cfg: Config) -> Result<Vec<RecordUpdateResult>, Error> {
    let (v4_addr, v6_addr) = cfg.fetcher.fetch()?;

    let api_credentials = Credentials::UserAuthToken { token: cfg.token };
    let api_client_config = HttpApiClientConfig::default();
    let environment = Environment::Production;
    let api_client = HttpApiClient::new(api_credentials, api_client_config, environment).unwrap(); // TODO Handle error

    let endpoint = zone::ListZones {
        params: zone::ListZonesParams::default(),
    };

    let response = api_client.request(&endpoint)?;
    // TODO Handle message and error arrays from valid ListZones response. Display them?
    let mut results: Vec<RecordUpdateResult> = Vec::new();
    let filtered_zones = response
        .result
        .iter()
        .filter(|z| cfg.domains.contains(&z.name));
    for zone in filtered_zones {
        let endpoint = dns::ListDnsRecords {
            zone_identifier: &zone.id,
            params: dns::ListDnsRecordsParams::default(),
        };

        let response = api_client.request(&endpoint)?;
        let filtered_recs = response
            .result
            .into_iter()
            .filter(|r| r.comment.contains(&cfg.tag) || r.tags.contains(&cfg.tag));
        // TODO Handle message and error arrays from valid ListDnsRecords response
        for rec in filtered_recs {
            match rec.content {
                dns::DnsContent::A { content } => {
                    if let Some(addr) = v4_addr {
                        if content == addr {
                            let error = RecordUpdateResult::Err(RecordUpdateError::UpToDate(rec));
                            results.push(error);
                        } else {
                            let new_content = dns::DnsContent::A { content: addr };
                            let endpoint = dns::UpdateDnsRecord {
                                zone_identifier: &rec.zone_id,
                                identifier: &rec.id,
                                params: dns::UpdateDnsRecordParams {
                                    ttl: None,
                                    proxied: None,
                                    name: &rec.name,
                                    content: new_content,
                                },
                            };
                            let response = api_client.request(&endpoint)?;
                            results.push(RecordUpdateResult::Ok(response.result));
                        }
                    } else {
                        let error = RecordUpdateResult::Err(RecordUpdateError::NoNewAddr(rec));
                        results.push(error)
                    }
                }
                dns::DnsContent::AAAA { content } => {
                    if let Some(addr) = v6_addr {
                        if content == addr {
                            let error = RecordUpdateResult::Err(RecordUpdateError::UpToDate(rec));
                            results.push(error);
                        } else {
                            let new_content = dns::DnsContent::AAAA { content: addr };
                            let endpoint = dns::UpdateDnsRecord {
                                zone_identifier: &rec.zone_id,
                                identifier: &rec.id,
                                params: dns::UpdateDnsRecordParams {
                                    ttl: None,
                                    proxied: None,
                                    name: &rec.name,
                                    content: new_content,
                                },
                            };
                            let response = api_client.request(&endpoint)?;
                            results.push(RecordUpdateResult::Ok(response.result));
                        }
                    } else {
                        let error = RecordUpdateResult::Err(RecordUpdateError::NoNewAddr(rec));
                        results.push(error)
                    }
                }
                _ => {
                    let error = RecordUpdateResult::Err(RecordUpdateError::IncompatibleType(rec));
                    results.push(error);
                }
            }
        }
    }
    Ok(results)
}

#[derive(Debug, Tabled)]
pub enum RecordUpdateResult {
    Ok(dns::DnsRecord),
    Err(RecordUpdateError),
}

#[derive(Error, Debug)]
pub enum RecordUpdateError {
    #[error("Record up-to-date, skipped")]
    UpToDate(dns::DnsRecord),
    #[error("Record locked, skipped")]
    Locked(dns::DnsRecord),
    #[error("Did not fetch an ip address appropriate for record type, skipped")]
    NoNewAddr(dns::DnsRecord),
    #[error("Cannot update this type of record, skipped")]
    IncompatibleType(dns::DnsRecord),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic error")]
    Generic,
    #[error("Error with cloudflare api: {source}")]
    Cloudflare {
        #[from]
        source: cloudflare::framework::response::ApiFailure,
    },
    #[error("Error fetching new ip addr: {source}")]
    FetchFailure {
        #[from]
        source: FetcherError,
    },
}

#[derive(Error, Debug)]
pub enum FetcherError {
    #[error("Reqwest returned an error connecting to a fetcher API: {source}")]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("API returned HTTP error {code}")]
    HttpError { code: u16 },
}

#[typetag::serde(tag = "fetcher")]
trait Fetch {
    fn fetch(&self) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), FetcherError>;
}

#[derive(Serialize, Deserialize)]
struct Ipify;

#[derive(Deserialize)]
struct IpifyResponse {
    ip: IpAddr,
}

#[typetag::serde]
impl Fetch for Ipify {
    fn fetch(&self) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), FetcherError> {
        let url = "https://api64.ipify.org?format=json";
        let response = blocking::get(url)?;
        if response.status().is_success() {
            let result: IpifyResponse = response.json()?;
            match result.ip {
                IpAddr::V4(addr) => Ok((Some(addr), None)),
                IpAddr::V6(addr) => Ok((None, Some(addr))),
            }
        } else {
            return Err(FetcherError::HttpError {
                code: response.status().as_u16(),
            });
        }
    }
}
