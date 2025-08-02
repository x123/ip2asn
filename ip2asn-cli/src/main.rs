use clap::{Parser, Subcommand};
use ip2asn::Builder;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::path::PathBuf;

const DATA_URL: &str = "https://iptoasn.com/data/ip2asn-combined.tsv.gz";

/// A high-performance CLI for mapping IP addresses to AS information.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Look up IP addresses.
    Lookup(LookupArgs),
    /// Download the latest IP-to-ASN dataset.
    Update,
}

#[derive(Parser, Debug)]
struct LookupArgs {
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

#[derive(Serialize)]
struct JsonOutput {
    ip: String,
    found: bool,
    #[serde(flatten)]
    info: Option<ip2asn::AsnInfo>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Lookup(args) => run_lookup(args),
        Commands::Update => run_update(),
    }
}

fn run_lookup(args: LookupArgs) -> Result<(), Box<dyn Error>> {
    let map = Builder::new().from_path(&args.data)?.build()?;
    if !args.ips.is_empty() {
        for ip in &args.ips {
            perform_lookup(&map, &ip.to_string(), args.json);
        }
    } else {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let ip_str = line?;
            perform_lookup(&map, &ip_str, args.json);
        }
    }
    Ok(())
}

fn run_update() -> Result<(), Box<dyn Error>> {
    let dirs = directories::ProjectDirs::from("io", "github", "x123")
        .ok_or("Could not determine cache directory")?;
    let cache_dir = dirs.cache_dir().join("ip2asn");
    fs::create_dir_all(&cache_dir)?;
    let data_path = cache_dir.join("data.tsv.gz");

    println!("Downloading dataset to {}...", data_path.display());

    let response = reqwest::blocking::get(DATA_URL)?;
    response.error_for_status_ref()?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

    let pb = indicatif::ProgressBar::new(total_size);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));

    let mut file = fs::File::create(&data_path)?;
    let mut reader = pb.wrap_read(response);
    io::copy(&mut reader, &mut file)?;

    pb.finish_with_message("Download complete");
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
