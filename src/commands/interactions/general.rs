extern crate serde;
extern crate serde_json;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{
    models::commands::NekosLifeResponse,
    utils::checks::{NSFW_MODERATE_CHECK, NSFW_STRICT_CHECK},
    InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings,
};

async fn img_command(
    ctx: &Context,
    msg: &Message,
    command: &str,
    image: &str,
    do_msg: &str,
) -> CommandResult {
    if msg.mentions.len() > 1 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title(command).content(format!(
                    "Stop being such a slut, you can only\
                    {} 1 person at once",
                    command.to_lowercase()
                ))
            })
            .await;
    }

    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, command, "Loading image")
        .await
        .unwrap();

    let content = if msg.mentions.len() == 0 {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        let pronoun = if settings.is_male {
            "himself"
        } else {
            "herself"
        };

        pronoun
    } else {
        &msg.mentions.get(0).unwrap().name
    };

    let my_name = match msg.author_nick(&ctx.http).await {
        Some(nick) => nick,
        None => (&msg.author.name).to_string(),
    };

    let res = reqwest::get(&format!("https://nekos.life/api/v2/img/{}", image))
        .await?
        .text()
        .await?;

    let res: NekosLifeResponse = serde_json::from_str(&res).expect("Couldn't parse response.");

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.title(command)
                .content(do_msg.replace("{0}", &my_name).replace("{1}", content))
                .image(res.url)
        })
        .await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[checks(NSFW_Strict)]
async fn anal(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Anal", "anal", "{0} gave clapped {1}'s cheeks").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn cuddle(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Cuddle", "cuddle", "{0} snuggled up to {1}").await
}

#[command]
#[aliases("bj")]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[checks(NSFW_Strict)]
async fn blowjob(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(
        ctx,
        msg,
        "Blowjob",
        "blowjob",
        "{0} gave {1} some sloppy top",
    )
    .await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn pat(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Pat", "pat", "{0} patted {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn poke(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Poke", "poke", "{0} poked {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn slap(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Slap", "slap", "{0} slapped {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[checks(NSFW_Moderate)]
async fn spank(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Spank", "spank", "{0} gave a {1} a good spanking").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn tickle(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Tickle", "tickle", "{0} tickled {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn hug(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Hug", "hug", "{0} snuggled up to {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn kiss(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Kiss", "kiss", "{0} kissed {1}").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn feed(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Feed", "feed", "{0} fed {1}").await
}
