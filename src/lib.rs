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

    let mut results: Vec<RecordUpdateResult> = Vec::new();

    let endpoint = zone::ListZones {
        params: zone::ListZonesParams::default(),
    };
    let response = api_client.request(&endpoint)?;
    for zone in response.result {
        let endpoint = dns::ListDnsRecords {
            zone_identifier: &zone.id,
            params: dns::ListDnsRecordsParams::default(),
        };
        let response = api_client.request(&endpoint)?;
        let filtered_recs = response.result.into_iter().filter(|r| {
            r.comment.as_ref().is_some_and(|x| x.contains(&cfg.tag)) || r.tags.contains(&cfg.tag)
        });
        for rec in filtered_recs {
            let new_content = match rec.content {
                dns::DnsContent::A { content } => match v4_addr {
                    Some(new_addr) => {
                        if content == new_addr {
                            results.push(Err(RecordUpdateError::UpToDate(rec)));
                            continue;
                        }
                        dns::DnsContent::A { content: new_addr }
                    }
                    None => {
                        results.push(Err(RecordUpdateError::NoNewAddr(rec)));
                        continue;
                    }
                },
                dns::DnsContent::AAAA { content } => match v6_addr {
                    Some(new_addr) => {
                        if content == new_addr {
                            results.push(Err(RecordUpdateError::UpToDate(rec)));
                            continue;
                        }
                        dns::DnsContent::AAAA { content: new_addr }
                    }
                    None => {
                        results.push(Err(RecordUpdateError::NoNewAddr(rec)));
                        continue;
                    }
                },
                _ => continue,
            };
            let endpoint = dns::UpdateDnsRecord {
                zone_identifier: &rec.zone_id,
                identifier: &rec.id,
                params: dns::UpdateDnsRecordParams {
                    ttl: None,
                    proxied: None,
                    name: &rec.name,
                    content: new_content,
                    comment: rec.comment.as_deref(),
                    tags: &rec.tags,
                },
            };
            match api_client.request(&endpoint) {
                Ok(succ) => results.push(Ok(succ.result)),
                Err(fail) => results.push(Err(RecordUpdateError::Cloudflare(fail, rec))),
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
    #[error("Error during transaction with the cloudflare api: {0}")]
    Cloudflare(cloudflare::framework::response::ApiFailure, dns::DnsRecord),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error during transaction with the cloudflare api: {0}")]
    Cloudflare(#[from] cloudflare::framework::response::ApiFailure),
    #[error("Error fetching new ip addr: {0}")]
    FetchFailure(#[from] fetchers::Error),
    #[error("Error building api client: {0}")]
    BuildApiClient(cloudflare::framework::Error),
}
