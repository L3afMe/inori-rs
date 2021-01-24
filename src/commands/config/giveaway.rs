use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::{giveaway_blacklist::*, giveaway_whitelist::*};
use crate::{parse_arg, save_settings, InoriChannelUtils, MessageCreator, Settings};

#[command]
#[description("Edit giveaway joiner configuration")]
#[usage("<subcommand>")]
#[example("delay 120")]
#[example("mode 1")]
#[example("blacklist add")]
#[min_args(1)]
#[sub_commands(enable, disable, toggle, maxdelay, mindelay, mode, whitelist, blacklist)]
async fn giveaway(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("Giveaway")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

#[command]
#[aliases("min")]
#[description("Set minimum delay before joining a giveaway")]
#[max_args(1)]
async fn mindelay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let min = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

            settings.giveaway.min_delay
        };

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Giveaway")
                    .content(format!("Minimum delay currently set to {}s", min))
            })
            .await;
    }

    let delay = parse_arg!(ctx, msg, args, "delay", u64);

    {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.giveaway.min_delay = delay;
        save_settings(&settings);
    }

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Giveaway")
                .content(format!("Minimum delay set to {}s", delay))
        })
        .await
}

#[command]
#[aliases("max")]
#[description("Set maximum delay before joining a giveaway")]
#[max_args(1)]
async fn maxdelay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let min = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

            settings.giveaway.max_delay
        };

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Giveaway")
                    .content(format!("Maximum delay currently set to {}s", min))
            })
            .await;
    }

    let delay = parse_arg!(ctx, msg, args, "delay", u64);

    {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.giveaway.max_delay = delay;
        save_settings(&settings);
    }

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Giveaway")
                .content(format!("Maximum delay set to {}s", delay))
        })
        .await
}

#[command]
#[aliases("t")]
#[description("Toggle giveaway joining")]
async fn toggle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.giveaway.enabled = !settings.giveaway.enabled;
    save_settings(&settings);

    let content = if settings.giveaway.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Giveaway").content(content))
        .await
}

#[command]
#[description("Enable giveaway joining")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.giveaway.enabled {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning().title("Giveaway").content("Already enabled")
            })
            .await
    } else {
        settings.giveaway.enabled = true;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Giveaway").content("Enabled"))
            .await
    }
}

#[command]
#[description("Disable giveaway joining")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.giveaway.enabled {
        settings.giveaway.enabled = false;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Giveaway").content("Disabled"))
            .await
    } else {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning().title("Giveaway").content("Already disabled")
            })
            .await
    }
}

#[command]
#[aliases("m")]
#[description(
    "Set the join mode\n**Modes**\n0 - All Servers\n1 - Whitelist; Only join in specific servers\n2 - Blacklist; Only \
     join outside of specific servers"
)]
#[usage("<mode>")]
#[example("1")]
#[num_args(1)]
async fn mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.write().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        let mode = match settings.giveaway.mode {
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
                    .title("Giveaway")
                    .content(format!("Join mode currently set to {}", mode))
            })
            .await;
    }

    let val = parse_arg!(ctx, msg, args, "mode", u8);
    if val <= 2 {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.giveaway.mode = val;
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
                m.success().title("Giveaway").content(format!("Join mode set to `{}`", mode))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Giveaway").content(
                    "Invalid mode specified.\n**Valid modes**\n`0` - All Servers\n`1` - Whitelist; Only join in \
                     specific servers\n`2` - Blacklist; Only join outside of specific servers",
                )
            })
            .await
    }
}
