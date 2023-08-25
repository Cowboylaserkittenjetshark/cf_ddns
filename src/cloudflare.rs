pub mod api;

use api::dns::RecordType::{A, AAAA};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::fetchers::IpSet;

use self::api::dns::{Record, RecordUpdateError};

#[derive(Serialize, Deserialize)]
pub struct Cloudflare {
    token: String,
    tag: String,
    domains: Vec<String>,
}

impl Cloudflare {
    pub fn update(
        &self,
        ips: IpSet,
    ) -> Result<Vec<Result<Vec<Record>, RecordUpdateError>>, Box<dyn std::error::Error>> {
        let client = self.build_client()?;
        let zones = api::get_zones(&client)?.into_iter().filter(|zone| {
            self.domains
                .iter()
                .any(|domain| zone.id == *domain || zone.name == *domain)
        });
        let mut results: Vec<Result<Vec<Record>, RecordUpdateError>> = Vec::with_capacity(0);
        for zone in zones {
            let records = zone.get_records(&client)?;
            let v4_records = records
                .iter()
                .filter(|r| matches!(r.record_type, A))
                .filter(|r| r.has_tag(&self.tag));
            if let Some(ip) = ips.v4_addr {
                for record in v4_records {
                    results.push(record.update(&client, ip.into()));
                }
            }
            let v6_records = records
                .iter()
                .filter(|r| matches!(r.record_type, AAAA))
                .filter(|r| r.has_tag(&self.tag));
            if let Some(ip) = ips.v6_addr {
                for record in v6_records {
                    results.push(record.update(&client, ip.into()));
                }
            }
        }
        Ok(results)
    }

    fn build_client(&self) -> Result<Client, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", &self.token)
                .parse()
                .expect("No sane token should fail to parse"),
        );
        Client::builder().default_headers(headers).build()
    }
}
