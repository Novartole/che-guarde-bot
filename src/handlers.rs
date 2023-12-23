use std::{collections::HashSet, sync::Arc};

use teloxide::{requests::Requester, respond, types::UserId, Bot, RequestError};
use tokio::sync::Mutex;

use crate::{config::Config, misc::ReturnType};

type HandlerType = ReturnType<Result<(), RequestError>>;

pub mod update {
    use super::*;
    use teloxide::types::{ChatMemberKind, ChatMemberUpdated};

    use crate::{
        filters::{filter_channel_chat_got_member, filter_channel_lost_member},
        misc::{create_username_or_default, FormatArgument},
    };

    const DEFAULT_USER_USERNAME: &str = "(пользователь скрыл свой ID)";

    pub fn user_was_invited_to_chat_by_admin() -> HandlerType {
        filter_channel_chat_got_member()
            .filter_async(|bot: Bot, req: ChatMemberUpdated| async move {
                let from_id = req.from.id;
                let user_id = req.old_chat_member.user.id;

                from_id != user_id
                    && bot.get_chat_administrators(req.chat.id).await.map_or_else(
                        |err| {
                            log::error!(
                                "Failed to get administrators of chat:\nchat: {:?},\nerror{}",
                                req.chat,
                                err
                            );

                            false
                        },
                        |admins| admins.into_iter().any(|admin| admin.user.id == from_id),
                    )
            })
            .inspect(|| {
                log::debug!("user_was_invited_to_chat_by_admin: filters passed, calling endpoint");
            })
            .endpoint(
                |bot: Bot, req: ChatMemberUpdated, cfg: Arc<Config>| async move {
                    let admin = req.from;
                    let user = req.old_chat_member.user;

                    if !bot
                        .get_chat_member(cfg.channel_id, user.id)
                        .await?
                        .is_member()
                    {
                        log::debug!(
                            "({:?}, {}) is NOT a member of channel ({})",
                            user.username,
                            user.id,
                            cfg.channel_id
                        );

                        bot.send_message(
                            cfg.work_chat_id,
                            format!(
                                "{} добавил(а) пользователя, которого нет в канале: {}",
                                create_username_or_default(
                                    "(админ скрыл свой ID)",
                                    admin.username.as_ref(),
                                ),
                                create_username_or_default(
                                    DEFAULT_USER_USERNAME,
                                    user.username.as_ref(),
                                )
                            ),
                        )
                        .await?;
                        log::debug!("A message was sent to work chat ({}).", cfg.work_chat_id);
                    }

                    respond(())
                },
            )
    }

    pub fn user_joined_channel_chat() -> HandlerType {
        filter_channel_chat_got_member()
            .filter(|req: ChatMemberUpdated| req.from.id == req.old_chat_member.user.id)
            .inspect(|| {
                log::debug!("user_joined_channel_chat: filters passed, calling endpoint");
            })
            .endpoint(
                |bot: Bot, req: ChatMemberUpdated, cfg: Arc<Config>| async move {
                    let user = req.from;
                    let channel_id = cfg.channel_id;

                    if bot.get_chat_member(channel_id, user.id).await?.is_member() {
                        log::debug!("({:?}, {}) is a member of channel ({})", user.username, user.id, channel_id);

                        bot.send_message(
                            cfg.work_chat_id,
                            format!(
                                "{}{} вступление одобрено",
                                user.first_name.end_with_comma_if_not_empty(),
                                create_username_or_default(DEFAULT_USER_USERNAME, user.username.as_ref())
                            ),
                        )
                        .await?;
                        log::debug!("A message was sent to work chat ({})", cfg.work_chat_id);

                    } else {
                        log::debug!("({:?}, {}) is NOT a member of channel ({})", user.username, user.id, channel_id);

                        bot.kick_chat_member(cfg.channel_chat_id, user.id).await?;
                        log::debug!("({:?}, {}) has been kicked from chat ({})", user.username, user.id, cfg.channel_chat_id);

                        let mut message = format!(
                            "{}{} нет в канале, удален из чата",
                            user.first_name.end_with_comma_if_not_empty(),
                            create_username_or_default(DEFAULT_USER_USERNAME, user.username.as_ref())
                        );

                        if let Err(err) = bot.ban_chat_member(cfg.channel_chat_id, user.id).await {
                            log::error!(
                                "Partial error. Failed to ban user in chat.\nUser: {:#?}.\nChat: {}.\nError: {}",
                                user,
                                req.chat.id,
                                err
                            );
                        } else {
                            log::debug!("({:?}, {}) has been banned in chat ({})", user.username, user.id, cfg.channel_chat_id);

                            message.push_str(" и заблокирован");
                        }

                        bot.send_message(cfg.work_chat_id, message).await?;
                        log::debug!("A message was sent to work chat ({})", cfg.work_chat_id);
                    }

                    respond(())
                },
            )
    }

    pub fn user_left_or_was_kicked_from_channel() -> HandlerType {
        filter_channel_lost_member()
            .inspect(|| {
                log::debug!("user_left_or_was_kicked_from_channel: filters passed");
            })
            .endpoint(
                |bot: Bot,
                 req: ChatMemberUpdated,
                 cfg: Arc<Config>,
                 ids: Arc<Mutex<HashSet<UserId>>>| async move {
                    let channel_chat_id = cfg.channel_chat_id;
                    let user = req.old_chat_member.user;

                    if bot
                        .get_chat_member(channel_chat_id, user.id)
                        .await?
                        .is_member()
                    {
                        log::debug!("({:?}, {}) is a member of chat ({})", user.username, user.id, channel_chat_id);

                        let inserted = ids.lock().await.insert(user.id);
                        log::debug!("{} inserted into ids? {}!", user.id, inserted);

                        match async {
                            match req.new_chat_member.kind {
                                ChatMemberKind::Left => {
                                    //
                                    // it only kicks user from chat
                                    //
                                    bot.unban_chat_member(channel_chat_id, user.id).await?;
                                    log::debug!("({:?}, {}) has been only kicked (no ban) from chat ({})", user.username, user.id, channel_chat_id);
                                }
                                ChatMemberKind::Banned(_) => {
                                    //
                                    // it additionally bans user
                                    //
                                    bot.kick_chat_member(channel_chat_id, user.id).await?;
                                    log::debug!("({:?}, {}) has been kicked (w/ ban) from chat ({})", user.username, user.id, channel_chat_id);
                                }
                                unexpected_kind => {
                                    log::warn!(
                                        "{:?} is not expected to be passed filters. User won't be touched.",
                                        unexpected_kind
                                    );

                                    return Ok(false);
                                }
                            }

                            Ok(true)
                        }
                        .await
                        {
                            Ok(success) => {
                                if success {
                                    log::debug!("got SUCCESS? true!");

                                    bot.send_message(
                                        cfg.work_chat_id,
                                        format!(
                                            "{}{} вышел из канала и был удален из чата",
                                            user.first_name.end_with_comma_if_not_empty(),
                                            create_username_or_default(
                                                DEFAULT_USER_USERNAME,
                                                user.username.as_ref(),
                                            )
                                        ),
                                    )
                                    .await?;
                                    log::debug!("A message was sent to work chat ({})", cfg.work_chat_id);
                                } else {
                                    log::debug!("got SUCCESS? false!");

                                    //
                                    // user hasn't been touched - no need to process left chat message
                                    //
                                    let removed = ids.lock().await.remove(&user.id);
                                    log::debug!("{} was removed from ids? {}!", user.id, removed);
                                }
                            }
                            Err(err) => {
                                log::debug!("got an error but expected success = true/false: {}", err);

                                //
                                // something went wrong - no need to process left chat message
                                //
                                let removed = ids.lock().await.remove(&user.id);
                                log::debug!("{} was removed from ids? {}!", user.id, removed);

                                return Err(err);
                            }
                        }
                    }

                    respond(())
                },
            )
    }
}

pub mod message {
    use super::*;
    use teloxide::{
        dispatching::HandlerExt,
        dptree,
        types::{Message, MessageKind, MessageLeftChatMember},
        utils::command::BotCommands,
    };

    use crate::filters::filter_channel_chat;

    pub fn user_was_kicked_from_channel_chat() -> HandlerType {
        filter_channel_chat()
            //
            // pass only message, which are not handled yet
            //
            .filter_map_async(
                |msg: Message, ids: Arc<Mutex<HashSet<UserId>>>| async move {
                    if let MessageKind::LeftChatMember(msg) = msg.kind {
                        if ids.lock().await.contains(&msg.left_chat_member.id) {
                            return Some(msg);
                        }
                    }

                    None
                },
            )
            .inspect(|| {
                log::debug!("user_was_kicked_from_channel_chat: filters passed, calling endpoint");
            })
            .endpoint(
                |bot: Bot,
                 msg: Message,
                 svc_msg: MessageLeftChatMember,
                 ids: Arc<Mutex<HashSet<UserId>>>| async move {
                    bot.delete_message(msg.chat.id, msg.id).await?;
                    log::debug!("service message deleted: ({:?})", msg.kind);

                    //
                    // mark id as handled
                    //
                    let removed = ids.lock().await.remove(&svc_msg.left_chat_member.id);
                    log::debug!(
                        "{} was removed from ids? {}!",
                        svc_msg.left_chat_member.id,
                        removed
                    );

                    respond(())
                },
            )
    }

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase")]
    enum MaintainerCommands {
        #[command(description = "get ids")]
        Ids,
        #[command(description = "unban maintainer in channel's chat")]
        UnbanChat,
        #[command(description = "unban maintainer in channel")]
        UnbanChannel,
        #[command(description = "kick (w/ ban) maintainer from channel")]
        KickAndBan,
        #[command(description = "kick (w/o ban) maintainer from channel")]
        Kick,
        #[command(description = "show this message")]
        Help,
    }

    pub fn maintainer_sent_command() -> HandlerType {
        dptree::filter_map(|msg: Message, cfg: Arc<Config>| {
            if msg.chat.is_private() {
                if let Some(id) = cfg.maintainer_id {
                    if msg.from().is_some_and(|user| user.id == id) {
                        return Some(id);
                    }
                }
            }

            None
        })
        .filter_command::<MaintainerCommands>()
        .inspect(|| {
            log::debug!("maintainer_sent_command: filters passed, calling endpoint");
        })
        .endpoint(
            |bot: Bot,
             msg: Message,
             maintainer_id: UserId,
             cmd: MaintainerCommands,
             cfg: Arc<Config>| async move {
                match cmd {
                    MaintainerCommands::UnbanChat => {
                        bot.unban_chat_member(cfg.channel_chat_id, maintainer_id)
                            .await?;
                    }
                    MaintainerCommands::UnbanChannel => {
                        bot.unban_chat_member(cfg.channel_id, maintainer_id).await?;
                    }
                    MaintainerCommands::KickAndBan => {
                        bot.kick_chat_member(cfg.channel_id, maintainer_id).await?;
                    }
                    MaintainerCommands::Kick => {
                        bot.unban_chat_member(cfg.channel_id, maintainer_id).await?;
                    }
                    MaintainerCommands::Help => {
                        bot.send_message(
                            msg.chat.id,
                            MaintainerCommands::descriptions().to_string(),
                        )
                        .await?;
                    }
                    MaintainerCommands::Ids => {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                "- channel: {}\n- channel's chat: {}\n- work chat: {}",
                                cfg.channel_id, cfg.channel_chat_id, cfg.work_chat_id
                            ),
                        )
                        .await?;
                    }
                }

                respond(())
            },
        )
    }

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase")]
    enum CheckhealthCommands {
        Ping,
    }

    pub fn sent_checkhealth_command() -> HandlerType {
        dptree::filter(|msg: Message| msg.chat.is_private())
            .filter_command::<CheckhealthCommands>()
            .endpoint(
                |bot: Bot, msg: Message, cmd: CheckhealthCommands| async move {
                    match cmd {
                        CheckhealthCommands::Ping => bot.send_message(msg.chat.id, "pong").await?,
                    };

                    Ok(())
                },
            )
    }
}
