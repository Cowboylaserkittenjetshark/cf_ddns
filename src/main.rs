use clap::Parser;
use ddns_client::{run, Config};
use std::{fs, process};
use tabled::Table;

fn main() {
    let args = Args::parse();
    let cfg = match fs::read_to_string(args.config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error accessing config file: {e}");
            process::exit(1);
        }
    };
    let cfg: Config = match toml::from_str(&cfg) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error parsing config file: {e}");
            process::exit(1);
        }
    };
    match run(cfg) {
        Ok(results) => {
            let table = Table::new(results).to_string();
            println!("{table}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///The configuration file to use
    #[arg(short, long, value_name = "FILE")]
    config: String,
}
