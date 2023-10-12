use reqwest::{Client, ClientBuilder, Response};
use std::{fmt::Display, future::Future, time::Duration};

/// Specify the emoji to be used via one of two methods
#[allow(dead_code)]
pub enum Emoji {
    /// Simply the name of the emoji
    Name(String),
    /// The code of the emoji, which varies based on the namespace (unicode, realm, or extra)
    /// https://zulip.com/api/update-status#parameter-reaction_type
    Code(ReactionType),
}

/// The emoji namespace and method of identification
#[derive(Debug)]
#[allow(dead_code)]
pub enum ReactionType {
    /// Hex encoding of the unicode emoji e.g. 1f419
    Unicode(String),
    /// Custom emoji: Use the ID of the emoji e.g. 9153
    Realm(u32),
    /// The name of the included Zulip emoji e.g. "zulip"
    Extra(String),
}

impl Display for ReactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner_value_str: String = match self {
            Self::Unicode(v) => v.to_string(),
            Self::Realm(v) => v.to_string(),
            Self::Extra(v) => v.to_string(),
        };
        write!(f, "{}", &inner_value_str)
    }
}

pub struct ZulipStatus {
    client: Client,
    request_url: String,
    username: String,
    key: String,
}

impl ZulipStatus {
    /// Constructs a new ZulipStatus instance which can be re-used to set and clear the status of a Zulip user
    pub fn new(base_url: String, username: String, key: String) -> Self {
        let request_url = format!("{base_url}/api/v1/users/me/status");
        let client = ClientBuilder::new()
            .timeout(Duration::new(5, 0))
            .build()
            .unwrap();

        Self {
            client,
            request_url,
            username,
            key,
        }
    }

    /// Sets a new status message
    pub fn set(
        &self,
        text: &str,
        emoji: Option<Emoji>,
        away: bool,
    ) -> impl Future<Output = Result<Response, reqwest::Error>> {
        let form: &[Option<(String, String)>] = &[
            Some(("status_text".to_string(), text.to_string())),
            Some((
                "away".to_string(),
                match away {
                    true => "true".to_string(),
                    _ => "false".to_string(),
                },
            )),
            emoji.map(|emoji| match emoji {
                Emoji::Name(name) => ("emoji_name".to_string(), name),
                Emoji::Code(reaction_type) => ("emoji_code".to_string(), reaction_type.to_string()),
            }),
        ];

        self.client
            .post(&self.request_url)
            .basic_auth(self.username.to_string(), Some(self.key.to_string()))
            .form(&form)
            .send()
    }

    /// Clear status (set to empty string)
    #[allow(dead_code)]
    pub fn clear(&self) -> impl Future<Output = Result<Response, reqwest::Error>> {
        self.client
            .post(&self.request_url)
            .basic_auth(self.username.to_string(), Some(self.key.to_string()))
            .form(&[("status_text", "")])
            .send()
    }
}
