use std::env;

use teloxide::types::{ChatId, UserId};

pub struct Config {
    pub channel_id: ChatId,
    pub channel_chat_id: ChatId,
    pub work_chat_id: ChatId,
    pub maintainer_id: Option<UserId>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            channel_id: ChatId({
                const CHANNEL_ID: &str = "CHANNEL_ID";
                env::var(CHANNEL_ID)
                    .expect(format!("{} must be specified", CHANNEL_ID).as_str())
                    .parse()
                    .expect(format!("Failed to parse {} value into i64", CHANNEL_ID).as_str())
            }),
            channel_chat_id: ChatId({
                const CHANNEL_CHAT_ID: &str = "CHANNEL_CHAT_ID";
                env::var(CHANNEL_CHAT_ID)
                    .expect(format!("{} must be specified", CHANNEL_CHAT_ID).as_str())
                    .parse()
                    .expect(format!("Failed to parse {} value into i64", CHANNEL_CHAT_ID).as_str())
            }),
            work_chat_id: ChatId({
                const WORK_CHAT_ID: &str = "WORK_CHAT_ID";
                env::var(WORK_CHAT_ID)
                    .expect(format!("{} must be specified", WORK_CHAT_ID).as_str())
                    .parse()
                    .expect(format!("Failed to parse {} value into i64", WORK_CHAT_ID).as_str())
            }),
            maintainer_id: {
                const MAINTAINER_ID: &str = "MAINTAINER_ID";
                match env::var(MAINTAINER_ID) {
                    Ok(value) => match value.parse() {
                        Ok(id) => Some(UserId(id)),
                        Err(err) => {
                            log::warn!(
                                "{} is optional but it can't be pasrsed successfully: {}",
                                MAINTAINER_ID,
                                err
                            );

                            None
                        }
                    },
                    Err(err) => {
                        log::warn!("{} is optional and it was not provided. Error: {}", MAINTAINER_ID, err);

                        None
                    }
                }
            },
        }
    }
}
