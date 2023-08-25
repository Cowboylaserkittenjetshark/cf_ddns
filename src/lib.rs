mod cloudflare;
mod fetchers;

use cloudflare::api::dns::{Record, RecordUpdateError};
use cloudflare::Cloudflare;
use fetchers::Fetchers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    fetchers: Fetchers,
    cloudflare: Cloudflare,
}

pub fn run(
    config: Config,
) -> Result<Vec<Result<Vec<Record>, RecordUpdateError>>, Box<dyn std::error::Error>> {
    let ips = config.fetchers.fetch()?;
    config.cloudflare.update(ips)
}
