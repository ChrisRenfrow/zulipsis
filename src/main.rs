use anyhow::{Error, Result};
use clap::{Parser, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use crossbeam_channel::{bounded, select, tick, Receiver};
use log::{debug, info};
use rand::Rng;

use std::{
    fs,
    path::{Path, PathBuf},
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
    zuliprc: Option<String>,
    /// The path to the config
    #[arg(short, long)]
    config: Option<String>,
    /// Skip sending the start and/or pause statuses
    #[arg(short, long)]
    skip: Option<SkipPhase>,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
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

fn phrase_with_emoji_or_default(phrase: Phrase, default_emoji: Emoji) -> (String, Emoji) {
    match phrase {
        Phrase::Basic(phrase_str) => (phrase_str, default_emoji),
        Phrase::Emoji((phrase_str, emoji_str)) => (phrase_str, Emoji::Name(emoji_str)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    // TODO: Prompt to create config path on first run?
    //       Maybe also Zulip login inline (like zulip cli client) to fetch zuliprc?
    //       ty @erikareads

    // Use $XDG_CONFIG_HOME
    let mut config_dir: PathBuf = match std::env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).to_path_buf(),
        // Or use `$HOME/.config``
        _ => Path::new(
            &std::env::var("HOME")
                .expect("Couldn't find HOME directory! Something is terribly wrong!"),
        )
        .join(".config"),
    };
    // Append `zulipsis` directory
    config_dir.push("zulipsis");
    // create director(y|ies)
    let _ = fs::create_dir_all(&config_dir);
    let config_dir = config_dir.into_boxed_path();
    // Construct the `zuliprc` path
    let zuliprc_path = config_dir.join("zuliprc");
    // Construct the `config.toml` path
    let config_path = config_dir.join("config.toml");

    // Check if zuliprc argument exists
    let zuliprc: ZulipRc = match args.zuliprc {
        // Use zuliprc path from argument
        Some(path) => toml::from_str(&fs::read_to_string(path)?)?,
        // Otherwise check if zuliprc path exists
        None => match zuliprc_path.try_exists() {
            // Use zuliprc from path
            Ok(true) => toml::from_str(&fs::read_to_string(zuliprc_path)?)?,
            // No zuliprc provided, we can't proceed
            // TODO: Don't use panic for user-facing errors (ty @erikareads)
            Ok(false) => panic!("zuliprc file doesn't exist"),
            // Or potential file-system error
            Err(_) => panic!("File-system error! Probably!"),
        },
    };

    // Practically the same as the above (DRY potential)
    let config: Config = match args.config {
        Some(path) => toml::from_str(&fs::read_to_string(path)?)?,
        None => match config_path.try_exists() {
            Ok(true) => toml::from_str(&fs::read_to_string(config_path)?)?,
            Ok(false) => panic!("config.toml doesn't exist"),
            Err(_) => panic!("File-system error! Probably!"),
        },
    };

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
    let ticks = tick(Duration::from_secs(1));
    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);

    let mut last_send = Instant::now();
    if !skip_start {
        let (start_phrase, emoji) = phrase_with_emoji_or_default(
            pick_one(&mut rng, &config.phrases.start),
            Emoji::Name(config.emoji.start),
        );
        info!("Sending start status: {}", &start_phrase);
        let response = status.set(&start_phrase, Some(emoji), false).await?;
        debug!("Response: {}", response.status());
    } else {
        info!("Skipping start status");
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
                    info!("Sending working status: {}", &working_phrase);
                    let response = status
                        .set(&working_phrase, Some(emoji), false)
                        .await?;
                    debug!("Response {}", response.status());
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
                    info!("Interrupt received. Sending pause status: {}", &pause_phrase);
                    let response = status
                        .set(&pause_phrase, Some(emoji), true)
                        .await?;
                    debug!("Response {}", response.status());
                } else {
                    info!("Interrupt received. Skipping pause status.");
                }
                break;
            }
        }
    }

    Ok(())
}
