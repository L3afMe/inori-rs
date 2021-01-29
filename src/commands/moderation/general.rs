use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Context,
};

use crate::commands::utility::purge::_purge;

#[command]
#[aliases("human")]
#[description("Purge messages that were sent by non bot accounts")]
#[usage("[channel] [silent] <amount> [regex]")]
#[example("20")]
#[example("silent 20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
async fn humans(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge Chat", args, async move |message: Message| !message.author.bot).await
}

#[command]
#[aliases("bot")]
#[description("Purge messages that were sent by bot accounts")]
#[usage("[channel] [silent] <amount> [regex]")]
#[example("20")]
#[example("silent 20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
async fn bots(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge Chat", args, async move |message: Message| message.author.bot).await
}

#[command]
#[aliases("embed", "emb")]
#[description("Purge messages that containing embeds")]
#[usage("[channel] [silent] <amount> [regex]")]
#[example("20")]
#[example("silent 20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
async fn embeds(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge Chat", args, async move |message: Message| {
        !message.embeds.is_empty()
    })
    .await
}

#[command]
#[aliases("prunechat", "clearchat")]
#[description("Deletes a specified amount of messages sent by anyone")]
#[usage("[chanel] [silent] <amount> [regex]")]
#[example("20")]
#[example("silent 20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
#[required_permissions("MANAGE_MESSAGES")]
#[min_args(1)]
#[max_args(3)]
#[sub_commands(bots, embeds, humans)]
async fn purgechat(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge Chat", args, async move |_: Message| true).await
}
