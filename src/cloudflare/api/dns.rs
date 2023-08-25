use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use super::{CloudflareError, Response};

#[derive(Deserialize, Debug)]
pub struct Record {
    id: String,
    zone_id: String,
    pub name: String,
    #[serde(rename(deserialize = "type"))] // Reserved keyword
    pub record_type: RecordType,
    content: String,
    locked: bool,
    comment: Option<String>,
    tags: Vec<String>,
}

impl Record {
    pub fn update(
        &self,
        client: &Client,
        ip: std::net::IpAddr,
    ) -> Result<Vec<Record>, RecordUpdateError> {
        if self.locked {
            return Err(RecordUpdateError::Locked);
        }
        if ip.to_string() == self.content {
            return Err(RecordUpdateError::UpToDate);
        }
        let url = format!(
            "https://api.cloudflare.com/client/v4
/zones/{zone_id}/dns_records/{id}",
            zone_id = self.zone_id,
            id = self.id
        );
        let body = json!({
            "content": ip,
            "name": self.name,
            "type": self.record_type,
        });
        let response: Response<Record> = client.patch(url).json(&body).send()?.json()?;
        if response.success {
            Ok(response
                .result
                .expect("Should always be some when `success` is `true`"))
        } else {
            Err(RecordUpdateError::Cloudflare(response.errors))
        }
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.comment.as_ref().is_some_and(|c| c.contains(tag)) || self.tags.iter().any(|t| t == tag)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum RecordType {
    #[default]
    A,
    AAAA,
    CAA,
    CERT,
    CNAME,
    DNSKEY,
    DS,
    HTTPS,
    LOC,
    MX,
    NAPTR,
    NS,
    PTR,
    SMIMEA,
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT,
    URI,
}

#[derive(Error, Debug)]
pub enum RecordUpdateError {
    #[error("record locked, skipping")]
    Locked,
    #[error("record already up to date, skipping")]
    UpToDate,
    #[error("cloudflare returned one or more errors: {0:?}")]
    Cloudflare(Vec<CloudflareError>),
    #[error("error sending PATCH request to update record")]
    Send {
        #[from]
        source: reqwest::Error,
    },
}
