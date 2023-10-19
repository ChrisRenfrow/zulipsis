use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use clap::{Parser, ValueEnum};
use crossbeam_channel::{bounded, select, tick, Receiver};
use rand::Rng;
use std::{
    fs,
    time::{Duration, Instant},
};

mod config;
mod zulip_status;
mod zuliprc;

use config::{Config, Phrase};
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
    /// Skip sending the start and/or pause statuses
    #[arg(short, long)]
    skip: Option<SkipPhase>,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
enum SkipPhase {
    Start,
    Pause,
    Both,
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn pick_one<T>(rng: &mut impl Rng, list: &[T]) -> T
where
    T: Clone,
{
    list[rng.gen_range(0..list.len())].clone()
}

fn timestamp() -> String {
    let current_datetime: DateTime<Local> = Local::now();
    current_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn phrase_with_emoji_or_default(phrase: Phrase, default_emoji: Emoji) -> (String, Emoji) {
    match phrase {
        Phrase::Basic(phrase_str) => (phrase_str, default_emoji),
        Phrase::Emoji((phrase_str, emoji_str)) => (phrase_str, Emoji::Name(emoji_str)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let zuliprc: ZulipRc = toml::from_str(&fs::read_to_string(args.zuliprc)?)?;
    let config: Config = toml::from_str(&fs::read_to_string(args.config)?)?;
    let (skip_start, skip_end) = match args.skip {
        Some(skip) => match skip {
            SkipPhase::Start => (true, false),
            SkipPhase::Pause => (false, true),
            _ => (true, true),
        },
        _ => (false, false),
    };
    let mut rng = rand::thread_rng();

    let email = zuliprc.api.email;
    let key = zuliprc.api.key;
    let status = ZulipStatus::new(zuliprc.api.site, email, key);

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(3));
    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);

    let mut last_send = Instant::now();
    if !skip_start {
        let (start_phrase, emoji) = phrase_with_emoji_or_default(
            pick_one(&mut rng, &config.phrases.start),
            Emoji::Name(config.emoji.start),
        );
        println!("{} Selected start status: {}", timestamp(), &start_phrase);
        let response = status.set(&start_phrase, Some(emoji), false).await?;
        println!("{} Response {}", timestamp(), response.status());
    } else {
        // Turn back the clock to trigger sending a working phrase shortly after starting
        last_send = Instant::now() - cycle_seconds;
    }
    loop {
        select! {
            recv(ticks) -> elapsed => {
                if last_send + cycle_seconds < elapsed.unwrap() {
                    let (working_phrase, emoji) = phrase_with_emoji_or_default(
                        pick_one(&mut rng, &config.phrases.working),
                        Emoji::Name(config.emoji.working.to_string())
                    );
                    println!("{} Sending working status: {}", timestamp(), &working_phrase);
                    let response = status
                        .set(&working_phrase, Some(emoji), false)
                        .await?;
                    println!("{} Response {}", timestamp(), response.status());
                    last_send = Instant::now();
                }
            }
            recv(ctrl_c_events) -> _ => {
                println!();
                if !skip_end {
                    let (pause_phrase, emoji) = phrase_with_emoji_or_default(
                        pick_one(&mut rng, &config.phrases.pause),
                        Emoji::Name(config.emoji.pause)
                    );
                    println!("{} User interrupt! Sending pause status: {}", timestamp(), &pause_phrase);
                    let response = status
                        .set(&pause_phrase, Some(emoji), true)
                        .await?;
                    println!("{} Response {}", timestamp(), response.status());
                } else {
                    println!("{} User interrupt!", timestamp());
                }
                break;
            }
        }
    }

    Ok(())
}
