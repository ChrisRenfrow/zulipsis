use clap::{Parser, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum SkipPhase {
    Start,
    Pause,
    Both,
}

#[derive(Parser)]
pub struct Cli {
    /// The path to zuliprc
    #[arg(short, long)]
    pub zuliprc: Option<String>,
    /// The path to the config
    #[arg(short, long)]
    pub config: Option<String>,
    /// Skip sending the start and/or pause statuses
    #[arg(short, long)]
    pub skip: Option<SkipPhase>,
    /// Print default config (e.g. to redirect to `~/.config/zulipsis/config.toml`)
    #[arg(long)]
    pub default_config: bool,
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}
