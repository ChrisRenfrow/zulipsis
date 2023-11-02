use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    pub phrases: Phrases,
    pub emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub start: String,
    pub working: String,
    pub pause: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General {
                cycle_duration_seconds: 300,
            },
            phrases: Phrases {
                start: vec![
                    Phrase::Basic("getting started".to_string()),
                    Phrase::Emoji(("waking-up".to_string(), "sunrise".to_string())),
                    Phrase::Emoji(("catching-up on zulip".to_string(), "zulip".to_string())),
                ],
                working: vec![
                    Phrase::Basic("working".to_string()),
                    Phrase::Emoji(("thinking".to_string(), "brain".to_string())),
                    Phrase::Basic("reading the docs".to_string()),
                ],
                pause: vec![
                    Phrase::Basic("taking a break".to_string()),
                    Phrase::Emoji(("afk".to_string(), "keyboard".to_string())),
                ],
            },
            emoji: Emoji {
                start: "start".to_string(),
                working: "tools".to_string(),
                pause: "zzz".to_string(),
            },
        }
    }
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
