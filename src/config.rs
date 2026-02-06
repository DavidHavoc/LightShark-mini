use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Application configuration, loadable from CLI or YAML file.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Network interface to capture on
    #[serde(default)]
    pub interface: Option<String>,

    /// API server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database path
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// Filter by port (only capture traffic on this port)
    #[serde(default)]
    pub filter_port: Option<u16>,

    /// Filter by IP (only capture traffic to/from this IP)
    #[serde(default)]
    pub filter_ip: Option<String>,

    /// Filter by protocol (TCP, UDP)
    #[serde(default)]
    pub filter_protocol: Option<String>,

    /// Connection timeout in seconds (for stale connection cleanup)
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Enable DNS resolution for IPs
    #[serde(default)]
    pub resolve_dns: bool,

    /// Quiet mode (suppress non-error logs)
    #[serde(default)]
    pub quiet: bool,
}

fn default_port() -> u16 {
    3000
}

fn default_db_path() -> String {
    "traffic.db".to_string()
}

fn default_connection_timeout() -> u64 {
    60
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interface: None,
            port: default_port(),
            db_path: default_db_path(),
            filter_port: None,
            filter_ip: None,
            filter_protocol: None,
            connection_timeout: default_connection_timeout(),
            resolve_dns: false,
            quiet: false,
        }
    }
}

impl Config {
    /// Load config from a YAML file
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Merge CLI args into config (CLI takes precedence)
    pub fn merge_cli(&mut self, cli: &CliArgs) {
        if cli.interface.is_some() {
            self.interface = cli.interface.clone();
        }
        if cli.port != 3000 {
            self.port = cli.port;
        }
        if cli.db_path != "traffic.db" {
            self.db_path = cli.db_path.clone();
        }
        if cli.filter_port.is_some() {
            self.filter_port = cli.filter_port;
        }
        if cli.filter_ip.is_some() {
            self.filter_ip = cli.filter_ip.clone();
        }
        if cli.filter_protocol.is_some() {
            self.filter_protocol = cli.filter_protocol.clone();
        }
        if cli.connection_timeout != 60 {
            self.connection_timeout = cli.connection_timeout;
        }
        if cli.resolve_dns {
            self.resolve_dns = true;
        }
        if cli.quiet {
            self.quiet = true;
        }
    }
}

use clap::Parser;

/// LightShark-mini: Lightweight network traffic analyzer
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Network interface to capture on (e.g., eth0). Auto-detects if not provided.
    #[arg(short, long)]
    pub interface: Option<String>,

    /// Port to serve the API on
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Database path
    #[arg(long, default_value = "traffic.db")]
    pub db_path: String,

    /// Path to YAML config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Filter: only capture traffic on this port
    #[arg(long)]
    pub filter_port: Option<u16>,

    /// Filter: only capture traffic to/from this IP
    #[arg(long)]
    pub filter_ip: Option<String>,

    /// Filter: only capture this protocol (TCP, UDP)
    #[arg(long)]
    pub filter_protocol: Option<String>,

    /// Connection timeout in seconds for stale cleanup
    #[arg(long, default_value_t = 60)]
    pub connection_timeout: u64,

    /// Enable DNS resolution for IPs
    #[arg(long)]
    pub resolve_dns: bool,

    /// Quiet mode (suppress non-error logs)
    #[arg(short = 'q', long)]
    pub quiet: bool,
}
