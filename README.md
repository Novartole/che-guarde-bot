# che-guarde-bot

A telegram bot made with [teloxide](https://docs.rs/teloxide/latest/teloxide/) to do some routine administration. A production version of this bot can be found [here](https://github.com/Insomnia-IT/che-guarde-bot). This version is also ready for deployment.

Фестивальная версия этого бота и документация на русском языке находятся [тут](https://github.com/Insomnia-IT/che-guarde-bot).

## User story
There is a public channel which has a joined chat for comments. The last one should be open for anyone to allow users to write comments below posts with no restrictions. But at the same time, disscussions beyond post's topics should be accessable only for those people who match some rules. Users who don't match the rules, should be kicked from the chat and banned in it.

As an admin I don't want to check every person who has joined and then do according actions manually.

## Solution
Bot is a member of the channel and chat. And it has admin rights to kick and ban users. It's also a member of a chat of administrators (aka work chat). 

Once a user joins the chat of comments, Bot checks that the user follows the rules. It does same when a user left the chat. In most cases Bot also sends a notification about processed users. For instance, if an admin invited a user, then Bot sends a message to the work chat like 'Admin (@username) added a user (@username) who isn't a member of channel'.

## Development
The following variables are mandatory to be able to run Bot locally:
- CHANNEL_ID - channel id,
- CHANNEL_CHAT_ID - chat id of a chat linked to the channel (usually this's a chat created for comments of channel's posts),
- WORK_CHAT_ID - chat id of an admin chat which members will get notifications from Bot,
- TELOXIDE_TOKEN - telegram-bot token. 

Channel and chat ids could be get, [for instance](https://stackoverflow.com/questions/72640703/telegram-how-to-find-group-chat-id), from the web version of Telegram. The token is provided by [BotFather](https://telegram.me/BotFather).

These variables can be set either explicitly or inside a config file. If there is a file called .env.local, then the following command starts the app:
```rust
CONFIG_PATH=".env.local" cargo run
```
In case when the app is going to be run in debug profile, Bot's maintainer could set his/her user id into MAINTAINER_ID. That allows to use helpfull commands while developing. The whole list of those commands can be get by calling /help, once Bot is up.

Logging can be configured by standart variable called RUST_LOG. Example of use can be found in .env.example file located in the root folder.

Access test can be done by calling /ping command. This command is available for any users but the command should be sent in a private chat to Bot. If Bot is available, it responds with *pong* message.
