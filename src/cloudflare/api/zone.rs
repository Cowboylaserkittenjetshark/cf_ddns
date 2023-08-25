use super::dns::Record;
use super::Response;
use reqwest::blocking::Client;
use serde::Deserialize;

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
            Ok(response
                .result
                .expect("Should always be some when `success` is `true`"))
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
