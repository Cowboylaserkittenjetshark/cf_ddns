mod fetchers;

use cloudflare::endpoints::{dns, zone};
use cloudflare::framework::Environment;
use cloudflare::framework::{auth::Credentials, HttpApiClient, HttpApiClientConfig};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Config {
    token: String,
    tag: String,
    fetcher: Box<dyn fetchers::Fetch>,
}

pub fn run(cfg: Config) -> Result<Vec<RecordUpdateResult>> {
    let (v4_addr, v6_addr) = cfg.fetcher.fetch()?;

    let api_credentials = Credentials::UserAuthToken { token: cfg.token };
    let api_client_config = HttpApiClientConfig::default();
    let environment = Environment::Production;
    let api_client = match HttpApiClient::new(api_credentials, api_client_config, environment) {
        Ok(client) => client,
        Err(err) => return Err(Error::BuildApiClient(err)),
    };

    let endpoint = zone::ListZones {
        params: zone::ListZonesParams::default(),
    };

    let response = api_client.request(&endpoint)?;
    // TODO Handle message and error arrays from valid ListZones response. Display them?
    let mut results: Vec<RecordUpdateResult> = Vec::new();
    for zone in response.result {
        let endpoint = dns::ListDnsRecords {
            zone_identifier: &zone.id,
            params: dns::ListDnsRecordsParams::default(),
        };

        let response = api_client.request(&endpoint)?;
        let filtered_recs = response.result.into_iter().filter(|r| {
            r.comment.as_ref().is_some_and(|x| x.contains(&cfg.tag)) || r.tags.contains(&cfg.tag)
        });
        // TODO Handle message and error arrays from valid ListDnsRecords response
        for rec in filtered_recs {
            match rec.content {
                // TODO Deduplicate for these match arms. Only the new_content needs to be created in the match
                dns::DnsContent::A { content } => {
                    if let Some(addr) = v4_addr {
                        if content == addr {
                            results.push(Err(RecordUpdateError::UpToDate(rec)));
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
                            results.push(Ok(response.result));
                        }
                    } else {
                        results.push(Err(RecordUpdateError::NoNewAddr(rec)));
                    }
                }
                dns::DnsContent::AAAA { content } => {
                    if let Some(addr) = v6_addr {
                        if content == addr {
                            results.push(Err(RecordUpdateError::UpToDate(rec)));
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
                            let response = api_client.request(&endpoint)?; // TODO Should it really bail when record fails to update? Could just return a RecordUpdateError
                            results.push(Ok(response.result));
                        }
                    } else {
                        results.push(Err(RecordUpdateError::NoNewAddr(rec)));
                    }
                }
                _ => {
                    results.push(Err(RecordUpdateError::IncompatibleType(rec)));
                    // TODO Should be filtered out instead of error-ing
                }
            }
        }
    }
    Ok(results)
}

pub type Result<T> = std::result::Result<T, Error>;

pub type RecordUpdateResult =
    std::result::Result<cloudflare::endpoints::dns::DnsRecord, RecordUpdateError>;

#[derive(Error, Debug)]
pub enum RecordUpdateError {
    #[error("Record up-to-date, skipped")]
    UpToDate(dns::DnsRecord),
    #[error("Record locked, skipped")]
    Locked(dns::DnsRecord),
    #[error("Did not fetch an ip address appropriate for record type, skipped")]
    NoNewAddr(dns::DnsRecord),
    #[error("Cannot update this type of record, skipped")] // TODO get rid of this, filter them out
    IncompatibleType(dns::DnsRecord),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error with cloudflare api: {0}")]
    Cloudflare(#[from] cloudflare::framework::response::ApiFailure),
    #[error("Error fetching new ip addr: {0}")]
    FetchFailure(#[from] fetchers::Error),
    #[error("Error building api client: {0}")]
    BuildApiClient(cloudflare::framework::Error),
}
