use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
    pub phrases: Phrases,
    pub emoji: Emoji,
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub cycle_duration_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct Phrases {
    pub start: Vec<String>,
    pub working: Vec<String>,
    pub pause: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Emoji {
    pub start: String,
    pub working: String,
    pub pause: String,
}
