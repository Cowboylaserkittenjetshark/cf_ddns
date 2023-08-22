mod cloudflare;
mod fetchers;

use cloudflare::Cloudflare;
use fetchers::Fetchers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    fetchers: Fetchers,
    cloudflare: Cloudflare,
}

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let v4 = config.fetchers.fetch_v4()?;
    let v6 = config.fetchers.fetch_v6()?;
    config.cloudflare.update(v4, v6)
}
