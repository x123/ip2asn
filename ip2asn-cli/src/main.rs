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

    /// One or more IP addresses to look up.
    #[arg(required = true, name = "IPS")]
    ips: Vec<IpAddr>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Build the map from the specified data file.
    let map = Builder::new().from_path(&cli.data)?.build()?;

    // Process each IP address.
    for ip in cli.ips {
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
    Ok(())
}
