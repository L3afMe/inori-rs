use serenity::{
    model::{id::ChannelId, Permissions},
    prelude::Context,
};

pub async fn get_perms(ctx: &Context, channel: &ChannelId) -> Permissions {
    if let Ok(channel) = ctx.http.get_channel(channel.0).await {
        if let Some(guild) = channel.guild() {
            let me = ctx.cache.current_user().await;

            if let Ok(perms) = guild.permissions_for_user(&ctx.cache, me.id.0).await {
                return perms;
            }
        }
    }

    Permissions::empty()
}
