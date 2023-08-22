mod api;

use std::net::{Ipv4Addr, Ipv6Addr};

use api::{
    RecordType::{A, AAAA},
    Response, Zone,
};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cloudflare {
    token: String,
    tag: String,
    domains: Vec<String>,
}

impl Cloudflare {
    pub fn update(
        &self,
        v4: Option<Ipv4Addr>,
        v6: Option<Ipv6Addr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.build_client()?;
        let zones = self.get_zones(&client)?.into_iter().filter(|zone| {
            self.domains
                .iter()
                .any(|domain| zone.id == *domain || zone.name == *domain)
        });
        for zone in zones {
            let records = zone.get_records(&client)?;
            let v4_records = records
                .iter()
                .filter(|r| matches!(r.record_type, A))
                .filter(|r| r.has_tag(&self.tag));
            match v4 {
                Some(ip) => {
                    for record in v4_records {
                        record.update(&client, ip.into())?;
                    }
                }
                None => {
                    for record in v4_records {
                        eprintln!("No IPV4 address fetched. Skipping {}", record.name);
                    }
                }
            }
            let v6_records = records
                .iter()
                .filter(|r| matches!(r.record_type, AAAA))
                .filter(|r| r.has_tag(&self.tag));
            match v6 {
                Some(ip) => {
                    for record in v6_records {
                        record.update(&client, ip.into())?;
                    }
                }
                None => {
                    for record in v6_records {
                        eprintln!("No IPV6 address fetched. Skipping {}", record.name);
                    }
                }
            }
        }
        Ok(())
    }

    fn get_zones(&self, client: &Client) -> Result<Vec<Zone>, Box<dyn std::error::Error>> {
        let url = "https://api.cloudflare.com/client/v4/zones";
        let response: Response<Zone> = client.get(url).send()?.json()?;
        if response.success {
            Ok(response.result)
        } else {
            let error = response
                .errors
                .into_iter()
                .next()
                .expect("Should always contain at least one error when success == false");
            Err(error.into())
        }
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
