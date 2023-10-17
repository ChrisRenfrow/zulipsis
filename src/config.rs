use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Phrases {
    pub start: Vec<Phrase>,
    pub working: Vec<Phrase>,
    pub pause: Vec<Phrase>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Phrase {
    Basic(String),
    Emoji((String, String)),
}

#[derive(Debug, Deserialize)]
pub struct Emoji {
    pub start: String,
    pub working: String,
    pub pause: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_config() {
        let config: Config = toml::from_str(
            r#"
[general]
cycle_duration_seconds = 300
[phrases]
start = ["start"]
working = [["Rewrite it in Rust!", "ferris"]]
pause = ["pause"]
[emoji]
start = "start"
working = "working"
pause = "pause"
"#,
        )
        .unwrap();

        assert_eq!(config.general.cycle_duration_seconds, 300);
        assert_eq!(
            config.phrases.start,
            vec![Phrase::Basic("start".to_string())]
        );
        assert_eq!(
            config.phrases.working,
            vec![Phrase::Emoji((
                "Rewrite it in Rust!".to_string(),
                "ferris".to_string()
            ))]
        );
        assert_eq!(
            config.phrases.pause,
            vec![Phrase::Basic("pause".to_string())]
        );
        assert_eq!(config.emoji.start, "start".to_string());
        assert_eq!(config.emoji.working, "working".to_string());
        assert_eq!(config.emoji.pause, "pause".to_string());
    }
}
