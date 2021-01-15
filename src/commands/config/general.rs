use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{
    save_settings,
    utils::chat::{say, say_error},
    Settings,
};

#[command]
#[description("Set the prefix of the bot")]
#[usage("<prefix>")]
#[example("~")]
#[num_args(1)]
async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let old_prefix = settings.clone().command_prefix;
    settings.command_prefix = args.rest().to_string();
    save_settings(&settings);

    let content = &format!(
        "Updated from '{}' to '{}'",
        old_prefix, settings.command_prefix
    );

    drop(settings);
    drop(data);
    say(ctx, &msg.channel_id, "Prefix", content).await
}

#[command]
#[aliases("filter")]
#[description(
    "Edit NSFW filtering levels\n\
    **Modes**\n\
    0 - Strict\n\
    1 - Moderate\n\
    2 - Disabled"
)]
#[usage("<level>")]
#[example("0")]
async fn nsfwfilter(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let content = if args.is_empty() {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        let level = match settings.global_nsfw_level {
            0 => "Strict",
            1 => "Moderate",
            _ => "Disabled",
        };

        format!("Filter level is currently set to {}", level)
    } else {
        if let Ok(val) = args.single::<u8>() {
            if val <= 2 {
                let data = ctx.data.write().await;
                let mut settings = data
                    .get::<Settings>()
                    .expect("Expected Setting in TypeMap.")
                    .lock()
                    .await;

                settings.global_nsfw_level = val;
                save_settings(&settings);

                let level = match val {
                    0 => "Strict",
                    1 => "Moderate",
                    _ => "Disabled",
                };

                format!("Filter level set to `{}`", level)
            } else {
                return say_error(
                    ctx,
                    &msg.channel_id,
                    "NSFW Filter",
                    "Invalid level specified.\n\
                    **Valid levels**\n\
                    `0` - Strict\n\
                    `1` - Moderate\n\
                    `2` - Disabled",
                )
                .await;
            }
        } else {
            "Unable to parse level".to_string()
        }
    };

    say(ctx, &msg.channel_id, "NSFW Filter", &content).await
}

#[command]
#[aliases("ga")]
#[description("Edit giveaway joiner configuration")]
#[usage("<subcommand>")]
#[example("delay 120")]
#[example("winmessage Hey, I won the giveway but I'm out rn. Can I redeem it when I get home?")]
#[min_args(1)]
async fn giveaway(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    say_error(
        ctx,
        &msg.channel_id,
        "Giveaway",
        &format!("Unknown subcommand: {}", args.current().unwrap()),
    )
    .await?;

    say(
        ctx,
        &msg.channel_id,
        "Giveaway",
        "Command not implemented yet",
    )
    .await
}
