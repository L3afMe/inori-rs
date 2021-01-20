extern crate meval;
extern crate urlencoding;
use std::{cmp::min, time::Instant};

use rand::Rng;
use serenity::{
    client::bridge::gateway::ShardId,
    constants::GATEWAY_VERSION,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::{ChannelType, Message},
        id::MessageId,
        prelude::OnlineStatus,
    },
    prelude::Context,
};
use tokio::runtime::Builder;
use urlencoding::encode;

use crate::{
    models::commands::{CommandCounter, FrankFurterResponse, ShardManagerContainer},
    save_settings,
    utils::emotes::EMOTES,
    InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings,
};

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

    for key in EMOTES.keys() {
        guild.create_emoji(&ctx.http, key, EMOTES.get(key).unwrap()).await?;
    }

    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.emoteserver = guild.id.0;
    save_settings(&settings);

    Ok(())
}

#[command]
#[description("Gives information about a guild")]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Server Info", "Loading information about the guild")
        .await
        .unwrap();

    let cached_guild = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).await.unwrap();

    // let owner: User = cached_guild.owner_id.to_user(&ctx).await?;

    let mut animated_emotes = 0;
    let mut regular_emotes = 0;
    for emoji in &cached_guild.emojis {
        if emoji.1.animated {
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
        "<:text_channel:797147634284101693> {}\n<:voice_channel:797147798209429535> {}",
        text_channels, voice_channels
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
        "<:status_online:797127703752081408> {} • <:status_idle:797127751764410408> {} • \
         <:status_dnd:797127797415084063> {} • <:status_offline:797127842235678731> {}\n{} humans \n{} bots\n{} total",
        online_count, idle_count, dnd_count, offline_count, human_count, bot_count, member_count
    );
    let boosts_string = format!(
        "<:nitro_boost:797148982358048798> {}\nLevel {}",
        cached_guild.premium_subscription_count,
        cached_guild.premium_tier.num()
    );

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.title(&cached_guild.name)
                .thumbnail(&cached_guild.icon_url().unwrap_or_default())
                .footer_text(format!("ID: {} • Created", cached_guild.id.0));

            //.timestamp(&msg.guild_id.unwrap().created_at());

            // msg.author(|f| {
            // f.name(format!(
            // "{}#{} 👑",
            // owner.name,
            // format!("{:0>4}", owner.discriminator)
            // ))
            // .icon_url(owner.avatar_url().unwrap_or(String::new()))
            // });

            m.field("Emotes", emote_string, true)
                .field("Channels", channels_text, true)
                .field("Members", member_string, false)
                .field("Boosts", boosts_string, true)
                .field("Roles", format!("{} roles", cached_guild.roles.len()), true)
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
#[aliases("prune", "clear")]
#[description("Deletes a specified amount of messages sent by yourself")]
#[usage("<amount>")]
#[example("20")]
#[num_args(1)]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let delete_num = args.single::<u64>();

    match delete_num {
        Err(_) => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Purge")
                        .content(":no_entry_sign: The value provided is not a valid number")
                })
                .await;
        },
        Ok(delete_n) => {
            let mut find_msg = msg
                .channel_id
                .send_loading(ctx, "Purge", &format!("Finding and deleting {} messages", delete_n))
                .await
                .unwrap();

            let channel = &ctx.http.get_channel(msg.channel_id.0).await.unwrap().guild().unwrap();
            let messages = channel.messages(ctx, |r| r.before(&msg.id).limit(100)).await?;
            let mut purge_count = 0;

            let runtime = Builder::new()
                .threaded_scheduler()
                .core_threads(16)
                .thread_name("purge-thread")
                .thread_stack_size(1024 * 1024 / 2)
                .build()
                .unwrap();

            for message in messages {
                if message.is_own(&ctx.cache).await && purge_count < delete_n {
                    let ctx = ctx.clone();

                    runtime.spawn(async move {
                        ctx.http.delete_message(message.channel_id.0, message.id.0).await.unwrap_or(());
                    });

                    purge_count += 1;
                }

                if purge_count >= delete_n {
                    break;
                }
            }

            let end = if delete_n == 1 { "message" } else { "messages" };

            return find_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.title("Purge")
                        .content(format!(":white_check_mark: Deleted {} {}", delete_n, end))
                })
                .await;
        },
    }
}

#[command]
#[description("Evaluate most mathmatical problems")]
#[usage("<expression>")]
#[example("3^(1 + 2)")]
#[min_args(1)]
async fn math(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let expr = args.rest();
    let res = meval::eval_str(expr).unwrap();

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Math").content(format!("Espression: {}\nResult: {}", expr, res))
        })
        .await
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
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("Base64").content(content))
        .await
}
