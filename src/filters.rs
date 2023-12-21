use std::sync::Arc;

use teloxide::{
    dptree,
    types::{ChatMemberKind, ChatMemberUpdated, Update},
};

use crate::{config::Config, misc::ReturnType};

type FilterType<Output> = ReturnType<Output>;

pub fn filter_channel_chat<Output>() -> FilterType<Output>
where
    Output: Send + Sync + 'static,
{
    dptree::filter(|req: Update, cfg: Arc<Config>| {
        req.chat()
            .is_some_and(|chat| chat.id == cfg.channel_chat_id)
    })
}

pub fn filter_channel_chat_got_member<Output>() -> FilterType<Output>
where
    Output: Send + Sync + 'static,
{
    filter_channel_chat().filter(|req: ChatMemberUpdated| {
        req.old_chat_member.kind.is_left() && req.new_chat_member.kind.is_member()
    })
}

pub fn filter_channel_lost_member<Output>() -> FilterType<Output>
where
    Output: Send + Sync + 'static,
{
    dptree::filter(|req: ChatMemberUpdated, cfg: Arc<Config>| {
        req.chat.id == cfg.channel_id
            && req.old_chat_member.is_member()
            && matches!(
                req.new_chat_member.kind,
                ChatMemberKind::Left | ChatMemberKind::Banned(_)
            )
    })
}
