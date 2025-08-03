//! # ip2asn-cli
//!
//! A high-performance command-line interface for mapping IP addresses to Autonomous
//! System (AS) information.
//!
//! This tool provides two main functions:
//! 1.  **Lookup**: Resolves one or more IP addresses, provided either as command-line
//!     arguments or via standard input. Output can be in a human-readable format
//!     or structured JSON.
//! 2.  **Update**: Downloads the latest IP-to-ASN dataset from iptoasn.com, ensuring
//!     lookups are based on current data. The dataset is cached locally.
//!
//! The CLI is designed to be fast, efficient, and suitable for interactive use or
//! integration into automated shell scripts. It automatically handles caching and
//! periodic updates of the dataset.
//!
//! ## Usage
//!
//! ```sh
//! # Look up a single IP
//! # Look up a single IP (default command)
//! ip2asn 8.8.8.8
//! 15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US
//!
//! # Look up multiple IPs from stdin
//! cat ips.txt | ip2asn
//! 15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US
//! 13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US
//!
//! # Output in JSON format
//! ip2asn --json 1.1.1.1 | jq .
//! {
//!  "ip": "1.1.1.1",
//!  "found": true,
//!  "info": {
//!    "network": "1.1.1.0/24",
//!    "asn": 13335,
//!    "country_code": "US",
//!    "organization": "CLOUDFLARENET"
//!  }
//! }
//!
//! # Download the latest dataset
//! ip2asn update
//! ```

#![deny(missing_docs)]
mod config;
mod error;

use clap::{Args, Parser, Subcommand};
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

/// The main CLI entry point, defining the command-line interface structure.
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A high-performance CLI for mapping IP addresses to AS information."
)]
struct Cli {
    /// The command to execute. If no command is specified, `lookup` is the default.
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    lookup: LookupArgs,

    /// Path to a custom configuration file.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Path to a custom cache directory.
    #[arg(long, global = true)]
    cache_dir: Option<PathBuf>,
}

/// Defines the available subcommands for the CLI.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Looks up one or more IP addresses and prints their ASN information.
    ///
    /// IPs can be passed as arguments or piped via stdin. The command will use
    /// the cached dataset by default and automatically check if a newer version
    /// is available.
    Lookup(LookupArgs),
    /// Forces an update of the IP-to-ASN dataset.
    ///
    /// This command downloads the latest version of the dataset from
    /// iptoasn.com and stores it in the local cache directory.
    Update,
}

/// Arguments for the `lookup` subcommand.
/// Arguments for the `lookup` subcommand.
#[derive(Args, Debug)]
struct LookupArgs {
    /// Path to a custom IP-to-ASN dataset file.
    ///
    /// If not provided, the CLI will use the default cached dataset located in
    /// `$HOME/.cache/ip2asn/data.tsv.gz`.
    #[arg(short, long)]
    data: Option<PathBuf>,

    /// One or more IP addresses to look up.
    ///
    /// If no IPs are provided as arguments, the command will read them from
    /// standard input, one per line.
    #[arg(name = "IPS")]
    ips: Vec<IpAddr>,

    /// Formats the output as JSON.
    ///
    /// Each line of output will be a JSON object containing the lookup
    /// details, including the original IP, whether it was found, and the
    /// corresponding ASN information.
    #[arg(short, long)]
    json: bool,
}

use ip2asn::IpAsnMap;
use serde::Serialize;

/// Represents the structured output for a single lookup in JSON format.
#[derive(Serialize)]
struct JsonOutput {
    /// The IP address that was looked up.
    ip: String,
    /// A boolean indicating whether a record was found for the IP.
    found: bool,
    /// The ASN information, present only if a record was found.
    info: Option<ip2asn::AsnInfo>,
}

/// The main function of the application.
///
/// Parses command-line arguments, dispatches to the appropriate subcommand
/// handler, and prints any resulting errors to stderr before exiting.
fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Some(Commands::Lookup(args)) => {
            run_lookup(args, cli.config.as_deref(), cli.cache_dir.as_deref())
        }
        Some(Commands::Update) => run_update(cli.cache_dir.as_deref()),
        None => run_lookup(&cli.lookup, cli.config.as_deref(), cli.cache_dir.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Handles the `lookup` subcommand logic.
///
/// This function orchestrates the lookup process:
/// 1. Loads the configuration.
/// 2. Determines the data file path (either user-provided or default cache).
/// 3. Checks for dataset updates if using the default path.
/// 4. Builds the `IpAsnMap` from the data file.
/// 5. Reads IPs from arguments or stdin and performs the lookups.
fn run_lookup(
    args: &LookupArgs,
    config_path: Option<&std::path::Path>,
    cache_dir_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    let config = config::Config::load(config_path)?;
    let (data_path, is_default_path) = match &args.data {
        Some(path) => (path.clone(), false),
        None => {
            let cache_dir = if let Some(p) = cache_dir_path {
                p.to_path_buf()
            } else {
                let home_dir = home::home_dir().ok_or_else(|| {
                    CliError::NotFound("Could not determine home directory".to_string())
                })?;
                home_dir.join(".cache/ip2asn")
            };
            (cache_dir.join("data.tsv.gz"), true)
        }
    };

    if is_default_path {
        check_for_updates(&config, &data_path, cache_dir_path)?;
    }

    if !data_path.exists() {
        if is_default_path {
            return Err(CliError::NotFound(
                "Dataset not found. Please run `ip2asn update` to download it.".to_string(),
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

/// Checks if a newer version of the dataset is available and triggers an update.
///
/// This check is only performed if auto-updates are enabled and the cached
/// dataset is more than 24 hours old. It compares the local file's modification
/// time with the `Last-Modified` header from the remote server.
fn check_for_updates(
    config: &config::Config,
    cache_path: &PathBuf,
    cache_dir_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    debug!(?cache_path, "Checking for updates");
    if !config.auto_update {
        debug!("Auto update disabled in config, skipping check.");
        return Ok(());
    }

    if !cache_path.exists() {
        debug!("Cache file does not exist, forcing update.");
        eprintln!("Cache file not found. Downloading...");
        io::stderr().flush()?;
        return run_update(cache_dir_path);
    }

    let metadata = fs::metadata(cache_path)?;
    let modified_time = metadata.modified()?;
    debug!(?modified_time, "Cache file modified time");

    let now = SystemTime::now();
    let age = now.duration_since(modified_time).unwrap_or_default();
    debug!(?age, "Cache file age");

    // In tests, we might manipulate the file time, so the 24h check is not reliable.
    // The IP2ASN_TESTING var allows us to bypass this for testing purposes.
    if age < Duration::from_secs(24 * 60 * 60) && std::env::var("IP2ASN_TESTING").is_err() {
        debug!("Cache file is recent and not in test mode, skipping remote check.");
        return Ok(());
    }

    eprintln!("Checking for dataset updates...");
    io::stderr().flush()?;
    let client = reqwest::blocking::Client::new();
    let response = client.head(&get_data_url()).send()?;
    response.error_for_status_ref()?;

    if let Some(last_modified) = response.headers().get(reqwest::header::LAST_MODIFIED) {
        let last_modified_str = last_modified.to_str().map_err(|e| {
            CliError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Invalid Last-Modified header: {}", e),
            ))
        })?;
        let remote_mtime = httpdate::parse_http_date(last_modified_str)?;
        debug!(?remote_mtime, "Parsed remote mtime");
        if remote_mtime > modified_time {
            debug!("Remote file is newer, starting update");
            eprintln!("New dataset found. Downloading...");
            io::stderr().flush()?;
            return run_update(cache_dir_path);
        } else {
            debug!("Remote file is not newer, skipping update");
        }
    }

    Ok(())
}

/// Handles the `update` subcommand logic.
///
/// This function ensures the cache directory exists, then downloads the dataset
/// from the remote URL, displaying a progress bar during the download.
fn run_update(cache_dir_path: Option<&std::path::Path>) -> Result<(), CliError> {
    let cache_dir = if let Some(p) = cache_dir_path {
        p.to_path_buf()
    } else {
        let home_dir = home::home_dir()
            .ok_or_else(|| CliError::NotFound("Could not determine home directory".to_string()))?;
        home_dir.join(".cache/ip2asn")
    };
    fs::create_dir_all(&cache_dir)?;
    let data_path = cache_dir.join("data.tsv.gz");

    eprintln!("Downloading dataset to {}...", data_path.display());
    io::stderr().flush()?;

    let mut response = reqwest::blocking::get(get_data_url())?;
    response.error_for_status_ref()?;
    let total_size = response.content_length().ok_or_else(|| {
        CliError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get content length",
        ))
    })?;

    if std::env::var("IP2ASN_TESTING").is_ok() {
        let mut file = fs::File::create(&data_path)?;
        io::copy(&mut response, &mut file)?;
    } else {
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .map_err(|e| CliError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Indicatif error: {}", e),
                )))?
                .progress_chars("#>-"),
        );

        let mut file = fs::File::create(&data_path)?;
        let mut reader = pb.wrap_read(response);
        io::copy(&mut reader, &mut file)?;

        pb.finish_with_message("Download complete");
    }
    Ok(())
}

/// Performs a lookup for a single IP address and prints the result.
///
/// This function parses the IP string, queries the `IpAsnMap`, and prints the
/// output in either human-readable or JSON format based on the `json` flag.
/// It handles invalid IP formats gracefully.
fn perform_lookup(map: &IpAsnMap, ip_str: &str, json: bool) -> Result<(), CliError> {
    let trimmed_ip = ip_str.trim();
    if trimmed_ip.is_empty() {
        return Ok(());
    }

    if json {
        let (ip_str, result) = match trimmed_ip.parse::<IpAddr>() {
            Ok(ip) => (ip.to_string(), map.lookup_owned(ip)),
            Err(_) => (trimmed_ip.to_string(), None),
        };
        let output = JsonOutput {
            ip: ip_str,
            found: result.is_some(),
            info: result,
        };
        println!(
            "{}",
            serde_json::to_string(&output)
                .map_err(|e| CliError::InvalidInput(format!("JSON serialization error: {}", e)))?
        );
    } else {
        match trimmed_ip.parse::<IpAddr>() {
            Ok(ip) => match map.lookup(ip) {
                Some(info) => {
                    println!(
                        "{} | {} | {} | {} | {}",
                        info.asn, ip, info.network, info.organization, info.country_code
                    );
                }
                None => {
                    println!("{} | Not Found", ip);
                }
            },
            Err(_) => {
                eprintln!("Error: Invalid IP address '{}'", trimmed_ip);
            }
        }
    }
    Ok(())
}
