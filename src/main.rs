use std::process;

use cf_ddns::Config;
use clap::Parser;

fn main() {
    let args = Args::parse();
    let config = get_config(&args.config).unwrap_or_else(|err| {
        eprintln!("Problem reading configuration file: {err}");
        process::exit(1);
    });
    let results = cf_ddns::run(config).unwrap_or_else(|err| {
        eprintln!("Problem updating records: {err}");
        process::exit(1);
    });
    for result in results {
        match result {
            Ok(res) => {
                for rec in res {
                    println!("{} successfully updated", rec.name)
                }
            }
            Err(err) => println!("{}", err),
        };
    }
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
