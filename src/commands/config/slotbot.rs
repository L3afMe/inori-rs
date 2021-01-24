use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::{slotbot_blacklist::*, slotbot_whitelist::*};
use crate::{parse_arg, save_settings, InoriChannelUtils, MessageCreator, Settings};

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
        .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("SlotBot").content(content))
        .await
}

#[command]
#[description("Enable SlotBot sniping")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.slotbot.enabled {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning().title("SlotBot").content("Already enabled")
            })
            .await
    } else {
        settings.slotbot.enabled = true;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("SlotBot").content("Enabled"))
            .await
    }
}

#[command]
#[description("Disable SlotBot sniping")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.slotbot.enabled {
        settings.slotbot.enabled = false;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("SlotBot").content("Disabled"))
            .await
    } else {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning().title("SlotBot").content("Already disabled")
            })
            .await
    }
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
    if args.is_empty() {
        let data = ctx.data.write().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        let mode = match settings.slotbot.mode {
            0 => "All servers",
            1 => "Whitelist",
            _ => "Blacklist",
        };

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("SlotBot")
                    .content(format!("Snipe mode currently set to {}", mode))
            })
            .await;
    }

    let val = parse_arg!(ctx, msg, args, "mode", u8);
    if val <= 2 {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.slotbot.mode = val;
        save_settings(&settings);

        let mode = match val {
            0 => "All servers",
            1 => "Whitelist",
            _ => "Blacklist",
        };

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("SlotBot").content(format!("Snipe mode set to `{}`", mode))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("SlotBot").content(
                    "Invalid mode specified.\n**Valid modes**\n`0` - All Servers\n`1` - Whitelist; Only snipe in \
                     specific servers\n`2` - Blacklist; Only snipe outside of specific servers",
                )
            })
            .await
    }
}
