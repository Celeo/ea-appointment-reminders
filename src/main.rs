use clap::Parser;
use log::{debug, error};
use std::env;

const EASY_APPOINTMENTS_API_KEY: &str = "EASY_APPOINTMENTS_API_KEY";

/// Easy!Appointments appointment reminders.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.debug {
        env::set_var("RUST_LOG", "info,tracing::span=warn,vzdv=debug");
    } else if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "tracing::span=warn,info");
    }
    pretty_env_logger::init();
    debug!("Logging configured");

    let api_key = match env::var(EASY_APPOINTMENTS_API_KEY) {
        Ok(s) => {
            debug!("API key env var found");
            s
        }
        Err(_) => {
            error!("No API key provided via the \"{EASY_APPOINTMENTS_API_KEY}\" env var");
            std::process::exit(1);
        }
    };

    // TODO
}
