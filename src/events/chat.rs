use once_cell::sync::Lazy;
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

use crate::models::{commands::CommandCounter, settings::Settings};

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

        println!("[Command] Running '{}'", command_name);
    }

    amatch
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, command_name: &str, res: CommandResult) {
    if !msg.attachments.is_empty() {
        msg.delete(&ctx.http).await.unwrap();
    }

    match res {
        Ok(()) => println!("[Command] Finished running '{}'", command_name),
        Err(why) => println!("[Command] Finishing running '{}' with error {:?}", command_name, why),
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
                            println!(
                                "[Sniper] Successfully sniped nitro in DM's with {}#{}",
                                msg.author.name, msg.author.discriminator
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

                            println!(
                                "[Sniper] Successfully sniped nitro in [{} > {}] from {}#{}",
                                guild_name, channel_name, msg.author.name, msg.author.discriminator
                            )
                        }
                    },
                    StatusCode::METHOD_NOT_ALLOWED => {
                        println!("[Sniper] There was an error on Discord's side.");
                    },
                    StatusCode::NOT_FOUND => {
                        println!("[Sniper] Code was fake or expired.");
                    },
                    StatusCode::BAD_REQUEST => {
                        println!("[Sniper] Code was already redeemed.");
                    },
                    StatusCode::TOO_MANY_REQUESTS => {
                        println!("[Sniper] Ratelimited.");
                    },
                    unknown => {
                        println!("[Sniper] Received unknown response ({})", unknown.as_str());
                    },
                }
            } else {
                println!("[Sniper] Erroring while POSTing nitro code");
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

        let res = reqwest::Client::new()
            .post(&format!("https://discord.com/api/v8/channels/{}/messages", msg.channel_id.0))
            .header("Authorization", &ctx.http.token)
            .json(&serde_json::json!({ "content": format!("{}grab", pfx) }))
            .send()
            .await;

        let sniped = match res {
            Ok(res) => res.status().as_u16() == 200,
            Err(_) => false,
        };

        let channel_name = match ctx.http.get_channel(msg.channel_id.0).await.unwrap() {
            Channel::Guild(channel) => channel.name,
            _ => "Unknown".to_string(),
        };

        let guild_name = guild_id.name(&ctx.cache).await.unwrap_or_else(|| "Unknown".to_string());

        let sniped_msg = if sniped {
            format!("Sent message in [{} > {}]", guild_name, channel_name)
        } else {
            "Failed to send message".to_string()
        };
        println!("[SlotBot] {}", sniped_msg);

        return;
    }

    if msg.author.id.0 == 294882584201003009
        && msg
            .content
            .to_string()
            .eq("<:yay:585696613507399692>   **GIVEAWAY**   <:yay:585696613507399692>")
    {
        let config = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
            settings.giveaway.clone()
        };

        if !config.enabled || msg.is_private() {
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

        println!(
            "[Giveaway] Detected giveaway in [{} > {}] waiting {} seconds",
            guild_name, channel_name, config.delay
        );

        tokio::time::delay_for(tokio::time::Duration::from_secs(config.delay)).await;
        msg.react(&ctx.http, ReactionType::Unicode("🎉".to_string())).await.unwrap();
        println!("[Giveaway] Joined giveaway");
    }
}
