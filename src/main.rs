use clap::Parser;
use ddns_client::{run, Config};
use std::{fs, process};

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
            let mut builder = tabled::builder::Builder::default();
            builder.set_header(["Name", "Status"]);
            for res in results {
                match res {
                    Ok(rec) => {
                        builder.push_record([&rec.name, "Updated"]);
                    }
                    Err(err) => match err {
                        ddns_client::RecordUpdateError::UpToDate(rec) => {
                            builder.push_record([&rec.name, "Skipped: Up to date"]);
                        }
                        ddns_client::RecordUpdateError::Locked(rec) => {
                            builder
                                .push_record([&rec.name, "Skipped: Record locked, cannot update"]);
                        }
                        ddns_client::RecordUpdateError::NoNewAddr(rec) => {
                            builder.push_record([
                                &rec.name,
                                "Skipped: No fetched ip coresponds to record type",
                            ]);
                        }
                        ddns_client::RecordUpdateError::Cloudflare(fail, rec) => {
                            builder.push_record([rec.name, format!("Failed: {fail}")]);
                        }
                    },
                }
            }
            let mut table = builder.build();
            table.with(tabled::settings::Style::ascii_rounded());
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
