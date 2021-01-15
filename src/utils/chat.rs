use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::standard::CommandResult,
    model::{channel::Message, id::ChannelId},
    prelude::*,
    utils::Colour,
};

use serenity_utils::menu::{Menu, MenuOptions};

use once_cell::sync::Lazy;

use regex::Regex;

use tokio::time::{delay_for, Duration};

use crate::{models::discord::Emote, Settings};

/*
 * Reminder if changing format that these
 * commands needs to be changed as well
 *
 * Server Info
 * Ping
 */

pub fn default_embed(title: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(format!("[{}]", title)).colour(Colour::BLURPLE);

    embed
}

pub fn default_error_embed(title: &str) -> CreateEmbed {
    let mut embed = default_embed(title);
    embed.colour(Colour::MEIBE_PINK);

    embed
}

pub async fn say(ctx: &Context, channel: &ChannelId, title: &str, content: &str) -> CommandResult {
    delete(ctx, &send(ctx, channel, title, content).await).await
}

pub async fn send(ctx: &Context, channel: &ChannelId, title: &str, content: &str) -> Message {
    let embed = default_embed(title);
    channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;
                e.description(content)
            })
        })
        .await
        .unwrap()
}

pub async fn say_error(
    ctx: &Context,
    channel: &ChannelId,
    title: &str,
    content: &str,
) -> CommandResult {
    delete(ctx, &send_error(ctx, channel, title, content).await).await
}

pub async fn send_error(ctx: &Context, channel: &ChannelId, title: &str, content: &str) -> Message {
    let embed = default_error_embed(title);
    channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;
                e.description(format!("Error lol: {}", content))
            })
        })
        .await
        .unwrap()
}

pub async fn send_embed(ctx: &Context, channel: &ChannelId, embed: CreateEmbed) -> Message {
    channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;

                e
            })
        })
        .await
        .unwrap()
}

pub async fn say_embed(ctx: &Context, channel: &ChannelId, embed: CreateEmbed) -> CommandResult {
    delete(ctx, &send_embed(ctx, channel, embed).await).await
}

pub async fn send_loading(
    ctx: &Context,
    channel: &ChannelId,
    title: &str,
    loading_msg: &str,
) -> Message {
    channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title(format!("[{}]", title))
                    .description(format!(
                        "<a:discordloading:395769211517009930> {}...",
                        loading_msg
                    ))
                    .colour(Colour::BLURPLE)
            })
        })
        .await
        .unwrap()
}

pub async fn delete(ctx: &Context, msg: &Message) -> CommandResult {
    let ad_delay = {
        let data = ctx.data.read().await;
        let settings: tokio::sync::MutexGuard<'_, Settings> = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        if settings.autodelete.enabled {
            Some(settings.autodelete.delay)
        } else {
            None
        }
    };

    if let Some(delay) = ad_delay {
        let ctx = ctx.clone();
        let msg = msg.clone();

        tokio::task::spawn(async move {
            delay_for(Duration::from_secs(delay)).await;
            let _ = ctx.http.delete_message(msg.channel_id.0, msg.id.0).await;
        });
    }

    Ok(())
}

pub async fn send_embed_paginator(
    ctx: &Context,
    msg: &Message,
    embeds: Vec<CreateEmbed>,
) -> CommandResult {
    let mut formatted_embeds = Vec::new();

    for (idx, embed) in embeds.iter().enumerate() {
        let mut msg = CreateMessage::default();

        msg.embed(|e| {
            let embed = embed.clone();

            e.0 = embed.0;

            e.colour(Colour::BLURPLE)
                .footer(|f| f.text(format!("Page {} of {}", idx + 1, embeds.len())))
        });

        formatted_embeds.push(msg);
    }

    let menu = Menu::new(ctx, msg, &formatted_embeds[..], MenuOptions::default());
    menu.run().await?;

    Ok(())
}

static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<@!?\d{18}>").unwrap());
static EMOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a?:[a-zA-Z0-9_]*?:\d{18}>").unwrap());
static EMOTE_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[^<a:]{1,}[a-zA-Z0-9][^:]").unwrap());
static EMOTE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{18}").unwrap());

pub fn is_mention(arg: &str) -> bool {
    MENTION_REGEX.is_match(&arg)
}

pub fn has_emotes(message: &str) -> bool {
    EMOTE_REGEX.is_match(&message)
}

pub fn get_emotes(arg: &str) -> Vec<Emote> {
    let mut matches = Vec::new();
    if !has_emotes(&arg) {
        return matches;
    }

    for mat in EMOTE_REGEX.captures_iter(&arg) {
        let mat = if let Some(mat) = mat.get(0) {
            mat.as_str()
        } else {
            continue;
        };

        if !EMOTE_ID_REGEX.is_match(&mat) || !EMOTE_NAME_REGEX.is_match(&mat) {
            continue;
        }

        let animated = mat.starts_with("<a:");
        let idm = EMOTE_ID_REGEX.find(&mat).unwrap();
        let id = mat[idm.start()..idm.end()].parse::<u64>().unwrap();

        let namem = EMOTE_NAME_REGEX.find(&mat).unwrap();
        let name = mat[namem.start()..namem.end()].to_string();
        let url = format!(
            "https://cdn.discordapp.com/emojis/{}.{}",
            id,
            if animated { "gif" } else { "png" }
        );

        let emote = Emote {
            name: name.to_string(),
            id,
            url,
            animated,
        };

        if !matches.contains(&emote) {
            matches.push(emote);
        }
    }

    matches
}
