use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{save_settings, utils, Settings};

async fn say(ctx: &Context, channel: &ChannelId, content: &str) -> CommandResult {
    utils::chat::say(ctx, channel, "Auto Delete", content).await
}

async fn say_error(ctx: &Context, channel: &ChannelId, content: &str) -> CommandResult {
    utils::chat::say_error(ctx, channel, "Auto Delete", content).await
}

#[command]
#[aliases("ad")]
#[description("Edit auto delete configuration")]
#[usage("<subcommand>")]
#[example("disable")]
#[example("delay 50")]
#[min_args(1)]
#[sub_commands(delay, enable, disable, toggle)]
async fn autodelete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    say_error(
        ctx,
        &msg.channel_id,
        &format!("Unknown subcommand: {}", args.current().unwrap()),
    )
    .await
}

#[command]
#[aliases("t")]
#[description("Toggles auto deleting messages")]
async fn toggle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    settings.autodelete.enabled = !settings.autodelete.enabled;
    save_settings(&settings);

    let content = if settings.autodelete.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    drop(settings);
    drop(data);
    say(ctx, &msg.channel_id, content).await
}

#[command]
#[description("Enables auto deleting messages")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let content = if settings.autodelete.enabled {
        "Already enabled"
    } else {
        settings.autodelete.enabled = true;
        save_settings(&settings);
        "Enabled"
    };

    drop(settings);
    drop(data);
    say(ctx, &msg.channel_id, content).await
}

#[command]
#[description("Disables auto deleting messages")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let content = if settings.autodelete.enabled {
        settings.autodelete.enabled = false;
        save_settings(&settings);
        "Disabled"
    } else {
        "Already disabled"
    };

    drop(settings);
    drop(data);
    say(ctx, &msg.channel_id, content).await
}

#[command]
#[description("Get or set the delay before messages are deleted")]
#[usage("<seconds>")]
#[example("10")]
async fn delay(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg = if args.is_empty() {
        "current"
    } else {
        args.current().unwrap()
    };

    let data = ctx.data.write().await;
    let mut settings: tokio::sync::MutexGuard<'_, Settings> = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let content = if arg.to_lowercase().eq("current") {
        format!(
            "Delay currently set to {} seconds",
            settings.autodelete.delay
        )
    } else if let Ok(delay) = arg.parse::<u64>() {
        settings.autodelete.delay = delay;
        save_settings(&settings);

        format!("Delay set to {} seconds", delay)
    } else {
        return say_error(ctx, &msg.channel_id, "Unable to parse delay to number").await;
    };

    drop(settings);
    drop(data);
    say(ctx, &msg.channel_id, &content).await
}
