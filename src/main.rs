use anyhow::{anyhow, bail, Error, Result};
use clap::Parser;

use cli::{Cli, SkipPhase};
use crossbeam_channel::{bounded, select, tick, Receiver};
use log::info;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

mod cli;
mod config;
mod zulip_status;
mod zuliprc;
mod zulipsis;

use config::Config;
use zulip_status::ZulipStatus;
use zuliprc::ZulipRc;
use zulipsis::{Phase, Zulipsis};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn get_config_home() -> PathBuf {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).to_path_buf(),
        // Or use `$HOME/.config``
        _ => Path::new(
            &std::env::var("HOME")
                .expect("Couldn't find HOME directory! Something is terribly wrong!"),
        )
        .join(".config"),
    }
}

fn get_config(path: PathBuf) -> Result<Config, Error> {
    match path.try_exists() {
        Ok(true) => match toml::from_str(&fs::read_to_string(path)?) {
            Ok(config) => Ok(config),
            Err(e) => Err(anyhow!("Problem parsing config: {e}")),
        },
        Ok(false) => bail!(
            r#"Specified file does not exist: {}
Run with --default-config to print a sensible default configuration"#,
            path.display()
        ),
        Err(_) => bail!("Not permitted to check if file exists: {}", path.display()),
    }
}

fn get_zuliprc(path: PathBuf) -> Result<ZulipRc, Error> {
    match path.try_exists() {
        Ok(true) => match toml::from_str(&fs::read_to_string(path)?) {
            Ok(zuliprc) => Ok(zuliprc),
            Err(e) => Err(anyhow!("Problem parsing zuliprc: {e}")),
        },
        Ok(false) => bail!(
            r#"Specified file does not exist: {}
Refer to https://zulip.com/api/api-keys#get-your-api-key to learn how to find your zuliprc"#,
            path.display()
        ),
        Err(_) => bail!("Not permitted to check if file exists: {}", path.display()),
    }
}

fn get_config_and_zuliprc(
    config_path: Option<String>,
    zuliprc_path: Option<String>,
) -> Result<(Config, ZulipRc), Error> {
    let mut config_home = get_config_home();
    config_home.push("zulipsis");
    let config_home = config_home.into_boxed_path();
    let default_config_path = config_home.join("config.toml");
    let default_zuliprc_path = config_home.join("zuliprc");

    Ok((
        match config_path {
            Some(path) => get_config(path.into())?,
            None => {
                fs::create_dir_all(&config_home)?;
                get_config(default_config_path)?
            }
        },
        match zuliprc_path {
            Some(path) => get_zuliprc(path.into())?,
            None => {
                fs::create_dir_all(&config_home)?;
                get_zuliprc(default_zuliprc_path)?
            }
        },
    ))
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
    if args.default_config {
        // Print the config and exit
        let default_cfg = Config::default();
        print!("{}", toml::to_string(&default_cfg).unwrap());
        return Ok(());
    }

    let (config, zuliprc) = get_config_and_zuliprc(args.config, args.zuliprc)?;
    let (skip_start, skip_end) = match args.skip {
        Some(skip) => match skip {
            SkipPhase::Start => (true, false),
            SkipPhase::Pause => (false, true),
            _ => (true, true),
        },
        _ => (false, false),
    };

    let cycle_seconds = Duration::from_secs(config.general.cycle_duration_seconds);
    let email = zuliprc.api.email;
    let key = zuliprc.api.key;
    let mut zulipsis = Zulipsis::new(config, ZulipStatus::new(zuliprc.api.site, email, key));

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let mut last_send = Instant::now();
    if !skip_start {
        zulipsis.set_status_for_phase(Phase::Start).await?;
    } else {
        info!("Skipping start status");
        // Turn back the clock to trigger sending a working phrase shortly after starting
        last_send = Instant::now() - cycle_seconds;
    }

    loop {
        select! {
            recv(ticks) -> elapsed => {
                if last_send + cycle_seconds < elapsed.unwrap() {
                    zulipsis.set_status_for_phase(Phase::Working).await?;
                    last_send = Instant::now();
                }
            }
            recv(ctrl_c_events) -> _ => {
                println!();
                if !skip_end {
                    zulipsis.set_status_for_phase(Phase::Pause).await?;
                } else {
                    info!("Interrupt received. Skipping pause status.");
                }
                break;
            }
        }
    }

    Ok(())
}
