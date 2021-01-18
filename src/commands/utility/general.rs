extern crate meval;
extern crate urlencoding;

use urlencoding::encode;

use rand::Rng;

use serenity::{
    client::bridge::gateway::ShardId,
    constants::GATEWAY_VERSION,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::{ChannelType, Message},
        id::MessageId,
        prelude::{OnlineStatus, ReactionType},
        user::User,
    },
    prelude::*,
    utils::Colour,
};

use std::{cmp::min, time::Instant};

use tokio::{
    runtime::Builder,
    time::{delay_for, Duration},
};

use crate::{
    models::commands::{CommandCounter, FrankFurterResponse, ShardManagerContainer},
    save_settings,
    utils::{
        chat,
        chat::{
            default_embed, say, say_embed, say_error, send, send_embed, send_embed_paginator,
            send_loading,
        },
        emotes::EMOTES,
    },
    Settings,
};

#[command]
#[description(
    "This will create a new server and add emotes to it which are used throughout the selfbot"
)]
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
            return say_error(
                ctx,
                &msg.channel_id,
                "Setup",
                &format!("Couldn't create server\n{:?}", why),
            )
            .await
        }
    };

    for key in EMOTES.keys() {
        guild
            .create_emoji(&ctx.http, key, EMOTES.get(key).unwrap())
            .await?;
    }

    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    settings.emoteserver = guild.id.0;
    save_settings(&settings);

    Ok(())
}

async fn print_av(ctx: &Context, msg: &Message, user: &User) -> CommandResult {
    let av = match user.avatar_url() {
        Some(av) => av,
        None => {
            return say_error(
                ctx,
                &msg.channel_id,
                "Avatar",
                &format!("Unable to get {}'s avatar URL", user.name),
            )
            .await
        }
    };

    let mut embed = default_embed("Avatar");
    embed
        .description(format!(
            "{}#{}'s profile picture",
            user.name,
            format!("{:0>4}", user.discriminator)
        ))
        .image(av)
        .footer(|f| f.text(format!("ID: {}", user.id.0)));

    say_embed(ctx, &msg.channel_id, embed).await
}

#[command]
#[aliases("av", "pfp")]
#[description("Gets the pfp(s) of the mentioned user(s), if no one mentioned then gets self")]
#[usage("[@users]")]
#[example("@L3af#0001")]
async fn avatar(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.len() > 0 {
        for mention in &msg.mentions {
            print_av(ctx, msg, mention).await.unwrap_or(());
        }

        return Ok(());
    } else {
        return print_av(ctx, msg, &msg.author).await;
    }
}

#[command]
#[aliases("ratelimit", "rl")]
#[description("List Discords ratelimits")]
async fn ratelimits(ctx: &Context, msg: &Message) -> CommandResult {
    let mut embed = default_embed("Discord Ratelimits");
    embed
        .description("Ratelimits are in request/seconds")
        .field("REST API", "Overall: 50/1\nPer Account", false)
        .field("[POST] Message", "5/5\nPer Channel", true)
        .field("[DELETE] Message", "5/1\nPer Channel", true)
        .field("[PUT/DELETE] Reaction", "1/0.25\nPer Channel", true)
        .field("[PATCH] Channel", "2/600\nPer Channel", true)
        .field("[PATCH] Member", "10/10\nPer Guild", true)
        .field("[PATCH] Member Nick", "1/1\nPer Guild", true)
        .field("[PATCH] Username", "2/3600\nPer Account", true)
        .field("WebSocket", "Overall: 120/60\nPer Account", false)
        .field("Gateway Connect", "1/5\nPer Account", true)
        .field("Presence Update", "5/60\nPer Account", true);

    send_embed(ctx, &msg.channel_id, embed).await;
    Ok(())
}

#[command]
#[description("Gives information about a guild")]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let mut message = send_loading(
        ctx,
        &msg.channel_id,
        "Server Info",
        "Loading information about the guild",
    )
    .await;

    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();

    let mut embed = default_embed("Server Info");

    embed
        .title(&cached_guild.name)
        .thumbnail(&cached_guild.icon_url().unwrap_or(String::new()))
        .color(Colour::BLURPLE)
        .footer(|f| f.text(format!("ID: {} • Created", cached_guild.id.0)))
        .timestamp(&msg.guild_id.unwrap().created_at());

    let owner: User = cached_guild.owner_id.to_user(&ctx).await?;
    embed.author(|f| {
        f.name(format!(
            "{}#{} 👑",
            owner.name,
            format!("{:0>4}", owner.discriminator)
        ))
        .icon_url(owner.avatar_url().unwrap_or(String::new()))
    });

    let mut animated_emotes = 0;
    let mut regular_emotes = 0;
    for emoji in cached_guild.emojis {
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
    embed.field("Emotes", emote_string, true);

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
        "<:text_channel:797147634284101693> {}\n\
        <:voice_channel:797147798209429535> {}",
        text_channels, voice_channels
    );
    embed.field("Channels", channels_text, true);

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
                _ => {}
            },
            None => {
                offline_count += 1;
            }
        }
    }
    let member_count = bot_count + human_count;
    let member_string = format!(
        "<:status_online:797127703752081408> {} • \
        <:status_idle:797127751764410408> {} • \
        <:status_dnd:797127797415084063> {} • \
        <:status_offline:797127842235678731> {}\n\
        {} humans \n\
        {} bots\n\
        {} total",
        online_count, idle_count, dnd_count, offline_count, human_count, bot_count, member_count
    );

    embed.field("Members", member_string, false);

    let boosts_string = format!(
        "<:nitro_boost:797148982358048798> {}\nLevel {}",
        cached_guild.premium_subscription_count,
        cached_guild.premium_tier.num()
    );
    embed.field("Boosts", boosts_string, true);

    embed.field("Roles", format!("{} roles", cached_guild.roles.len()), true);

    message
        .edit(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
#[aliases("latency")]
#[description("Gets the current GET, POST and Shard latencies")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            say_error(
                ctx,
                &msg.channel_id,
                "Ping",
                "ERROR: Couldn't get Shard Manager",
            )
            .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => return say_error(ctx, &msg.channel_id, "Ping", "ERROR: No shard found").await,
    };

    let mut shard_latency = match runner.latency {
        Some(latency) => format!("\nShard latency: {}ms", latency.as_millis()).to_string(),
        None => "".to_string(),
    };
    shard_latency = format!("{}ms", shard_latency);

    let gateway_url = format!("https://discord.com/api/v{}/gateway", GATEWAY_VERSION);

    let now = Instant::now();
    reqwest::get(&gateway_url).await?;
    let get_latency = now.elapsed().as_millis();

    let now = Instant::now();
    let mut sent_message = send_loading(ctx, &msg.channel_id, "Ping", "Calculating latency").await;
    let post_latency = now.elapsed().as_millis();

    sent_message
        .edit(&ctx, |m| {
            m.embed(|e| {
                e.0 = default_embed("Ping").0;

                e.description(format!(
                    "REST GET: {}ms\nREST POST: {}ms{}",
                    get_latency, post_latency, shard_latency
                ))
            })
        })
        .await?;

    Ok(())
}

#[command]
#[aliases("count")]
#[description("Lists how many times commands have been used since the bot last restarted")]
async fn usages(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    let mut embed = default_embed("Command Usages");
    for (name, amount) in counter {
        embed.field(name, amount, true);
    }

    say_embed(ctx, &msg.channel_id, embed).await
}

#[command]
#[aliases("prunechat", "clearchat")]
#[description("Deletes a specified amount of messages")]
#[usage("<amount>")]
#[example("20")]
#[required_permissions("MANAGE_MESSAGES")]
#[num_args(1)]
async fn purgechat(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let delete_num = args.single::<u64>();
    match delete_num {
        Err(_) => {
            say(
                ctx,
                &msg.channel_id,
                "Purge Chat",
                ":no_entry_sign: The value provided was not a valid number",
            )
            .await?;
        }
        Ok(delete_n) => {
            let mut find_msg = send_loading(
                ctx,
                &msg.channel_id,
                "Purge Chat",
                &format!("Finding and deleting {} messages", delete_n),
            )
            .await;

            let channel = &ctx
                .http
                .get_channel(msg.channel_id.0)
                .await
                .unwrap()
                .guild()
                .unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(delete_n))
                .await?;
            let message_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            for message_id in message_ids {
                ctx.http
                    .delete_message(msg.channel_id.0, message_id.0)
                    .await?;
            }

            find_msg
                .edit(ctx, |m| {
                    m.embed(|e| {
                        e.0 = default_embed("Purge Chat").0;

                        e.description(format!(":white_check_mark: Deleted {} messages", delete_n))
                    })
                })
                .await?;

            delay_for(Duration::from_secs(5)).await;
            ctx.http
                .delete_message(msg.channel_id.0, find_msg.id.0)
                .await?;
        }
    }

    Ok(())
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
            say(
                ctx,
                &msg.channel_id,
                "Purge",
                ":no_entry_sign: The value provided is not a valid number",
            )
            .await?;
        }
        Ok(delete_n) => {
            let mut find_msg = send_loading(
                ctx,
                &msg.channel_id,
                "Purge",
                &format!("Finding and deleting {} messages", delete_n),
            )
            .await;

            let channel = &ctx
                .http
                .get_channel(msg.channel_id.0)
                .await
                .unwrap()
                .guild()
                .unwrap();

            let messages = channel
                .messages(ctx, |r| r.before(&msg.id).limit(100))
                .await?;

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
                        println!("Here!");
                        /*ctx.http
                        .delete_message(message.channel_id.0, message.id.0)
                        .await
                        .unwrap_or(());*/
                    });

                    purge_count += 1;
                }

                if purge_count >= delete_n {
                    break;
                }
            }

            let end = if delete_n == 1 { "message" } else { "messages" };
            find_msg
                .edit(ctx, |m| {
                    m.embed(|e| {
                        e.0 = default_embed("Purge").0;

                        e.description(format!(
                            ":white_check_mark: Deleted {} {}",
                            purge_count, end
                        ))
                    })
                })
                .await?;

            chat::delete(ctx, &find_msg).await?;
        }
    }

    Ok(())
}

#[command]
#[description("Evaluate most mathmatical problems")]
#[usage("<expression>")]
#[example("3^(1 + 2)")]
#[min_args(1)]
async fn math(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let expr = args.rest();

    let res = meval::eval_str(expr).unwrap();

    say(
        ctx,
        &msg.channel_id,
        "Math",
        &format!("Espression: {}\nResult: {}", expr, res),
    )
    .await
}

#[command]
#[description("Run a poll, options split with `|`. Max of 10 options")]
#[usage("<question> | <option> | <option> | <option>")]
#[example("Do you like chocolate? | Yes | No")]
async fn poll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = args.rest();
    let spl = args.split('|').collect::<Vec<&str>>();
    if args.contains("|") && spl.len() >= 3 && spl.len() <= 11 {
        let emojis = (0..(spl.len() - 1))
            .map(|i| std::char::from_u32('🇦' as u32 + i as u32).expect("Failed to format emoji"))
            .collect::<Vec<_>>();

        let poll_msg = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.0 = default_embed("Poll").0;

                    e.description(format!("**{}**", spl[0]));

                    for i in 1..spl.len() {
                        e.field(emojis[i - 1], spl[i], true);
                    }

                    e
                })
            })
            .await?;

        for &emoji in &emojis {
            poll_msg
                .react(&ctx.http, ReactionType::Unicode(emoji.to_string()))
                .await?;
        }
    } else {
        say(ctx, &msg.channel_id, "Poll", "Invalid arguments").await?;
    }

    Ok(())
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
    let search = if args.len() > 0 {
        format!("?search={}", encode(args.rest()))
    } else {
        "".to_string()
    };

    let mut embed = default_embed("Rust Doc");
    embed
        .field(
            "Crates.io",
            format!("https://crates.io/crates/{}", lib),
            true,
        )
        .field(
            "docs.rs",
            format!("https://docs.rs/{}{}", lib, search),
            true,
        );

    say_embed(ctx, &msg.channel_id, embed).await
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
        return say_error(ctx, &msg.channel_id, "Exchange", "Invalid amount speicifed").await;
    };
    let from = args.single::<String>().unwrap().to_uppercase();
    let to_wrapped = args.single::<String>();

    if !currencies.contains(&from) {
        return say_error(
            ctx,
            &msg.channel_id,
            "Exchange",
            "Invalid `from` currency speicifed",
        )
        .await;
    }

    let res = reqwest::get(&format!(
        "https://api.frankfurter.app/latest?from={}&amount={}",
        from, amount
    ))
    .await?
    .text()
    .await?;
    let res: FrankFurterResponse =
        serde_json::from_str::<FrankFurterResponse>(&res).expect("Couldn't parse response");

    match to_wrapped {
        Ok(to) => {
            if !currencies.contains(&from) {
                return say_error(
                    ctx,
                    &msg.channel_id,
                    "Exchange",
                    "Invalid `to` currency speicifed",
                )
                .await;
            }

            let to = to.to_uppercase();
            let val: f64 = res.rates[&to];

            send(
                ctx,
                &msg.channel_id,
                "Exchange",
                &format!(
                    "{:.2} {} is roughly equal to {:.2} {}",
                    amount, from, val, to
                ),
            )
            .await;
        }
        Err(_) => {
            let mut rates: Vec<(String, f64)> = Vec::new();
            for (cur, val) in &res.rates {
                rates.push((cur.to_string(), *val));
            }

            rates.sort_by(|(cur1, _val1), (cur2, _val2)| cur1.partial_cmp(cur2).unwrap());

            let embed_count = (rates.len() as f64 / 9.0).ceil();
            let mut embeds = Vec::new();
            for idx in 0..embed_count as u64 {
                let mut embed = default_embed("Exchange");
                embed
                    .description(&format!(
                        "**Base**\n\
                            {} {}\n\n\
                            Exchange rates as of {}",
                        res.amount, res.base, res.date
                    ))
                    .footer(|f| f.text(&format!("Page {} of {}", idx + 1, embed_count)));

                let field_count = min(rates.len() as u64, (idx + 1) * 9) - idx * 9;
                for i in 0..field_count {
                    let rate_idx = idx * 9 + i;
                    let rate = &rates[rate_idx as usize];
                    embed.field(&format!("**{}**", rate.0), &format!("{:.2}", rate.1), true);
                }

                embeds.push(embed);
            }

            send_embed_paginator(ctx, msg, embeds).await?;
        }
    }

    Ok(())
}

#[command]
#[description(
    "Force set your Hype Squad house.\n\
    Available houses:\n\
    Bravery\n\
    Brilliance\n\
    Random"
)]
#[usage("<house>")]
#[example("Bravery")]
#[num_args(1)]
async fn hypesquad(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let houses = vec![
        "Bravery".to_string(),
        "Brilliance".to_string(),
        "Balance".to_string(),
    ];
    let houses_lower = houses
        .iter()
        .map(|h| h.to_lowercase())
        .collect::<Vec<String>>();

    let house = args.current().unwrap_or("").to_lowercase();
    let house_id = if house.to_lowercase().eq("random") {
        rand::thread_rng().gen_range(1..4)
    } else if houses_lower.contains(&house) {
        houses_lower.iter().position(|r| r.eq(&house)).unwrap_or(0) + 1
    } else {
        return say_error(
            ctx,
            &msg.channel_id,
            "Hype Squad Changer",
            "Invalid house specified",
        )
        .await;
    };

    let res = reqwest::Client::new()
        .post("https://discord.com/api/v8/hypesquad/online")
        .header("Authorization", &ctx.http.token)
        .json(&serde_json::json!({ "house_id": house_id }))
        .send()
        .await;

    return match res {
        Ok(res) => {
            let status = res.status().as_u16();
            if status == 204 {
                say(
                    ctx,
                    &msg.channel_id,
                    "Hype Squad Changer",
                    &format!("Set house to {}", house),
                )
                .await
            } else {
                say_error(
                    ctx,
                    &msg.channel_id,
                    "Hype Squad Changer",
                    &format!("Invalid response status: {}", status),
                )
                .await
            }
        }
        Err(_) => {
            say_error(
                ctx,
                &msg.channel_id,
                "Hype Squad Changer",
                "Error occurred while changing house",
            )
            .await
        }
    };
}
