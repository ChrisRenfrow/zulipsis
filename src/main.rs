use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use crossbeam_channel::{bounded, select, tick, Receiver};
use rand::Rng;
use std::{
    fs,
    time::{Duration, Instant},
};

mod config;
mod zulip_status;
mod zuliprc;

use config::Config;
use zulip_status::{Emoji, ZulipStatus};
use zuliprc::ZulipRc;

#[derive(Parser)]
struct Cli {
    /// The path to zuliprc
    #[arg(short, long)]
    zuliprc: String,
    /// The path to the config
    #[arg(short, long)]
    config: String,
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn pick_one(rng: &mut impl Rng, list: &[String]) -> String {
    list[rng.gen_range(0..list.len())].to_string()
}

fn timestamp() -> String {
    let current_datetime: DateTime<Local> = Local::now();
    current_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let zuliprc: ZulipRc = toml::from_str(&fs::read_to_string(args.zuliprc)?)?;
    let config: Config = toml::from_str(&fs::read_to_string(args.config)?)?;
    let mut rng = rand::thread_rng();

    let email = zuliprc.api.email;
    let key = zuliprc.api.key;
    let status = ZulipStatus::new(zuliprc.api.site, email, key);

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(3));
    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);

    let start_phrase = pick_one(&mut rng, &config.phrases.start);
    println!("{} Sending start status: {}", timestamp(), &start_phrase);
    let response = status
        .set(&start_phrase, Some(Emoji::Name(config.emoji.start)), false)
        .await?;
    println!("{} Response {}", timestamp(), response.status());
    let mut last_send = Instant::now();
    loop {
        select! {
            recv(ticks) -> elapsed => {
                if last_send + cycle_seconds < elapsed.unwrap() {
                    let working_phrase = pick_one(&mut rng, &config.phrases.working);
                    println!("{} Sending working status: {}", timestamp(), &working_phrase);
                    let response = status
                        .set(&working_phrase, Some(Emoji::Name(config.emoji.working.to_string())), false)
                        .await?;
                    println!("{} Response {}", timestamp(), response.status());
                    last_send = Instant::now();
                }
            }
            recv(ctrl_c_events) -> _ => {
                let pause_phrase = pick_one(&mut rng, &config.phrases.pause);
                println!();
                println!("{} User interrupt! Sending pause status: {}", timestamp(), &pause_phrase);
                let response = status
                    .set(&pause_phrase, Some(Emoji::Name(config.emoji.pause)), true)
                    .await?;
                println!("{} Response {}", timestamp(), response.status());
                break;
            }
        }
    }

    Ok(())
}
