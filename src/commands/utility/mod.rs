mod automsg;
mod emotestealer;
pub mod purge;
mod tags;

// use automsg::*;
use std::{cmp::min, collections::HashMap, time::Instant};

use emotestealer::*;
use purge::*;
use serenity::{
    client::bridge::gateway::ShardId,
    constants::GATEWAY_VERSION,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{
        channel::{ChannelType, Message},
        guild::Role,
        id::RoleId,
        prelude::OnlineStatus,
    },
    prelude::Context,
};
use tags::*;
use urlencoding::encode;

use crate::{
    models::{
        commands::{CommandCounter, FrankFurterResponse, ShardManagerContainer},
        discord::BasicUser,
    },
    save_settings,
    utils::{
        chat::{get_user, is_user},
        discord::{get_member, get_permissions, get_roles, get_top_colour},
        emotes::EMOTES,
    },
    InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings,
};

#[group]
#[commands(
    base64,
    checktoken,
    emotestealer,
    exchange,
    math,
    ping,
    purge,
    roleinfo,
    rustdoc,
    serverinfo,
    setup,
    tags,
    usages,
    userinfo
)]
#[description("**Utilities**")]
struct Utility;

#[command]
#[description("This will create a new server and add emotes to it which are used throughout the selfbot")]
async fn setup(ctx: &Context, msg: &Message) -> CommandResult {
    let res = ctx
        .http
        .create_guild(&serde_json::json!({
            "name": "Emote Support",
            "region": "us-west"
        }))
        .await;

    let guild = match res {
        Ok(guild) => guild,
        Err(why) => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Setup").content(format!("Couldn't create server\n{:?}", why))
                })
                .await;
        },
    };

    let mut ids = HashMap::new();
    for key in EMOTES.keys() {
        let emote = guild.create_emoji(&ctx.http, key, EMOTES.get(key).unwrap()).await?;
        ids.insert(emote.name, emote.id.0);
    }

    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.emoteserver = guild.id.0;
    settings.sb_emotes = ids;
    save_settings(&settings);

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.success().title("Setup").content("Successfully created emote support server")
        })
        .await
}

#[command]
#[aliases("ui")]
#[description("List information about a user")]
#[min_args(1)]
async fn userinfo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "User Info", "Loading information about the user")
        .await
        .unwrap();

    let arg = args.rest().to_lowercase();

    if !is_user(&arg) {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("User Info").content("Could not parse user ID")
            })
            .await;
    }

    let user = if let Ok(user) = get_user(&arg).parse::<u64>() {
        user
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("User Info").content("Could not parse user ID")
            })
            .await;
    };

    let user = if let Ok(user) = ctx.http.get_user(user).await {
        user
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("User Info").content("Couldn't get user")
            })
            .await;
    };

    let member = if !msg.is_private() {
        if let Ok(member) = get_member(ctx, msg.guild_id.unwrap(), user.id).await {
            Some(member)
        } else {
            None
        }
    } else {
        None
    };


    let roles = if let Some(member) = member.clone() {
        get_roles(ctx, msg.guild_id.unwrap(), &member).await
    } else {
        None
    };

    let colour = if let Some(roles) = roles.clone() {
        get_top_colour(roles)
    } else {
        None
    };

    let perms = if let Some(roles) = roles.clone() {
        Some(get_permissions(ctx, msg.guild_id.unwrap(), Some(&member.clone().unwrap()), Some(roles)).await)
    } else {
        None
    };

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            let bot_tag = if user.bot { " (BOT)" } else { "" };
            m.title("User Info")
                .field("Name", format!("{}{}", user.tag(), bot_tag), false)
                .thumbnail(user.face());

            if let Some(member) = member {
                let roles = member
                    .roles
                    .into_iter()
                    .map(|r| format!("<@&{}>", r.0))
                    .collect::<Vec<String>>();

                m.field("Roles", roles.join(", "), false);


                if let Some(perms) = perms {
                    m.field(
                        "Permissions",
                        format!("Bits: {}\n{}", perms.bits, perms.get_permission_names().join(", ")),
                        false,
                    );
                }

                if let Some(colour) = colour {
                    let color_hex = colour.hex();
                    let colour_rgb = format!("{}, {}, {}", colour.r(), colour.g(), colour.b());
                    m.field(
                        "Color",
                        format!("Raw: {}\nHex: {}\nRGB: {}", colour.0, color_hex, colour_rgb),
                        true,
                    )
                    .colour(colour);
                }
            }
            m.field("ID", user.id, true)
        })
        .await
}

#[command]
#[aliases("ri")]
#[description("List information about a role")]
#[only_in("guilds")]
#[min_args(1)]
async fn roleinfo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let role_name_inp = args.rest().to_lowercase();
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Role Info", "Loading information about the role")
        .await
        .unwrap();

    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();
    let roles = cached_guild
        .clone()
        .roles
        .into_iter()
        .filter(|(_, role)| role.name.to_lowercase().contains(&role_name_inp))
        .collect::<HashMap<RoleId, Role>>();
    let roles_eq = roles
        .clone()
        .into_iter()
        .filter(|(_, role)| role.name.to_lowercase().eq(&role_name_inp))
        .collect::<HashMap<RoleId, Role>>();

    let role = match roles.len() {
        0 => {
            return new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Role Info")
                        .content("Unable to find any roles that match the specified text")
                })
                .await;
        },
        1 => {
            let keys = roles.keys().collect::<Vec<&RoleId>>();
            let key = keys.get(0).unwrap();

            roles.get(key).unwrap()
        },
        _ => {
            if roles_eq.len() == 1 {
                let keys = roles_eq.keys().collect::<Vec<&RoleId>>();
                let key = keys.get(0).unwrap();

                roles_eq.get(key).unwrap()
            } else {
                let role_list = roles_eq
                    .into_iter()
                    .map(|r| format!("<@&{}> - {}", r.0, r.0))
                    .collect::<Vec<String>>();

                return new_msg
                    .update_tmp(ctx, |m: &mut MessageCreator| {
                        m.error().title("Role Info").content(format!(
                            "Too many roles found, try narrow your search request or use a RoleId.\nRole Ids:\n{}",
                            role_list.join("\n")
                        ))
                    })
                    .await;
            }
        },
    };

    let name = role.name.clone();
    let id = role.id;
    let colour = role.colour;
    let color_hex = colour.hex();
    let colour_rgb = format!("{}, {}, {}", colour.r(), colour.g(), colour.b());
    let hoisted = role.hoist;
    let mentionable = role.mentionable;
    let position = role.position;
    let permissions = role.permissions;

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.colour(colour)
                .title("Role Info")
                .field("Name", name, false)
                .field(
                    "Color",
                    format!("Raw: {}\nHex: {}\nRGB: {}", colour.0, color_hex, colour_rgb),
                    true,
                )
                .field("Hoisted", if hoisted { "True" } else { "False" }, true)
                .field("Mentionable", if mentionable { "True" } else { "False" }, true)
                .field("Position", position, true)
                .field("ID", id, true);

            if !permissions.is_empty() {
                m.field(
                    "Permissions",
                    format!("Bits: {}\n{}", permissions.bits, permissions.get_permission_names().join(", ")),
                    false,
                );
            }

            m
        })
        .await
}

#[command]
#[aliases("server", "guild", "guildinfo", "si")]
#[description("List information about a guild")]
#[only_in("guilds")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Server Info", "Loading information about the guild")
        .await
        .unwrap();

    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();

    let owner = cached_guild.owner_id.to_user(&ctx.http).await?;

    let emotes = {
        let data = ctx.data.read().await;
        let settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;

        settings.sb_emotes.clone()
    };

    let mut animated_emotes = 0;
    let mut regular_emotes = 0;
    for emote in &cached_guild.emojis {
        if emote.1.animated {
            animated_emotes += 1;
        } else {
            regular_emotes += 1;
        };
    }

    let emoji_limit = cached_guild.premium_tier.num() * 50 + 50;
    let emote_string = format!(
        "Regular: {}/{}\nAnimated: {}/{}",
        regular_emotes, emoji_limit, animated_emotes, emoji_limit
    );

    let mut text_channels = 0;
    let mut voice_channels = 0;
    for channel in &cached_guild.channels {
        let channel = channel.1;

        if channel.kind == ChannelType::Text {
            text_channels += 1;
        } else if channel.kind == ChannelType::Voice {
            voice_channels += 1;
        }
    }
    let channels_text = format!(
        "<:text_channel:{}> {}\n<:voice_channel:{}> {}",
        emotes.get("text_channel").unwrap_or(&0),
        text_channels,
        emotes.get("voice_channel").unwrap_or(&0),
        voice_channels
    );

    let mut bot_count = 0;
    let mut human_count = 0;
    let mut online_count = 0;
    let mut idle_count = 0;
    let mut dnd_count = 0;
    let mut offline_count = 0;
    for member_result in &cached_guild.members {
        if member_result.1.user.bot {
            bot_count += 1
        } else {
            human_count += 1
        };

        match cached_guild.presences.get(member_result.0) {
            Some(presence) => match presence.status {
                OnlineStatus::Online => online_count += 1,
                OnlineStatus::DoNotDisturb => dnd_count += 1,
                OnlineStatus::Idle => idle_count += 1,
                OnlineStatus::Offline => offline_count += 1,
                OnlineStatus::Invisible => offline_count += 1,
                _ => {},
            },
            None => {
                offline_count += 1;
            },
        }
    }

    let member_count = bot_count + human_count;
    let member_string = format!(
        "<:status_online:{}> {} • <:status_idle:{}> {} • <:status_dnd:{}> {} • <:status_offline:{}> {}\n{} humans \
         \n{} bots\n{} total",
        emotes.get("status_online").unwrap_or(&0),
        online_count,
        emotes.get("status_idle").unwrap_or(&0),
        idle_count,
        emotes.get("status_dnd").unwrap_or(&0),
        dnd_count,
        emotes.get("status_offline").unwrap_or(&0),
        offline_count,
        human_count,
        bot_count,
        member_count
    );
    let boosts_string = format!(
        "<:nitro_boost:{}> {}\nLevel {}",
        emotes.get("nitro_boost").unwrap_or(&0),
        cached_guild.premium_subscription_count,
        cached_guild.premium_tier.num()
    );

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.title("Server Info")
                .content(format!("**{}**", &cached_guild.name))
                .thumbnail(&cached_guild.icon_url().unwrap_or_default())
                .footer_text(format!("ID: {} | Created", cached_guild.id.0))
                .field("Emotes", emote_string, true)
                .field("Channels", channels_text, true)
                .field("Members", member_string, false)
                .field("Boosts", boosts_string, true)
                .field("Roles", format!("{} roles", cached_guild.roles.len()), true)
                .field("Owner", owner.tag(), true)
        })
        .await
}

#[command]
#[aliases("latency")]
#[description("Gets the current GET, POST and Shard latencies")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Ping").content("ERROR: Couldn't get Shard Manager")
                })
                .await;
        },
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Ping").content("ERROR: No shard found")
                })
                .await;
        },
    };

    let shard_latency = match runner.latency {
        Some(latency) => format!("\nShard latency: {}ms", latency.as_millis()).to_string(),
        None => "".to_string(),
    };

    let gateway_url = format!("https://discord.com/api/v{}/gateway", GATEWAY_VERSION);
    let now = Instant::now();
    reqwest::get(&gateway_url).await?;
    let get_latency = now.elapsed().as_millis();

    let now = Instant::now();
    let sent_message = msg.channel_id.send_loading(ctx, "Ping", "Calculating latency").await;
    let post_latency = now.elapsed().as_millis();

    sent_message
        .unwrap()
        .update_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Ping").content(format!(
                "REST GET: {}ms\nREST POST: {}ms{}",
                get_latency, post_latency, shard_latency
            ))
        })
        .await
}

#[command]
#[aliases("count")]
#[description("Lists how many times commands have been used since the bot last restarted")]
async fn usages(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Usage");

            for (name, amount) in counter {
                m.field(name, amount, true);
            }

            m
        })
        .await
}

#[command]
#[description("Evaluate most mathmatical problems")]
#[usage("<expression>")]
#[example("3^(1 + 2)")]
#[min_args(1)]
async fn math(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let expr = args.rest();
    if let Ok(res) = meval::eval_str(expr) {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success()
                    .title("Math")
                    .content(format!("Equation: {}\nResult: {}", expr, res))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Math").content("Unable to evaluate equation")
            })
            .await
    }
}

#[command]
#[aliases("rust")]
#[description("Get a link to a specified libraries Rust Doc")]
#[usage("<library> [search]")]
#[example("serenity")]
#[example("std Result")]
#[min_args(1)]
async fn rustdoc(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let lib = args.single::<String>().unwrap().to_lowercase();

    let search = if !args.is_empty() {
        format!("?search={}", encode(args.rest()))
    } else {
        "".to_string()
    };

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Rust Doc")
                .field("Crates.io", format!("https://crates.io/crates/{}", lib), true)
                .field("docs.rs", format!("https://docs.rs/{}{}", lib, search), true)
        })
        .await
}

#[command]
#[aliases("exch")]
#[description("Get exchange rate for specified currency")]
#[usage("<amount> <from currency> [to currency]")]
#[example("20 NZD")]
#[example("500 JPY USD")]
#[min_args(2)]
#[max_args(3)]
async fn exchange(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Surely there's a better way to do this
    // Have to use String rather than &str bc
    // you can't .contains on Vec<&&str> with
    // a String or &str
    let currencies = vec![
        "AUD".to_string(),
        "BGN".to_string(),
        "BRL".to_string(),
        "CAD".to_string(),
        "CHF".to_string(),
        "CNY".to_string(),
        "CZK".to_string(),
        "DKK".to_string(),
        "EUR".to_string(),
        "GBP".to_string(),
        "HKD".to_string(),
        "HRK".to_string(),
        "HUF".to_string(),
        "IDR".to_string(),
        "ILS".to_string(),
        "INR".to_string(),
        "ISK".to_string(),
        "JPY".to_string(),
        "KRW".to_string(),
        "MXN".to_string(),
        "MYR".to_string(),
        "NOK".to_string(),
        "NZD".to_string(),
        "PHP".to_string(),
        "PLN".to_string(),
        "RON".to_string(),
        "RUB".to_string(),
        "SEK".to_string(),
        "SGD".to_string(),
        "THB".to_string(),
        "TRY".to_string(),
        "ZAR".to_string(),
    ];

    let amount = if let Ok(amt) = args.single::<f64>() {
        amt
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Exchange").content("Invalid amount specified")
            })
            .await;
    };

    let from = args.single::<String>().unwrap().to_uppercase();
    let to_wrapped = args.single::<String>();

    if !currencies.contains(&from) {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Exchange").content("Invalid `from` currency speicifed")
            })
            .await;
    }

    let res = reqwest::get(&format!("https://api.frankfurter.app/latest?from={}&amount={}", from, amount))
        .await?
        .text()
        .await?;

    let res: FrankFurterResponse = serde_json::from_str::<FrankFurterResponse>(&res).expect("Couldn't parse response");

    match to_wrapped {
        Ok(to) => {
            let to = to.to_uppercase();

            if !currencies.contains(&to) {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.error().title("Exchange").content("Invalid `to` currency speicifed")
                    })
                    .await;
            }

            let val: f64 = res.rates[&to];

            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.title("Exchange")
                        .content(format!("{:.2} {} is roughly equal to {:.2} {}", amount, from, val, to))
                })
                .await;
        },
        Err(_) => {
            let mut rates: Vec<(String, f64)> = Vec::new();

            for (cur, val) in &res.rates {
                rates.push((cur.to_string(), *val));
            }

            rates.sort_by(|(cur1, _val1), (cur2, _val2)| cur1.partial_cmp(cur2).unwrap());
            let msg_count = (rates.len() as f64 / 9.0).ceil();
            let mut msgs = Vec::new();

            for idx in 0..msg_count as u64 {
                let mut msg = MessageCreator::default();

                msg.title("Exchange").content(&format!(
                    "**Base**\n{} {}\n\nExchange rates as of {}",
                    res.amount, res.base, res.date
                ));

                let field_count = min(rates.len() as u64, (idx + 1) * 9) - idx * 9;
                for i in 0..field_count {
                    let rate_idx = idx * 9 + i;
                    let rate = &rates[rate_idx as usize];

                    msg.field(&format!("**{}**", rate.0), &format!("{:.2}", rate.1), true);
                }

                msgs.push(msg);
            }

            return msg.channel_id.send_paginator_noret(ctx, msg, msgs).await;
        },
    }
}

#[command]
#[aliases("b64")]
#[description("Encode/decode base64")]
#[usage("<encode/decode> <message>")]
#[example("decode SW5vcmkgaXMgdGhlIGJlc3Qgd2FpZnU=")]
#[example("encode I agree")]
#[min_args(2)]
async fn base64(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mode = args.single::<String>().unwrap_or_default();
    let encode = match mode.to_lowercase().as_ref() {
        "encode" | "enc" => true,
        "decode" | "dec" => false,
        _ => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Base64").content("Invalid argument")
                })
                .await;
        },
    };

    let content = if encode {
        let msg = args.rest();
        let enc = base64::encode(msg);

        format!("Input: {}\nOutput: {}", msg, enc)
    } else {
        let enc_msg = args.rest();
        let dec_bytes = base64::decode(enc_msg).unwrap_or_default();
        let dec = match String::from_utf8(dec_bytes) {
            Ok(dec) => dec,
            Err(err) => {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.error().title("Base64").content(format!("Invalid UTF-8 sequence: {}", err))
                    })
                    .await;
            },
        };

        format!("Input: {}\nOutput: {:?}", enc_msg, dec)
    };

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.success().title("Base64").content(content))
        .await
}

#[command]
#[aliases("token")]
#[description("Check the validitiy of a token")]
#[usage("<token>")]
#[example("ODAyMTc5MzM1OTg0NTEzMDY0.YArduw.30nmw_xqSuUX6hzRAC_li05Jw3Q")]
#[num_args(1)]
async fn checktoken(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Token Checker", "Checking Token")
        .await
        .unwrap();

    let tkn = args.rest();

    let res = reqwest::Client::new()
        .get("https://discord.com/api/v8/users/@me")
        .header("Authorization", tkn)
        .send()
        .await;

    let content = match res {
        Ok(res) => match res.status().as_u16() {
            401 => "Invalid token".to_string(),
            200 => {
                let user = serde_json::from_str::<BasicUser>(&res.text().await.unwrap()).unwrap();

                let bot_tag = if user.is_bot() {
                    let end = if user.is_verified_bot() { " - Verified" } else { "" };
                    format!("(Bot{})", end)
                } else {
                    "".to_string()
                };

                let mut extras = Vec::new();

                if user.is_early_verified_bot_dev() {
                    extras.push("Early Verified Bot Dev");
                }

                if user.is_partner_server_owner() {
                    extras.push("Partner Server Owner");
                }

                let mut content = format!(
                    "Tag: {}#{} {}\nID: {}\nEmail: {}\nPhone: {}\nVerified: {}\n2FA: {}\nNitro: {}",
                    user.username,
                    user.discriminator,
                    bot_tag,
                    user.id,
                    user.email,
                    user.phone.clone().unwrap_or_else(|| "Not set".to_string()),
                    user.verified.to_string(),
                    if user.mfa_enabled { "Enabled" } else { "Disabled" },
                    user.nitro_str(),
                );

                if !extras.is_empty() {
                    content = format!("{}\nExtras: {}", content, extras.join(", "));
                }

                return new_msg
                    .update_tmp(ctx, |m: &mut MessageCreator| {
                        m.title("Token Checker").thumbnail(user.avatar_url()).content(content)
                    })
                    .await;
            },
            _ => {
                format!("Unexpected response: {}", res.status().as_u16())
            },
        },
        Err(_) => "Unable to check token with Discord, try again in a minute.".to_string(),
    };

    new_msg
        .update_tmp(ctx, |m: &mut MessageCreator| m.error().title("Token Checker").content(content))
        .await
}
