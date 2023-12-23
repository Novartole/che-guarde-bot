mod config;
mod filters;
mod handlers;
mod misc;

use config::Config;
use std::{collections::HashSet, sync::Arc};
use teloxide::prelude::*;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    if let Ok(filename) = std::env::var("CONFIG_PATH") {
        dotenv::from_filename(filename).ok();
    }

    pretty_env_logger::init();

    let mut handlers = dptree::entry()
        .branch(
            Update::filter_chat_member()
                .branch(handlers::update::user_was_invited_to_chat_by_admin())
                .branch(handlers::update::user_joined_channel_chat())
                .branch(handlers::update::user_left_or_was_kicked_from_channel()),
        )
        .branch(
            Update::filter_message()
                .branch(handlers::message::user_was_kicked_from_channel_chat())
                .branch(handlers::message::sent_checkhealth_command()),
        );

    if cfg!(debug_assertions) {
        handlers = dptree::entry()
            .branch(handlers)
            .branch(Update::filter_message().branch(handlers::message::maintainer_sent_command()));
    }

    log::info!("Starting bot...");
    Dispatcher::builder(Bot::from_env(), handlers)
        .dependencies(dptree::deps![
            // config
            Arc::new(Config::new()),
            // kicked or banned user ids
            Arc::new(Mutex::new(HashSet::<UserId>::new()))
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
