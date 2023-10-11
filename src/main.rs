use anyhow::{Error, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use crossbeam_channel::{bounded, select, tick, Receiver};

use rand::Rng;
use reqwest::{Client, ClientBuilder, Response};
use serde::Deserialize;
use std::{
    fmt::format,
    fs,
    future::Future,
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

#[derive(Debug, Clone, Copy)]
struct StatusUpdater<'a> {
    client: &'a Client,
    url: &'a str,
    email: &'a str,
    key: &'a str,
}

impl<'a> StatusUpdater<'a> {
    fn new(client: &'a Client, url: &'a str, email: &'a str, key: &'a str) -> Self {
        Self {
            client,
            url,
            email,
            key,
        }
    }

    fn update(
        self: Self,
        status_text: &str,
        away: Option<bool>,
        emoji_name: &str,
    ) -> impl Future<Output = Result<Response, reqwest::Error>> {
        let away_str = match away {
            Some(true) => "true",
            _ => "false",
        };
        self.client
            .post(self.url)
            .basic_auth(self.email, Some(self.key))
            .form(&[
                ("status_text", status_text),
                ("away", away_str),
                ("emoji_name", emoji_name),
            ])
            .send()
    }
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

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(3));

    let email = zuliprc.api.email;
    let key = zuliprc.api.key;
    let client = ClientBuilder::new().timeout(Duration::new(5, 0)).build()?;
    let url = format!("{}/api/v1/users/me/status", &zuliprc.api.site);
    let status = StatusUpdater::new(&client, &url, &email, &key);
    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);

    let mut last_send = Instant::now();

    let start_phrase = pick_one(&mut rng, &config.phrases.start);
    println!("{} Sending start status: {}", timestamp(), &start_phrase);
    let response = status
        .update(&start_phrase, None, &config.emoji.start)
        .await?;
    println!("{} Response {}", timestamp(), response.status());
    loop {
        select! {
            recv(ticks) -> elapsed => {
                if last_send + cycle_seconds < elapsed.unwrap() {
                    let working_phrase = pick_one(&mut rng, &config.phrases.working);
                    println!("{} Sending working status: {}", timestamp(), &working_phrase);
                    let response = status
                        .update(&working_phrase, None, &config.emoji.working)
                        .await?;
                    println!("{} Response {}", timestamp(), response.status());
                    last_send = Instant::now();
                } else {

                }

            }
            recv(ctrl_c_events) -> _ => {
                let pause_phrase = pick_one(&mut rng, &config.phrases.pause);
                println!();
                println!("{} User interrupt! Sending pause status: {}", timestamp(), &pause_phrase);
                let response = status
                    .update(&pause_phrase, Some(true), &config.emoji.pause)
                    .await?;
                println!("{} Response {}", timestamp(), response.status());
                break;
            }
        }
    }

    Ok(())
}
