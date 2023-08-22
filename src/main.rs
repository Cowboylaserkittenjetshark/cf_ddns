use std::process;

use cf_ddns::Config;
use clap::Parser;

fn main() {
    let args = Args::parse();
    let config = get_config(&args.config).unwrap_or_else(|err| {
        eprintln!("Problem reading configuration file: {err}");
        process::exit(1);
    });
    cf_ddns::run(config).unwrap_or_else(|err| {
        eprintln!("Problem updating records: {err}");
        process::exit(1);
    })
    // let response = reqwest::blocking::Client::new()
    //     .get("https://api.cloudflare.com/client/v4/zones")
    //     .header(reqwest::header::CONTENT_TYPE, "application/json")
    //     .header(
    //         reqwest::header::AUTHORIZATION,
    //         format!("Bearer {}", config.cloudflare.token),
    //     )
    //     .send()
    //     .expect("Failure getting zone list")
    //     .text()
    //     .expect("Failed parsing body of zone list response");
    // println!("{}", response);
    // let parsed_response: ZoneListResp =
    //     serde_json::from_str(&response).expect("Failed to parse zone list response");
    // let zones_to_update = parsed_response.result.into_iter().filter(|zone| {
    //     config
    //         .cloudflare
    //         .domains
    //         .iter()
    //         .any(|name| zone.name == *name)
    // });
    // for zone in zones_to_update {
    //     let response = reqwest::blocking::Client::new()
    //         .get(format!(
    //             "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records",
    //             zone_id = zone.id
    //         ))
    //         .header(reqwest::header::CONTENT_TYPE, "application/json")
    //         .header(
    //             reqwest::header::AUTHORIZATION,
    //             format!("Bearer {token}", token = config.cloudflare.token),
    //         )
    //         .send()
    //         .expect("Failed getting dns record list for zone")
    //         .text()
    //         .expect("Failed parsing body of dns record list response");
    //     let parsed_response: DnsListResp =
    //         serde_json::from_str(&response).expect("Failed to parse dns record list response");
    //     parsed_response
    //         .result
    //         .iter()
    //         .filter(|rec| rec.record_type == "A")
    //         .for_each(|rec| println!("{:?}", rec));
    // }
}

fn get_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: String,
}
