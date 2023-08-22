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

// TODO Put all functions directly in lib. Trying to abstract them out is a pain and now just overcomplicating things.
// Construct the reqwest client, then pass it the functions.
// fn get_zones(Client)
// fn get_records(Client, Zone)
