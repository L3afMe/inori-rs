use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

use crate::{parse_arg, save_settings, InoriChannelUtils, MessageCreator, Settings};

#[command]
#[aliases("ad")]
#[description("Edit auto delete configuration")]
#[usage("<subcommand>")]
#[example("disable")]
#[example("delay 50")]
#[min_args(1)]
#[sub_commands(delay, enable, disable, toggle)]
async fn autodelete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("Auto Delete")
                .content(&format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

#[command]
#[aliases("t")]
#[description("Toggles auto deleting messages")]
async fn toggle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
    settings.autodelete.enabled = !settings.autodelete.enabled;
    save_settings(&settings);

    let content = if settings.autodelete.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Auto Delete").content(content))
        .await
}

#[command]
#[description("Enables auto deleting messages")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.autodelete.enabled {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("Auto Delete").content("Already enabled")
            })
            .await
    } else {
        settings.autodelete.enabled = true;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("Auto Delete").content("Enabled")
            })
            .await
    }
}

#[command]
#[description("Disables auto deleting messages")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.autodelete.enabled {
        settings.autodelete.enabled = false;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("Auto Delete").content("Disabled")
            })
            .await
    } else {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("Auto Delete").content("Already disabled")
            })
            .await
    }
}

#[command]
#[description("Get or set the delay before messages are deleted")]
#[usage("<seconds>")]
#[example("10")]
#[max_args(1)]
async fn delay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.write().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
        let delay = settings.autodelete.delay;

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Auto Delete")
                    .content(format!("Delay currently set to {} seconds", delay))
            })
            .await;
    }

    let delay = parse_arg!(ctx, msg, args, "delay", u64);

    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.autodelete.delay = delay;
    save_settings(&settings);

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Auto Delete")
                .content(format!("Delay set to {} seconds", delay))
        })
        .await
}
