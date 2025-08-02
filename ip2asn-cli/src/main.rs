mod config;
mod error;

use clap::{Parser, Subcommand};
use error::CliError;
use ip2asn::Builder;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::debug;

fn get_data_url() -> String {
    std::env::var("IP2ASN_DATA_URL")
        .unwrap_or_else(|_| "https://iptoasn.com/data/ip2asn-combined.tsv.gz".to_string())
}

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
    /// Path to the IP-to-ASN dataset file. Defaults to the cached data file.
    #[arg(short, long)]
    data: Option<PathBuf>,

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
    info: Option<ip2asn::AsnInfo>,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Lookup(args) => run_lookup(args),
        Commands::Update => run_update(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_lookup(args: LookupArgs) -> Result<(), CliError> {
    let config = config::Config::load()?;
    let (data_path, is_default_path) = match args.data {
        Some(path) => (path, false),
        None => {
            let home_dir = home::home_dir().ok_or_else(|| {
                CliError::NotFound("Could not determine home directory".to_string())
            })?;
            let cache_dir = home_dir.join(".cache/ip2asn");
            (cache_dir.join("data.tsv.gz"), true)
        }
    };

    if is_default_path {
        check_for_updates(&config, &data_path)?;
    }

    if !data_path.exists() {
        if is_default_path {
            return Err(CliError::NotFound(
                "Dataset not found. Please run `ip2asn-cli update` to download it.".to_string(),
            ));
        } else {
            return Err(CliError::NotFound(format!(
                "Data file not found at {}",
                data_path.display()
            )));
        }
    }

    let map = Builder::new().from_path(&data_path)?.build()?;

    if !args.ips.is_empty() {
        for ip in &args.ips {
            perform_lookup(&map, &ip.to_string(), args.json)?;
        }
    } else {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let ip_str = line?;
            perform_lookup(&map, &ip_str, args.json)?;
        }
    }
    Ok(())
}

fn check_for_updates(config: &config::Config, cache_path: &PathBuf) -> Result<(), CliError> {
    debug!(?cache_path, "Checking for updates");
    if !config.auto_update {
        debug!("Auto update disabled");
        return Ok(());
    }

    if !cache_path.exists() {
        debug!("Cache file does not exist");
        eprintln!("Cache file not found. Downloading...");
        io::stderr().flush()?;
        return run_update();
    }

    let metadata = fs::metadata(cache_path)?;
    let modified_time = metadata.modified()?;
    debug!(?modified_time, "Cache file modified time");
    let now = SystemTime::now();
    let age = now
        .duration_since(modified_time)
        .map_err(|e| CliError::Update(format!("System time error: {}", e)))?;
    debug!(?age, "Cache file age");
    if age < Duration::from_secs(24 * 60 * 60) {
        debug!("Cache file is recent, skipping update check");
        return Ok(());
    }

    eprintln!("Checking for dataset updates...");
    io::stderr().flush()?;
    let client = reqwest::blocking::Client::new();
    let response = client.head(&get_data_url()).send()?;
    response.error_for_status_ref()?;

    if let Some(last_modified) = response.headers().get(reqwest::header::LAST_MODIFIED) {
        let last_modified_str = last_modified
            .to_str()
            .map_err(|e| CliError::Update(format!("Invalid Last-Modified header: {}", e)))?;
        debug!(%last_modified_str, "Remote Last-Modified header");
        let remote_mtime = httpdate::parse_http_date(last_modified_str)?;
        debug!(?remote_mtime, "Parsed remote mtime");
        if remote_mtime > modified_time {
            debug!("Remote file is newer, starting update");
            eprintln!("New dataset found. Downloading...");
            io::stderr().flush()?;
            return run_update();
        } else {
            debug!("Remote file is not newer, skipping update");
        }
    }

    Ok(())
}

fn run_update() -> Result<(), CliError> {
    let home_dir = home::home_dir()
        .ok_or_else(|| CliError::NotFound("Could not determine home directory".to_string()))?;
    let cache_dir = home_dir.join(".cache/ip2asn");
    fs::create_dir_all(&cache_dir)?;
    let data_path = cache_dir.join("data.tsv.gz");

    eprintln!("Downloading dataset to {}...", data_path.display());
    io::stderr().flush()?;

    let mut response = reqwest::blocking::get(get_data_url())?;
    response.error_for_status_ref()?;
    let total_size = response
        .content_length()
        .ok_or_else(|| CliError::Update("Failed to get content length".to_string()))?;

    if std::env::var("IP2ASN_TESTING").is_ok() {
        let mut file = fs::File::create(&data_path)?;
        io::copy(&mut response, &mut file)?;
    } else {
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .map_err(|e| CliError::Update(format!("Indicatif error: {}", e)))?
                .progress_chars("#>-"),
        );

        let mut file = fs::File::create(&data_path)?;
        let mut reader = pb.wrap_read(response);
        io::copy(&mut reader, &mut file)?;

        pb.finish_with_message("Download complete");
    }
    Ok(())
}

fn perform_lookup(map: &IpAsnMap, ip_str: &str, json: bool) -> Result<(), CliError> {
    let trimmed_ip = ip_str.trim();
    if trimmed_ip.is_empty() {
        return Ok(());
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
                println!(
                    "{}",
                    serde_json::to_string(&output).map_err(|e| CliError::InvalidInput(format!(
                        "JSON serialization error: {}",
                        e
                    )))?
                );
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
                println!(
                    "{}",
                    serde_json::to_string(&output).map_err(|e| CliError::InvalidInput(format!(
                        "JSON serialization error: {}",
                        e
                    )))?
                );
            } else {
                eprintln!("Error: Invalid IP address '{}'", trimmed_ip);
            }
        }
    }
    Ok(())
}
