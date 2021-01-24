use colored::Colorize;
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use reqwest::StatusCode;
use serenity::{
    framework::standard::{macros::hook, CommandResult},
    model::{
        channel::{Channel, Message},
        prelude::ReactionType,
    },
    prelude::Context,
};

use crate::{
    inori_error, inori_info, inori_success,
    models::{commands::CommandCounter, settings::Settings},
};

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    let amatch = msg
        .author
        .id
        .to_string()
        .eq(&ctx.http.get_current_user().await.unwrap().id.to_string());

    if amatch {
        let mut data = ctx.data.write().await;
        let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;

        if msg.attachments.is_empty() {
            msg.delete(&ctx.http).await.unwrap();
        }

        inori_info!("Command", "Running '{}'", command_name);
    }

    amatch
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, command_name: &str, res: CommandResult) {
    if !msg.attachments.is_empty() {
        msg.delete(&ctx.http).await.unwrap();
    }

    match res {
        Ok(()) => inori_info!("Command", "Finished running '{}'", command_name),
        Err(why) => inori_error!("Command", "Finishing running '{}' with error {:?}", command_name, why),
    }
}

static NITRO_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(discord.com/gifts/|discordapp.com/gifts/|discord.gift/)[ ]*([a-zA-Z0-9]{16,24})").unwrap()
});
static SLOTBOT_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"`.*grab`").unwrap());

#[hook]
pub async fn normal_message(ctx: &Context, msg: &Message) {
    let nitro_enabled = {
        let data = ctx.data.read().await;
        let settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;

        settings.nitrosniper
    };

    if nitro_enabled {
        if let Some(code) = NITRO_REGEX.captures(&msg.content) {
            let code = code.get(2).unwrap().as_str();

            let res = reqwest::Client::new()
                .post(&format!(
                    "https://discordapp.com/api/v8/entitlements/gift-codes/{}/redeem",
                    code
                ))
                .header("Authorization", &ctx.http.token)
                .header("Content-Length", 0)
                .body(String::new())
                .send()
                .await;

            if let Ok(res) = res {
                match res.status() {
                    StatusCode::OK => {
                        if msg.is_private() {
                            inori_success!(
                                "Sniper",
                                "Successfully sniped nitro in DM's with {}#{}",
                                msg.author.name,
                                msg.author.discriminator,
                            )
                        } else {
                            let channel_name = match ctx.http.get_channel(msg.channel_id.0).await.unwrap() {
                                Channel::Guild(channel) => channel.name,
                                _ => "Unknown".to_string(),
                            };

                            let guild_name = msg
                                .guild_id
                                .unwrap()
                                .name(&ctx.cache)
                                .await
                                .unwrap_or_else(|| "Unknown".to_string());

                            inori_success!(
                                "Sniper",
                                "Successfully sniped nitro in [{} > {}] from {}#{}",
                                guild_name,
                                channel_name,
                                msg.author.name,
                                msg.author.discriminator,
                            )
                        }
                    },
                    StatusCode::METHOD_NOT_ALLOWED => {
                        inori_info!("Nitro Sniper", "There was an error on Discord's side.");
                    },
                    StatusCode::NOT_FOUND => {
                        inori_info!("Nitro Sniper", "Code was fake or expired.");
                    },
                    StatusCode::BAD_REQUEST => {
                        inori_info!("Nitro Sniper", "Code was already redeemed.");
                    },
                    StatusCode::TOO_MANY_REQUESTS => {
                        inori_info!("Nitro Sniper", "Ratelimited.");
                    },
                    unknown => {
                        inori_info!("Nitro Sniper", "Received unknown response ({})", unknown.as_str());
                    },
                }
            } else {
                inori_info!("Nitro Sniper", "Erroring while POSTing nitro code");
            }
        }
    }

    if msg.author.id.0 == 346353957029019648
        && msg
            .content
            .starts_with("Someone just dropped their wallet in this channel! Hurry and pick it up with")
    {
        let config = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
            settings.slotbot.clone()
        };

        if msg.is_private() || !config.enabled {
            return;
        }

        // Check if it's a guild above so this will never
        // throw error
        let guild_id = msg.guild_id.unwrap();

        if (config.mode == 1 && !config.whitelisted_guilds.contains(&guild_id.0))
            || (config.mode == 2 && config.blacklisted_guilds.contains(&guild_id.0))
        {
            return;
        }

        let pfx = if config.dynamic_prefix {
            if let Some(pfx) = SLOTBOT_PREFIX_REGEX.find(&msg.content) {
                msg.content[pfx.start() + 1..pfx.end() - 5].to_string()
            } else {
                "~".to_string()
            }
        } else {
            "~".to_string()
        };

        let new_msg = msg.channel_id.say(&ctx.http, format!("{}grab", pfx)).await;

        let channel_name = match ctx.http.get_channel(msg.channel_id.0).await.unwrap() {
            Channel::Guild(channel) => channel.name,
            _ => "Unknown".to_string(),
        };

        let guild_name = guild_id.name(&ctx.cache).await.unwrap_or_else(|| "Unknown".to_string());

        let sniped_msg = if new_msg.is_ok() {
            format!("Sent message in [{} > {}]", guild_name, channel_name)
        } else {
            "Failed to send message".to_string()
        };
        inori_info!("SlotBot", "{}", sniped_msg);

        if let Ok(msg) = new_msg {
            let _ = ctx.http.delete_message(msg.channel_id.0, msg.id.0).await;
        }

        return;
    }

    if msg.author.id.0 == 294882584201003009
        && msg
            .content
            .to_string()
            .eq("<:yay:585696613507399692>   **GIVEAWAY**   <:yay:585696613507399692>")
        && msg.embeds.len() == 1
    {
        let config = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
            settings.giveaway.clone()
        };

        if !config.enabled || msg.is_private() {
            return;
        }

        let guild_id = msg.guild_id.unwrap();
        let embed = msg.embeds.get(0).unwrap();
        let prize = embed.clone().author.unwrap().name;


        let is_whitelisted = config.whitelisted_words.is_empty()
            || config
                .whitelisted_words
                .into_iter()
                .filter(|w| prize.to_lowercase().contains(&w.to_lowercase()))
                .count()
                != 0;
        let is_blacklisted = config
            .blacklisted_words
            .into_iter()
            .filter(|w| prize.to_lowercase().contains(&w.to_lowercase()))
            .count()
            != 0;

        if (config.mode == 1 && !config.whitelisted_guilds.contains(&guild_id.0))
            || (config.mode == 2 && config.blacklisted_guilds.contains(&guild_id.0))
            || is_blacklisted
            || !is_whitelisted
        {
            return;
        }


        let channel_name = match ctx.http.get_channel(msg.channel_id.0).await.unwrap() {
            Channel::Guild(channel) => channel.name,
            _ => "Unknown".to_string(),
        };

        // Check if it's a guild above so unwrap() will
        // never throw error
        let guild_name = msg
            .guild_id
            .unwrap()
            .name(&ctx.cache)
            .await
            .unwrap_or_else(|| "Unknown".to_string());

        let min = std::cmp::min(config.min_delay, config.max_delay);
        let max = std::cmp::max(config.min_delay, config.max_delay);
        let delay = if min == max {
            min
        } else {
            rand::thread_rng().gen_range(min..max)
        };

        inori_info!(
            "Giveaway",
            "Detected giveaway in [{} > {}] waiting {} seconds",
            guild_name,
            channel_name,
            delay,
        );

        tokio::time::delay_for(tokio::time::Duration::from_secs(delay)).await;
        msg.react(&ctx.http, ReactionType::Unicode("🎉".to_string())).await.unwrap();
        inori_info!("Giveaway", "Joined giveaway");
    }
}
