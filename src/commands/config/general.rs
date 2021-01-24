use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{parse_arg, save_settings, InoriChannelUtils, MessageCreator, Settings};

#[command]
#[description("Set the prefix of the bot")]
#[usage("<prefix>")]
#[example("~")]
#[num_args(1)]
async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
    let old_prefix = settings.clone().command_prefix;

    settings.command_prefix = args.rest().to_string();
    save_settings(&settings);

    let content = &format!("Updated from '{}' to '{}'", old_prefix, settings.command_prefix);

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Prefix").content(content))
        .await
}

#[command]
#[aliases("filter")]
#[description("Edit NSFW filtering levels\n**Modes**\n0 - Strict\n1 - Moderate\n2 - Disabled")]
#[usage("<level>")]
#[example("0")]
async fn nsfwfilter(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.read().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        let level = match settings.global_nsfw_level {
            0 => "Strict",
            1 => "Moderate",
            _ => "Disabled",
        };

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("NSFW")
                    .content(format!("Filter level is currently set to {}", level))
            })
            .await;
    }

    let val = parse_arg!(ctx, msg, args, "level", u8);

    if val <= 2 {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.global_nsfw_level = val;
        save_settings(&settings);

        let level = match val {
            0 => "Strict",
            1 => "Moderate",
            _ => "Disabled",
        };

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success()
                    .title("NSFW Filter")
                    .content(format!("Filter level set to `{}`", level))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("NSFW Filter")
                    .content("Invalid level specified.\n**Valid levels**\n`0` - Strict\n`1` - Moderate\n`2` - Disabled")
            })
            .await
    }
}

#[command]
#[description(
    "Edit embed config\n**Modes**\n0 - Never\n1 - Detect embed perms (Doesn't work for some people)\n2 - Always"
)]
#[usage("<mode>")]
#[example("2")]
async fn embedmode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.read().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        let mode = match settings.embed_mode {
            0 => "Never",
            1 => "Detect",
            _ => "Always",
        };

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Embed Mode")
                    .content(format!("Embed mode is currently set to {}", mode))
            })
            .await;
    }

    let val = parse_arg!(ctx, msg, args, "mode", u8);

    if val <= 2 {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.embed_mode = val;
        save_settings(&settings);

        let mode = match val {
            0 => "Never",
            1 => "Detect",
            _ => "Always",
        };

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("Embed Mode").content(format!("Embed mode set to `{}`", mode))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Embed Mode")
                    .content("Invalid mode specified.\n**Valid Modes**\n`0` - Never\n`1` - Detect\n`2` - Always")
            })
            .await
    }
}
