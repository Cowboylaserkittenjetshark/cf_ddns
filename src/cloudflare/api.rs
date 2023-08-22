use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct Response<T> {
    #[serde(default = "Vec::new")]
    pub result: Vec<T>,
    pub errors: Vec<Error>,
    pub success: bool,
}

#[derive(Deserialize, Debug)]
pub struct Error {
    code: i32,
    message: String,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cloudflare API Error: {message} (Code {err_code})",
            message = &self.message,
            err_code = &self.code
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

impl Zone {
    pub fn get_records(&self, client: &Client) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_identifier}/dns_records",
            zone_identifier = self.id
        );
        let response: Response<Record> = client.get(url).send()?.json()?;
        if response.success {
            Ok(response.result)
        } else {
            let error = response
                .errors
                .into_iter()
                .nth(0)
                .expect("Should always contain at least one error when success == false");
            Err(error.into())
        }
    }
}

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.locked {
            eprintln!("Record locked, skipping: {}", self.name);
            return Ok(());
        }
        if ip.to_string() == self.content {
            eprintln!("Record up to date, skipping: {}", self.name);
            return Ok(());
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
        let _response = client.patch(url).json(&body).send()?;
        Ok(())
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
