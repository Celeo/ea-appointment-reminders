use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, TimeDelta, TimeZone, Utc};
use clap::Parser;
use itertools::Itertools;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    thread::sleep,
    time::Duration,
};

const DEFAULT_CONFIG_FILE_NAME: &str = "reminders_config.toml";

/// Easy!Appointments appointment reminders.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Load the config from a specific file.
    ///
    /// [default: reminders_config.toml]
    #[arg(long)]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

/// Easy!Appointments URL and API key, and SMTP server info.
#[derive(Debug, Deserialize)]
struct Config {
    api_root: String,
    api_key: String,
    email_from: String,
    email_reply_to: String,
    email_subject: String,
    email_body: String,
    smtp_host: String,
    smtp_user: String,
    smtp_pass: String,
}

impl Config {
    fn load_config(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        Ok(config)
    }
}

/// A single appointments's information.
///
/// There are additional fields in the API that aren't useful here.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Appointment {
    id: u32,
    start: String,
    customer_id: u32,
}

impl Appointment {
    /// Parse the `String` timestamp into a `chrono::DateTime` struct.
    ///
    /// The timestamp is parsed without a timezone and then interpreted as Utc, as
    /// the timestamp from the API does not include a timezone.
    fn start_date(&self) -> Result<DateTime<Utc>> {
        let naive = NaiveDateTime::parse_from_str(&self.start, "%Y-%m-%d %H:%M:%S")?;
        match Utc.from_local_datetime(&naive) {
            chrono::LocalResult::Single(t) => Ok(t),
            _ => Err(anyhow!("Could not parse datetime")),
        }
    }
}

/// A single customer's information.
///
/// There are additional fields in the API that aren't useful here.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CustomerInfo {
    id: u32,
    first_name: String,
    last_name: String,
    email: String,
}

/// Get appointments from the API.
async fn get_appointments(client: &Client, config: &Config) -> Result<Vec<Appointment>> {
    let resp = client
        .get(&format!("{}appointments", config.api_root))
        .header(
            reqwest::header::AUTHORIZATION,
            &format!("Bearer {}", config.api_key),
        )
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "Got status {} from appointments API",
            resp.status().as_u16()
        ));
    }
    let data = resp.json().await?;
    Ok(data)
}

/// Get customers from the API.
async fn get_customers(client: &Client, config: &Config) -> Result<Vec<CustomerInfo>> {
    let resp = client
        .get(&format!("{}customers", config.api_root))
        .header(
            reqwest::header::AUTHORIZATION,
            &format!("Bearer {}", config.api_key),
        )
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "Got status {} from appointments API",
            resp.status().as_u16()
        ));
    }
    let data = resp.json().await?;
    Ok(data)
}

/// Send an email to the customer to remind them of the upcoming appointment.
async fn send_notification(
    customer_info: &CustomerInfo,
    appointment_datetime: &str,
    config: &Config,
) -> Result<()> {
    let body = config
        .email_body
        .replace("%APPOINTMENT_DATETIME%", appointment_datetime)
        .replace("%FIRST_NAME%", &customer_info.first_name)
        .replace("%LAST_NAME%", &customer_info.last_name);
    let email = Message::builder()
        .from(config.email_from.parse()?)
        .reply_to(config.email_reply_to.parse()?)
        .to(customer_info.email.parse()?)
        .subject(&config.email_subject)
        .body(body)?;

    let sender = SmtpTransport::relay(&config.smtp_host)?
        .credentials(Credentials::from((&config.smtp_user, &config.smtp_pass)))
        .build();
    let result = sender.send(&email)?;
    if result.is_positive() {
        debug!("Email notification sent");
    } else {
        warn!(
            "Error response code from sending email to {}",
            customer_info.email
        );
    }

    Ok(())
}

/// Access to the Easy!Appointments instance, check for upcoming appointments, and potentially send reminders.
async fn check(config: &Config, reminders_set: &mut Vec<u32>) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("github.com/Celeo/ea-appointment-reminders")
        .build()
        .unwrap();
    let appointments = get_appointments(&client, config).await?;
    let customers = get_customers(&client, config).await?;
    let now = Utc::now();

    for appointment in appointments {
        if reminders_set.contains(&appointment.id) {
            debug!("Already notified for #{}", appointment.id);
            continue;
        }
        let date = appointment.start_date()?;
        if date <= now {
            // in the past
            continue;
        }
        if date - now > TimeDelta::days(3) {
            // more than 3 days out
            continue;
        }
        debug!("Upcoming appointment #{}", appointment.id);
        let customer = match customers.iter().find(|c| c.id == appointment.customer_id) {
            Some(c) => c,
            None => {
                error!(
                    "Could not find email for customer {}",
                    appointment.customer_id
                );
                continue;
            }
        };
        send_notification(customer, &appointment.start, config).await?;
        info!(
            "Adding appointment #{} to the list of sent reminders",
            appointment.id
        );
        reminders_set.push(appointment.id);
    }

    Ok(())
}

/// Entrypoint.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.debug {
        env::set_var("RUST_LOG", "info,ea_appointment_reminders=debug");
    } else if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    debug!("Logging configured");

    let config_location = match cli.config {
        Some(path) => path,
        None => Path::new(DEFAULT_CONFIG_FILE_NAME).to_owned(),
    };
    debug!("Loading from config file at: {}", config_location.display());
    let config = match Config::load_config(&config_location) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not load config: {e}");
            process::exit(1);
        }
    };

    let reminders_file = Path::new("reminders.txt");
    let mut reminders_set: Vec<u32> = Vec::new();
    if reminders_file.exists() {
        debug!("Reading from reminders file");
        let existing_reminders = match fs::read_to_string(reminders_file) {
            Ok(s) => s,
            Err(e) => {
                error!("Could not read from reminders.txt: {e}");
                process::exit(1);
            }
        };
        reminders_set.extend(
            existing_reminders
                .split_terminator('\n')
                .map(|line| line.parse::<u32>().expect("Could not parse to int")),
        );
        info!(
            "Loaded {} existing reminder IDs from file",
            reminders_set.len()
        );
    }

    loop {
        info!("Checking for reminders");
        if let Err(e) = check(&config, &mut reminders_set).await {
            error!("Error processing potential reminders: {e}");
        };
        if let Err(e) = fs::write(
            reminders_file,
            reminders_set.iter().map(|id| id.to_string()).join("\n"),
        ) {
            error!("Error writing to 'reminders.txt': {e}");
        }
        debug!("Sleeping for 1 hour");
        sleep(Duration::from_secs(60 * 60));
    }
}
