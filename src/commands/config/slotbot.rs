use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::{slotbot_blacklist::*, slotbot_whitelist::*};
use crate::{save_settings, InoriChannelUtils, MessageCreator, Settings};

#[command]
#[description("Snipe SlotBot wallet drops")]
#[min_args(1)]
#[sub_commands(enable, disable, toggle, mode, whitelist, blacklist)]
async fn slotbot(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("SlotBot")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

#[command]
#[aliases("t")]
#[description("Toggle SlotBot sniping")]
async fn toggle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.slotbot.enabled = !settings.slotbot.enabled;
    save_settings(&settings);

    let content = if settings.slotbot.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("SlotBot").content(content))
        .await
}

#[command]
#[description("Enable SlotBot sniping")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if settings.slotbot.enabled {
        "Already enabled"
    } else {
        settings.slotbot.enabled = true;
        save_settings(&settings);

        "Enabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("SlotBot").content(content))
        .await
}

#[command]
#[description("Disable SlotBot sniping")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if settings.slotbot.enabled {
        settings.slotbot.enabled = false;
        save_settings(&settings);

        "Disabled"
    } else {
        "Already disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("SlotBot").content(content))
        .await
}

#[command]
#[aliases("m")]
#[description(
    "Set the snipe mode\n**Modes**\n0 - All Servers\n1 - Whitelist; Only snipe in specific servers\n2 - Blacklist; \
     Only snipe outside of specific servers"
)]
#[usage("<mode>")]
#[example("1")]
#[num_args(1)]
async fn mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if args.is_empty() {
        let sort = match settings.slotbot.mode {
            0 => "All servers",
            1 => "Whitelist",
            _ => "Blacklist",
        };

        format!("Sort mode currently set to {}", sort)
    } else if let Ok(val) = args.single::<u8>() {
        if val <= 1 {
            settings.slotbot.mode = val;
            save_settings(&settings);

            let sort = match val {
                0 => "All servers",
                1 => "Whitelist",
                _ => "Blacklist",
            };

            format!("Sort mode set to `{}`", sort)
        } else {
            drop(settings);
            drop(data);

            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("SlotBot").content(
                        "Invalid mode specified.\n**Valid modes**\n`0` - All Servers\n`1` - Whitelist; Only snipe in \
                         specific servers\n`2` - Blacklist; Only snipe outside of specific servers",
                    )
                })
                .await;
        }
    } else {
        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("SlotBot").content("Unable to parse mode")
            })
            .await;
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("SlotBot").content(content))
        .await
}
