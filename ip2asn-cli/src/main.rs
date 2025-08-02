use clap::Parser;
use ip2asn::Builder;
use std::error::Error;
use std::net::IpAddr;
use std::path::PathBuf;

/// A high-performance CLI for mapping IP addresses to AS information.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the IP-to-ASN dataset file (e.g., ip2asn-combined.tsv.gz).
    #[arg(short, long, required = true)]
    data: PathBuf,

    /// One or more IP addresses to look up. If not provided, reads from stdin.
    #[arg(name = "IPS")]
    ips: Vec<IpAddr>,

    /// Output results in JSON format.
    #[arg(short, long)]
    json: bool,
}

use ip2asn::IpAsnMap;
use serde::Serialize;
use std::io::{self, BufRead};

#[derive(Serialize)]
struct JsonOutput {
    ip: String,
    found: bool,
    #[serde(flatten)]
    info: Option<ip2asn::AsnInfo>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let map = Builder::new().from_path(&cli.data)?.build()?;

    if !cli.ips.is_empty() {
        for ip in &cli.ips {
            perform_lookup(&map, &ip.to_string(), cli.json);
        }
    } else {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let ip_str = line?;
            perform_lookup(&map, &ip_str, cli.json);
        }
    }

    Ok(())
}

fn perform_lookup(map: &IpAsnMap, ip_str: &str, json: bool) {
    let trimmed_ip = ip_str.trim();
    if trimmed_ip.is_empty() {
        return;
    }
    match trimmed_ip.parse::<IpAddr>() {
        Ok(ip) => {
            if json {
                let result = map.lookup_owned(ip);
                let output = JsonOutput {
                    ip: ip.to_string(),
                    found: result.is_some(),
                    info: result,
                };
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                match map.lookup(ip) {
                    Some(info) => {
                        println!(
                            "{} | {} | {} | {} | {}",
                            info.asn, ip, info.network, info.organization, info.country_code
                        );
                    }
                    None => {
                        println!("{} | Not Found", ip);
                    }
                }
            }
        }
        Err(_) => {
            if json {
                let output = JsonOutput {
                    ip: trimmed_ip.to_string(),
                    found: false,
                    info: None,
                };
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                eprintln!("Error: Invalid IP address '{}'", trimmed_ip);
            }
        }
    }
}
