use core::future::Future;

use regex::Regex;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::ChannelId},
    prelude::Context,
};

use crate::{
    utils::chat::{get_channel, is_channel},
    InoriChannelUtils, InoriMessageUtils, MessageCreator,
};

pub async fn _purge<F, Fut>(ctx: &Context, msg: &Message, title: &str, mut args: Args, f: F) -> CommandResult
where
    F: Fn(Message) -> Fut,
    Fut: Future<Output = bool>, {
    let mut current = args.single::<String>().unwrap_or_default();

    let channel_id = if args.len() >= 2 && is_channel(&current) {
        let channel = get_channel(&current);
        current = args.single::<String>().unwrap_or_default();

        channel.parse::<u64>().unwrap_or_default()
    } else {
        msg.channel_id.0
    };

    let amount = if let Ok(amount) = current.parse::<u8>() {
        amount
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Purge").content("The value provided is not a valid number")
            })
            .await;
    };

    let rest = args.rest();
    let regex = if let Ok(regex) = if !rest.is_empty() {
        Regex::new(rest)
    } else {
        Regex::new(r".*")
    } {
        regex
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Purge").content("Unable to parse regex")
            })
            .await;
    };

    let mut loading_msg = msg
        .channel_id
        .send_loading(ctx, title, &format!("Finding and deleting {} messages", amount))
        .await
        .unwrap();

    let mut purge_count = 0;

    let channel = &ctx.http.get_channel(channel_id).await.unwrap().guild().unwrap();
    let find_msg = if channel_id == msg.channel_id.0 {
        loading_msg.clone()
    } else {
        let msgs = ChannelId(channel_id).messages(ctx, |r| r.limit(1)).await?;
        let msg = msgs.get(0).unwrap().clone();

        if regex.is_match(&msg.content) && f(msg.clone()).await {
            ctx.http.delete_message(msg.channel_id.0, msg.id.0).await.unwrap_or(());
            purge_count += 1;
        }

        msg
    };

    let messages = channel.messages(ctx, |r| r.before(&find_msg.id).limit(100)).await?;

    for message in messages {
        if purge_count < amount && regex.is_match(&message.content) && f(message.clone()).await {
            ctx.http.delete_message(message.channel_id.0, message.id.0).await.unwrap_or(());

            purge_count += 1;
        }

        if purge_count >= amount {
            break;
        }
    }

    let end = if purge_count == 1 { "message" } else { "messages" };

    return loading_msg
        .update_tmp(ctx, |m: &mut MessageCreator| {
            m.success().title(title).content(format!("Deleted {} {}", purge_count, end))
        })
        .await;
}

#[command]
#[aliases("embed", "emb")]
#[description("Purge messages that contain embeds")]
#[usage("[channel] <amount> [regex]")]
#[example("20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
async fn embeds(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge", args, async move |message: Message| {
        message.author.id == ctx.http.get_current_user().await.unwrap().id && !message.embeds.is_empty()
    })
    .await
}

#[command]
#[aliases("prune", "clear")]
#[description("Purge messages sent by yourself")]
#[usage("[chanel] <amount> [regex]")]
#[example("20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
#[max_args(3)]
#[sub_commands(embeds)]
async fn purge(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge", args, async move |message: Message| {
        message.author.id == ctx.http.get_current_user().await.unwrap().id
    })
    .await
}
