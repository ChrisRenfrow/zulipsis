use anyhow::{bail, Error};

use log::{debug, info};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    config::{Config, Phrase},
    zulip_status::{Emoji, ZulipStatus},
};

#[derive(Debug)]
pub enum Phase {
    Start,
    Working,
    Pause,
}

pub struct Zulipsis {
    rng: ThreadRng,
    config: Config,
    status: ZulipStatus,
}

fn phrase_with_emoji_or_default(phrase: Phrase, default_emoji: Emoji) -> (String, Emoji) {
    match phrase {
        Phrase::Basic(phrase_str) => (phrase_str, default_emoji),
        Phrase::Emoji((phrase_str, emoji_str)) => (phrase_str, Emoji::Name(emoji_str)),
    }
}

fn pick_one<T>(rng: &mut impl Rng, list: &[T]) -> T
where
    T: Clone,
{
    list[rng.gen_range(0..list.len())].clone()
}

impl Zulipsis {
    pub fn new(config: Config, status: ZulipStatus) -> Self {
        Self {
            rng: rand::thread_rng(),
            config,
            status,
        }
    }

    pub async fn set_status_for_phase(&mut self, phase: Phase) -> Result<(), Error> {
        let (phrase, emoji) = match phase {
            Phase::Start => phrase_with_emoji_or_default(
                pick_one(&mut self.rng, &self.config.phrases.start),
                Emoji::Name(self.config.emoji.start.clone()),
            ),
            Phase::Working => phrase_with_emoji_or_default(
                pick_one(&mut self.rng, &self.config.phrases.working),
                Emoji::Name(self.config.emoji.working.clone()),
            ),
            Phase::Pause => phrase_with_emoji_or_default(
                pick_one(&mut self.rng, &self.config.phrases.pause),
                Emoji::Name(self.config.emoji.pause.clone()),
            ),
        };
        info!("Sending {:?} status: {}", phase, &phrase);
        match self.status.set(&phrase, Some(emoji), false).await {
            Ok(response) => Ok(debug!("Response: {}", response.status())),
            Err(e) => bail!("Problem sending status: {e}"),
        }
    }
}
