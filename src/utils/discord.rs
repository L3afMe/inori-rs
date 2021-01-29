use serde_json::{Number, Value};
use serenity::{
    model::{
        id::{GuildId, UserId},
        prelude::Member,
    },
    prelude::Context,
    Result,
};

pub async fn get_member(ctx: &Context, gid: GuildId, uid: UserId) -> Result<Member> {
    let mut value = reqwest::Client::new()
        .get(&format!("https://discord.com/api/v8/guilds/{}/members/{}", gid.0, uid.0))
        .header("Authorization", &ctx.http.token)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(map) = value.as_object_mut() {
        map.insert("guild_id".to_string(), Value::Number(Number::from(gid.0)));
    }

    serde_json::from_value::<Member>(value).map_err(From::from)
}
