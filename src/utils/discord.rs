use once_cell::sync::Lazy;
use serde_json::{Number, Value};
use serenity::{
    model::prelude::{GuildId, Member, Permissions, Role, UserId},
    prelude::Context,
    utils::Colour,
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

pub async fn get_roles(ctx: &Context, gid: GuildId, member: &Member) -> Option<Vec<Role>> {
    if let Ok(roles) = ctx.http.get_guild_roles(gid.0).await {
        let mut mem_roles = Vec::new();

        let mut itr = roles.into_iter();
        for role in &member.roles {
            if let Some(role) = itr.find(|r| role.eq(&r.id)) {
                mem_roles.push(role.clone());
            }
        }

        Some(mem_roles)
    } else {
        None
    }
}

pub fn get_top_colour(roles: Vec<Role>) -> Option<Colour> {
    roles.into_iter().map(|r| r.colour).find(|c| c.0 != 0)
}

pub async fn get_permissions(
    ctx: &Context,
    gid: GuildId,
    member: Option<&Member>,
    roles: Option<Vec<Role>>,
) -> Permissions {
    let member = if let Some(member) = member {
        member.clone()
    } else {
        let user = ctx.http.get_current_user().await.unwrap();
        ctx.http.get_member(gid.0, user.id.0).await.unwrap()
    };

    let roles = if let Some(roles) = roles {
        roles
    } else if let Some(roles) = get_roles(ctx, gid, &member).await {
        roles
    } else {
        return Permissions::empty();
    };

    let mut bits = 0;

    if let Ok(guild) = ctx.http.get_guild(gid.0).await {
        if guild.owner_id == member.user.id {
            return Permissions {
                bits: 2146959359,
            };
        }
    }

    for role in roles {
        bits |= role.permissions.bits
    }

    Permissions {
        bits,
    }
}

pub const DM_PERMISSIONS: Lazy<Option<Permissions>> = Lazy::new(|| {
    Some(Permissions::from_bits(0b000_0010_0011_0101_1100_1100_0100_0000).unwrap_or_else(Permissions::empty))
});
