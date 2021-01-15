use serenity::{
    framework::standard::{macros::check, CheckResult},
    model::channel::Message,
    prelude::Context,
};

use crate::Settings;

#[check]
#[display_in_help]
#[check_in_help(false)]
#[name = "NSFW_Moderate"]
pub async fn _nsfw_moderate(ctx: &Context, msg: &Message) -> CheckResult {
    can_nsfw_moderate(ctx, msg).await
}

pub async fn can_nsfw_moderate(ctx: &Context, msg: &Message) -> CheckResult {
    let level = {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        settings.global_nsfw_level
    };

    if ctx
        .http
        .get_channel(msg.channel_id.0)
        .await
        .unwrap()
        .is_nsfw()
        || level >= 1
    {
        return CheckResult::Success;
    } else {
        return CheckResult::new_user("nsfw_moderate");
    }
}

#[check]
#[display_in_help]
#[check_in_help(false)]
#[name = "NSFW_Strict"]
pub async fn _nsfw_strict(ctx: &Context, msg: &Message) -> CheckResult {
    can_nsfw_strict(ctx, msg).await
}

pub async fn can_nsfw_strict(ctx: &Context, msg: &Message) -> CheckResult {
    let level = {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        settings.global_nsfw_level
    };

    if ctx
        .http
        .get_channel(msg.channel_id.0)
        .await
        .unwrap()
        .is_nsfw()
        || level == 2
    {
        return CheckResult::Success;
    } else {
        return CheckResult::new_user("nsfw_strict");
    }
}
