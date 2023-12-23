# che-guarde-bot

A telegram bot made with [teloxide](https://docs.rs/teloxide/latest/teloxide/) to do some routine administration.

## User story.
There is a public channel which has a joined chat for comments. The last one should be open for anyone to allow users to write comments below posts with no restrictions. But at the same time, disscussions beyond post's topics should be accessable only for those people who match some rules. Users who don't match the rules, should be kicked from the chat and banned in it.

As an admin I don't want to check every person who has joined and then do according actions manually.

## Solution
Bot is a member of the channel and chat. And it has admin rights to kick and ban users. It's also a member of a chat of administrators (aka work chat). 

Once a user joins the chat of comments, Bot checks that the user follows the rules. It does same when a user left the chat. In most cases Bot also sends a notification about processed users. For instance, if an admin invited a user, then Bot sends a message to the work chat like 'Admin (@username) added a user (@username) who isn't a member of channel'.

## Development
When Debug profile is used and maintaner_id is set, additional commands are available. It helps with development and debugging. Maintainer can (un)ban h(im/er)self in the channel and chat, kick from the chat and do some other minor things.
