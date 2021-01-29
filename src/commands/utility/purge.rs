use core::future::Future;

use colored::Colorize;
use regex::Regex;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::{Channel, Message},
        id::ChannelId,
    },
    prelude::Context,
};

use crate::{
    inori_error, inori_info, inori_success,
    utils::chat::{get_channel, is_channel},
    InoriChannelUtils, InoriMessageUtils, MessageCreator,
};

pub async fn _purge<F, Fut>(ctx: &Context, msg: &Message, title: &str, mut args: Args, f: F) -> CommandResult
where
    F: Fn(Message) -> Fut,
    Fut: Future<Output = bool>, {
    let mut current = args.single::<String>().unwrap_or_default();

    let silent = if args.len() >= 2 && current.to_lowercase().eq("silent") {
        current = args.single::<String>().unwrap_or_default();

        true
    } else {
        false
    };

    let channel_id = if args.len() >= 2 && is_channel(&current) {
        let channel = get_channel(&current);
        current = args.single::<String>().unwrap_or_default();

        channel.parse::<u64>().unwrap_or_default()
    } else {
        msg.channel_id.0
    };

    let amount = if let Ok(amount) = current.parse::<u64>() {
        amount
    } else {
        return if silent {
            inori_error!(title, "The value provided is not a valid number");

            Ok(())
        } else {
            msg.channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title(title).content("The value provided is not a valid number")
                })
                .await
        };
    };

    let rest = args.rest();
    let regex = if let Ok(regex) = if !rest.is_empty() {
        Regex::new(rest)
    } else {
        Regex::new(r".*")
    } {
        regex
    } else {
        return if silent {
            inori_error!(title, "Unable to parse regex");

            Ok(())
        } else {
            msg.channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title(title).content("Unable to parse regex")
                })
                .await
        };
    };

    let loading_msg = if silent {
        inori_info!(title, "Finding and deleting {} messages", amount);

        None
    } else {
        Some(
            msg.channel_id
                .send_loading(ctx, title, &format!("Finding and deleting {} messages", amount))
                .await
                .unwrap(),
        )
    };

    let mut purge_count = 0;

    let find_msg = if channel_id == msg.channel_id.0 && loading_msg.is_some() {
        loading_msg.clone().unwrap()
    } else {
        let msgs = ChannelId(channel_id).messages(ctx, |r| r.limit(1)).await?;
        let msg = msgs.get(0).unwrap().clone();

        if regex.is_match(&msg.content) && f(msg.clone()).await {
            ctx.http.delete_message(msg.channel_id.0, msg.id.0).await.unwrap_or(());
            purge_count += 1;
        }

        msg
    };

    let mut last_message_id = find_msg.id;
    let channel = &ctx.http.get_channel(channel_id).await.unwrap();

    let mut total_count = 0;
    'outer: while purge_count < amount {
        let messages = match channel {
            Channel::Guild(channel) => channel.messages(ctx, |r| r.before(last_message_id).limit(100)).await,
            Channel::Private(channel) => channel.messages(ctx, |r| r.before(last_message_id).limit(100)).await,
            _ => Ok(Vec::new()),
        };

        let messages = if let Ok(messages) = messages {
            messages
        } else {
            if silent {
                println!("Unable to get messages");

                return Ok(());
            }

            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title(title).content("Unable to get messages")
                })
                .await;
        };

        for message in &messages {
            if purge_count < amount && regex.is_match(&message.content) && f(message.clone()).await {
                ctx.http.delete_message(message.channel_id.0, message.id.0).await.unwrap_or(());

                purge_count += 1;
            }

            total_count += 1;
            last_message_id = message.id;

            if purge_count >= amount || total_count >= 1000 {
                break 'outer;
            }
        }

        if messages.is_empty() {
            break;
        }
    }

    let end = format!(
        "{}{}",
        if purge_count == 1 { "" } else { "s" },
        if total_count >= 1000 {
            " (Reached 1000 messages)"
        } else {
            ""
        }
    );
    let content = format!("Deleted {} message{}", purge_count, end);


    if let Some(mut loading_msg) = loading_msg {
        loading_msg
            .update_tmp(ctx, |m: &mut MessageCreator| m.success().title(title).content(content))
            .await
    } else {
        inori_success!(title, "{}", content);

        Ok(())
    }
}

#[command]
#[aliases("embed", "emb")]
#[description("Purge messages that contain embeds")]
#[usage("[channel] [silent] <amount> [regex]")]
#[example("20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
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
#[usage("[silent] [channel] <amount> [regex]")]
#[example("20")]
#[example("silent 20")]
#[example("#general 20")]
#[example("801105575038041266 20")]
#[example("20 \\[[a-zA-Z]*]")]
#[example("#general 20 \\[[a-zA-Z]*]")]
#[example("silent #general 20 \\[[a-zA-Z]*]")]
#[min_args(1)]
#[sub_commands(embeds)]
async fn purge(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    _purge(ctx, msg, "Purge", args, async move |message: Message| {
        message.author.id == ctx.http.get_current_user().await.unwrap().id
    })
    .await
}
