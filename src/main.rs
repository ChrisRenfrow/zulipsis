use anyhow::{Error, Result};
use clap::Parser;
use crossbeam_channel::{bounded, select, tick, Receiver};

use rand::Rng;
use serde::Deserialize;
use std::{
    fs,
    time::{Duration, Instant},
};

#[derive(Parser)]
struct Cli {
    /// The path to zuliprc
    #[arg(short, long)]
    zuliprc: String,
    /// The path to the config
    #[arg(short, long)]
    config: String,
}

#[derive(Debug, Deserialize)]
struct ZulipRc {
    api: Api,
}

#[derive(Debug, Deserialize)]
struct Api {
    email: String,
    key: String,
    site: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    general: General,
    phrases: Box<Phrases>,
    emoji: Emoji,
}

#[derive(Debug, Deserialize)]
struct General {
    cycle_duration_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct Phrases {
    start: Box<[String]>,
    working: Box<[String]>,
    pause: Box<[String]>,
}

#[derive(Debug, Deserialize)]
struct Emoji {
    start: String,
    working: String,
    pause: String,
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

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let zuliprc: ZulipRc = toml::from_str(&fs::read_to_string(args.zuliprc)?)?;
    let config: Config = toml::from_str(&fs::read_to_string(args.config)?)?;

    let mut rng = rand::thread_rng();

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));
    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);

    let mut last_send = Instant::now();

    println!("Start: {}", pick_one(&mut rng, &config.phrases.start));

    loop {
        select! {
            recv(ticks) -> elapsed => {
                if last_send + cycle_seconds < elapsed.unwrap() {
                    println!("Working: {}", pick_one(&mut rng, &config.phrases.working));
                    last_send = Instant::now();
                } else {
                    println!("Sleeping...");
                }

            }
            recv(ctrl_c_events) -> _ => {
                println!();
                println!("User interrupt: Pause: {}", pick_one(&mut rng, &config.phrases.pause));
                break;
            }
        }
    }

    Ok(())
}
