use std::collections::HashMap;

use once_cell::sync::Lazy;
use rand::Rng;
use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::channel::Message,
    prelude::*,
};

use crate::{
    models::commands::NekosLifeResponse,
    utils::checks::{NSFW_MODERATE_CHECK, NSFW_STRICT_CHECK},
    InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings,
};

#[group]
#[commands(anal, blowjob, cuddle, feed, hug, kiss, pat, poke, slap, spank, tickle)]
#[description = "**Interactions**"]
struct Interactions;

static RESPONSES: Lazy<HashMap<String, Vec<String>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    map.insert("anal".to_string(), vec!["{0} dug their dick into {1}".to_string()]);
    map.insert("cuddle".to_string(), vec!["{0} snuggled up to {1}".to_string()]);
    map.insert("blowjob".to_string(), vec!["{0} gave {1} some sloppy top".to_string()]);
    map.insert("pat".to_string(), vec!["{0} petted {1}".to_string()]);
    map.insert("poke".to_string(), vec!["{0} poked {1}".to_string()]);
    map.insert("slap".to_string(), vec!["{0} slapped {1}".to_string()]);
    map.insert("spank".to_string(), vec!["{0} gave {1} a good spanking".to_string()]);
    map.insert("tickle".to_string(), vec!["{0} tickled {1}".to_string()]);
    map.insert("hug".to_string(), vec!["{0} snuggled up to {1}".to_string()]);
    map.insert("kiss".to_string(), vec!["{0} kissed {1}".to_string()]);
    map.insert("feed".to_string(), vec!["{0} fed {1}".to_string()]);

    map
});

async fn img_command(ctx: &Context, msg: &Message, command: &str, image: &str) -> CommandResult {
    if msg.mentions.len() > 1 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title(command).content(format!(
                    "Stop being such a slut, you can only{} 1 person at once",
                    command.to_lowercase()
                ))
            })
            .await;
    }

    let mut new_msg = msg.channel_id.send_loading(ctx, command, "Loading image").await.unwrap();

    let content = if msg.mentions.is_empty() {
        let data = ctx.data.read().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
        if settings.is_male { "himself" } else { "herself" }
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

    let responses = RESPONSES.get(image).unwrap().clone();
    let do_msg = responses[rand::thread_rng().gen_range(0..responses.len())].clone();

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
    img_command(ctx, msg, "Anal", "anal").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn cuddle(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Cuddle", "cuddle").await
}

#[command]
#[aliases("bj")]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[checks(NSFW_Strict)]
async fn blowjob(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Blowjob", "blowjob").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn pat(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Pat", "pat").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn poke(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Poke", "poke").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn slap(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Slap", "slap").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
#[checks(NSFW_Moderate)]
async fn spank(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Spank", "spank").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn tickle(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Tickle", "tickle").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn hug(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Hug", "hug").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn kiss(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Kiss", "kiss").await
}

#[command]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn feed(ctx: &Context, msg: &Message) -> CommandResult {
    img_command(ctx, msg, "Feed", "feed").await
}
