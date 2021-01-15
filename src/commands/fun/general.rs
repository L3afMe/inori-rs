use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, user::User},
    prelude::*,
};

use crate::utils::chat::{default_embed, say_error, send, send_embed_paginator};

async fn print_dick(ctx: &Context, msg: &Message, user: &User) {
    let len = (user.id.0 % 15) + 1;
    let mut dick = "8".to_string();
    for _ in 0..len {
        dick = format!("{}=", dick);
    }
    dick = format!("{}D", dick);

    send(
        ctx,
        &msg.channel_id,
        "Dick",
        &format!("{}'s dick size\n{}", user.name, dick),
    )
    .await;
}

#[command]
#[description("Check how big a users dick is")]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn dick(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.len() == 0 {
        print_dick(ctx, msg, &msg.author).await;
    } else {
        for mention in &msg.mentions {
            print_dick(ctx, msg, &mention).await;
        }
    }
    Ok(())
}

async fn print_sexuality(ctx: &Context, msg: &Message, user: &User) {
    let len = user.id.0 % 100;
    let mut bar = "[".to_string();
    for i in 0..20 {
        if i < (len / 5) {
            bar = format!("{}=", bar);
        } else {
            bar = format!("{}-", bar);
        }
    }
    bar = format!("{}]", bar);

    send(
        ctx,
        &msg.channel_id,
        "Sexuality",
        &format!("{} is {}% gay\n{}", user.name, len, bar),
    )
    .await;
}

#[command]
#[aliases("gay")]
#[description("Check how big a users dick is")]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn sexuality(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.len() == 0 {
        print_sexuality(ctx, msg, &msg.author).await;
    } else {
        for mention in &msg.mentions {
            print_sexuality(ctx, msg, &mention).await;
        }
    }
    Ok(())
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
        say_error(ctx, &msg.channel_id, "Urban Dictionary", "No results found").await
    } else {
        let mut embeds = Vec::new();

        for result in results {
            let mut embed = default_embed("Urban Dictionary");

            embed.description(format!("**{}**\n{}", result.word(), result.definition()));

            embeds.push(embed);
        }

        send_embed_paginator(ctx, msg, embeds).await
    };
}

#[command]
#[aliases("bal")]
#[description("Check your balance")]
async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    say_error(
        ctx,
        &msg.channel_id,
        "Balance",
        "You a broke ass nigga, don't even bother checking your bal",
    )
    .await
}
