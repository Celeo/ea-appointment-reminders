use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};
use std::{env, path::Path, thread::sleep, time::Duration};

/// Easy!Appointments appointment reminders.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// Easy!Appointments URL and API key, and SMTP server info.
#[derive(Debug)]
struct Config {
    api_root: String,
    api_key: String,
    smtp_host: String,
    smtp_user: String,
    smtp_pass: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let api_root = env::var("REMINDERS_API_ROOT")?;
        let api_key = env::var("REMINDERS_API_KEY")?;
        let smtp_host = env::var("REMINDERS_SMTP_HOST")?;
        let smtp_user = env::var("REMINDERS_SMTP_USER")?;
        let smtp_pass = env::var("REMINDERS_SMTP_PASS")?;

        Ok(Self {
            api_root,
            api_key,
            smtp_host,
            smtp_user,
            smtp_pass,
        })
    }
}

async fn send_reminders(config: &Config, reminders_set: &mut Vec<u32>) -> Result<()> {
    // TODO
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.debug {
        env::set_var("RUST_LOG", "debug");
    } else if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    debug!("Logging configured");

    if Path::new(".env").exists() {
        if let Err(e) = dotenv::dotenv() {
            error!("Error reading .env file: {e}");
        } else {
            debug!("Read from .env file");
        }
    }

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(_) => {
            error!("Missing env vars");
            std::process::exit(1);
        }
    };

    let mut reminders_set: Vec<u32> = Vec::new();
    loop {
        info!("Checking for reminders");
        if let Err(e) = send_reminders(&config, &mut reminders_set).await {
            error!("Error processing potential reminders: {e}");
        };
        debug!("Sleeping for 1 hour");
        sleep(Duration::from_secs(60 * 60 * 1));
    }
}
