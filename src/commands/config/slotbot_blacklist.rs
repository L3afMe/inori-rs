use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{parse_arg, save_settings, InoriChannelUtils, MessageCreator, Settings};

#[command]
#[aliases("bl")]
#[description("Add/remove blacklisted guilds")]
#[min_args(1)]
#[sub_commands(add, remove)]
async fn blacklist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("SlotBot")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

#[command]
#[aliases("delete", "rem", "del", "d", "r")]
#[description("Remove a guild from blacklist, if no guild id is specified then the current guild will be removed")]
#[usage("[guild id]")]
#[example("800041653318451232")]
#[max_args(1)]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = if args.is_empty() {
        if let Some(guild) = msg.guild_id {
            guild.0
        } else {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("SlotBot").content("Unable to get current guild id")
                })
                .await;
        }
    } else {
        parse_arg!(ctx, msg, args, "guild id", u64)
    };

    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;

    if !settings.slotbot.blacklisted_guilds.contains(&guild_id) {
        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("SlotBot").content("Guild not currently blacklisted")
            })
            .await;
    } else {
        let filtered = settings
            .slotbot
            .blacklisted_guilds
            .clone()
            .into_iter()
            .filter(|g| *g != guild_id)
            .collect::<Vec<u64>>();
        settings.slotbot.blacklisted_guilds = filtered;
        save_settings(&settings);
        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("SlotBot").content("Removed guild from blacklist")
            })
            .await;
    };
}

#[command]
#[aliases("a")]
#[description("Add a guild to blacklist, if no guild id is specified then the current guild will be added")]
#[usage("[guild id]")]
#[example("800041653318451232")]
#[max_args(1)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = if args.is_empty() {
        if let Some(guild) = msg.guild_id {
            guild.0
        } else {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("SlotBot").content("Unable to get current guild id")
                })
                .await;
        }
    } else {
        parse_arg!(ctx, msg, args, "guild id", u64)
    };


    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;

    if settings.slotbot.blacklisted_guilds.contains(&guild_id) {
        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("SlotBot").content("Guild already blacklisted")
            })
            .await;
    } else {
        settings.slotbot.blacklisted_guilds.push(guild_id);
        save_settings(&settings);

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("SlotBot").content("Added guild to blacklist")
            })
            .await;
    };
}
