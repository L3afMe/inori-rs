mod mal;
mod pfpswitcher;
mod quote;

use mal::*;
use pfpswitcher::*;
use quote::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{channel::Message, id::ChannelId, user::User},
    prelude::Context,
};

use crate::{InoriChannelUtils, MessageCreator};

#[group]
#[commands(
    balance,
    compatibility,
    dick,
    myanimelist,
    pfpswitcher,
    quote,
    sexuality,
    urbandictionary
)]
#[description("**Fun**")]
struct Fun;

fn make_bar(percent: u64) -> String {
    let mut bar = "[".to_string();

    for i in 0..20 {
        if i < (percent / 5) {
            bar = format!("{}=", bar);
        } else {
            bar = format!("{}-", bar);
        }
    }

    format!("{}]", bar)
}

#[command]
#[aliases("ship")]
#[description("Check compatability between two people")]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[min_args(1)]
#[max_args(2)]
async fn compatibility(ctx: &Context, msg: &Message) -> CommandResult {
    let mut user1 = &msg.author;
    let user2;

    if msg.mentions.len() == 1 {
        user2 = msg.mentions.get(0).unwrap_or(&msg.author);
    } else {
        user1 = msg.mentions.get(0).unwrap_or(&msg.author);
        user2 = msg.mentions.get(1).unwrap_or(&msg.author);
    }

    let compat = (user1.id.0 + user2.id.0) % 100;
    let bar = make_bar(compat);
    let shipnamep1 = user1.name[0..user1.name.len() / 2].to_string();
    let shipnamep2 = user2.name[user2.name.len() / 2..].to_string();
    let shipname = format!("{}{}", shipnamep1, shipnamep2);

    msg.channel_id
        .send_noret(ctx, |m: &mut MessageCreator| {
            m.title("Compatibility")
                .content(format!("{} has {}% compatibility\n{}", shipname, compat, bar))
        })
        .await
}

async fn print_dick(ctx: &Context, channel: &ChannelId, users: &[User]) -> CommandResult {
    let mut content = String::new();

    for user in users {
        let len = (user.id.0 % 15) + 1;
        let mut dick = "8".to_string();

        for _ in 0..len {
            dick = format!("{}=", dick);
        }

        dick = format!("{}D", dick);
        if content.is_empty() {
            content = format!("{}'s dick size\n{}", user.name, dick);
        } else {
            content = format!("{}\n\n{}'s dick size\n{}", content, user.name, dick);
        }
    }

    channel
        .send_noret(ctx, |m: &mut MessageCreator| m.title("Dick").content(content))
        .await
}

#[command]
#[description("Check how big a users dick is")]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn dick(ctx: &Context, msg: &Message) -> CommandResult {
    let mut users = Vec::new();

    if msg.mentions.is_empty() {
        users.push(msg.author.clone());
    } else {
        users = msg.mentions.clone();
    }

    print_dick(ctx, &msg.channel_id, &users).await
}

async fn print_sexuality(ctx: &Context, channel: &ChannelId, users: &[User]) -> CommandResult {
    let mut content = String::new();

    for user in users {
        let perc = user.id.0 % 100;
        let bar = make_bar(perc);

        if content.is_empty() {
            content = format!("{} is {}% gay\n{}", user.name, perc, bar);
        } else {
            content = format!("{}\n\n{} is {}% gay\n{}", content, user.name, perc, bar);
        }
    }

    channel
        .send_noret(ctx, |m: &mut MessageCreator| m.title("Sexuality").content(content))
        .await
}

#[command]
#[aliases("gay")]
#[description("Check how big a users dick is")]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn sexuality(ctx: &Context, msg: &Message) -> CommandResult {
    let mut users = Vec::new();

    if msg.mentions.is_empty() {
        users.push(msg.author.clone());
    } else {
        users = msg.mentions.clone();
    }

    print_sexuality(ctx, &msg.channel_id, &users).await
}

#[command]
#[aliases("urban", "ud")]
#[description("Searches Urban Dictionary for a word or phrase")]
#[usage("<word/phrase>")]
#[example("bet")]
#[min_args(1)]
async fn urbandictionary(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let client = reqwest::Client::new();
    let results = urban_rs::fetch_definition(&client, args.rest()).await?;

    return if results.is_empty() {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning().title("Urban Dictionary").content("No results found")
            })
            .await
    } else {
        let mut msgs = Vec::new();

        for result in results {
            let mut msg = MessageCreator::default();
            msg.title("Urban Dictionary")
                .content(format!("**{}**\n{}", result.word(), result.definition()));

            msgs.push(msg);
        }

        msg.channel_id.send_paginator_noret(ctx, msg, msgs).await
    };
}

#[command]
#[aliases("bal")]
#[description("Check your balance")]
async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("Balance")
                .content("You a broke ass, don't even bother checking your bal")
        })
        .await
}
