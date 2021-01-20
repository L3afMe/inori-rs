#![feature(async_closure)]

mod commands;

mod models;
mod settings;
mod utils;

use std::{
    collections::{HashMap, HashSet},
    fs::DirEntry,
    io::Error,
    path::Path,
    sync::Arc,
};

use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use reqwest::StatusCode;
use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, Reason, StandardFramework,
    },
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        prelude::ReactionType,
    },
    prelude::*,
    utils::read_image,
};
use tokio::{
    task,
    time::{delay_for, Duration},
};

use crate::{
    commands::*,
    models::{
        commands::{CommandCounter, ShardManagerContainer},
        discord::{InoriChannelUtils, InoriMessageUtils, MessageCreator},
        settings::Settings,
    },
    settings::{load_settings, save_settings, setup_settings},
};

struct Handler;

#[async_trait]

impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!(
            "[Bot] Client started, connected as {}#{:0>4}",
            ready.user.name, ready.user.discriminator
        );

        spawn_pfp_change_thread(Arc::new(Mutex::new(ctx))).await;
    }
}

#[help]
#[individual_command_tip("**Help**\nTo get help for a specific command, subcommand or group use `help <command>`.")]
#[suggestion_text("**Error** Unable to find command. Similar commands: `{}`")]
#[no_help_available_text("**Error** Unable to find command")]
#[command_not_found_text("**Error** Unable to find command")]
#[dm_only_text("DM")]
#[guild_only_text("Servers")]
#[dm_and_guild_text("DM, Servers")]
#[max_levenshtein_distance(4)]
#[indention_prefix("-")]
#[lacking_permissions("Strike")]
#[lacking_role("Strike")]
#[wrong_channel("Strike")]
#[strikethrough_commands_tip_in_dm(
    "Commands with a ~~`strikethrough`~~ require certain lacking permissions to execute."
)]
#[strikethrough_commands_tip_in_guild(
    "Commands with a ~~`strikethrough`~~ require certain lacking permissions to execute."
)]
#[embed_error_colour(MEIBE_PINK)]
#[embed_success_colour(BLURPLE)]

async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;

    Ok(())
}

#[hook]

async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
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

async fn after(ctx: &Context, msg: &Message, command_name: &str, res: CommandResult) {
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

async fn normal_message(ctx: &Context, msg: &Message) {
    if let Some(code) = NITRO_REGEX.captures(&msg.content) {
        let code = code.get(2).unwrap().as_str();

        let res = reqwest::Client::new()
            .post(&format!(
                "https://discordapp.com/api/v8/entitlements/gift-codes/{}/redeem",
                code
            ))
            .header("Authorization", &ctx.http.token)
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

                        let guild_name = msg.guild_id.unwrap().name(&ctx.cache).await.unwrap_or("Unknown".to_string());

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

        // Check if it's a guild above so this will never throw error
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

        let guild_name = guild_id.name(&ctx.cache).await.unwrap_or("Unknown".to_string());

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

        // Check if it's a guild above so unwrap() will never throw error
        let guild_name = msg.guild_id.unwrap().name(&ctx.cache).await.unwrap_or("Unknown".to_string());

        println!(
            "[Giveaway] Detected giveaway in [{} > {}] waiting {} seconds",
            guild_name, channel_name, config.delay
        );

        tokio::time::delay_for(tokio::time::Duration::from_secs(config.delay)).await;

        msg.react(&ctx.http, ReactionType::Unicode("🎉".to_string())).await.unwrap();

        println!("[Giveaway] Joined giveaway");
    }
}

#[hook]

async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    ctx.http.delete_message(msg.channel_id.0, msg.id.0).await.unwrap();

    match error {
        DispatchError::Ratelimited(duration) => {
            let content = format!("Try this again in {} seconds.", duration.as_secs());

            println!("[Error] Ratelimit, {}", content);

            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| m.error().title("Ratelimit").content(content))
                .await;
        },

        DispatchError::CheckFailed(_, reason) => {
            if let Reason::User(err) = reason {
                let content = match err.as_ref() {
                    "nsfw_1" => "This server is not marked as NSFW and you've specified a NSFW image.\nThis can be \
                                 overriden by executing `nsfwfilter 1`"
                        .to_string(),
                    "nsfw_2" => "This server is not marked as NSFW and you've specified a NSFW image.\nThis can be \
                                 overriden by executing `nsfwfilter 2`"
                        .to_string(),
                    _ => {
                        let content = format!("Undocumted error, please report this to L3af#0001\nError: `{:?}``", err);

                        println!("{}", content);

                        content
                    },
                };

                let _ = msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| m.error().title("Error").content(content))
                    .await;
            }
        },

        DispatchError::TooManyArguments {
            max,
            given,
        } => {
            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Error")
                        .content(&format!("Too many args given!\nMaximum: {}, Given: {}", max, given))
                })
                .await;
        },

        DispatchError::NotEnoughArguments {
            min,
            given,
        } => {
            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Error")
                        .content(&format!("To few args given!\nMinimum: {}, Given: {}", min, given))
                })
                .await;
        },

        _ => {
            println!(
                "Unhandled dispatch error, please contact #L3af#0001 about this.\nError: {:?}",
                error
            );
        },
    };
}

#[hook]

async fn dynamic_prefix(ctx: &Context, _msg: &Message) -> Option<String> {
    let data = ctx.data.read().await;

    let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    Some(settings.clone().command_prefix)
}

async fn spawn_pfp_change_thread(ctx: Arc<Mutex<Context>>) {
    task::spawn(async move {
        loop {
            let start_time = std::time::SystemTime::now();

            loop {
                {
                    let ctx = ctx.lock().await;

                    let data = ctx.data.read().await;

                    let settings: tokio::sync::MutexGuard<'_, Settings> =
                        data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

                    if settings.pfp_switcher.enabled
                        && start_time.elapsed().unwrap().as_secs() >= (settings.pfp_switcher.delay * 60) as u64
                    {
                        let path = Path::new("./pfps/");

                        let ops = path.read_dir().unwrap().collect::<Vec<Result<DirEntry, Error>>>();

                        let new_pfp = match settings.pfp_switcher.mode {
                            0 => ops[rand::thread_rng().gen_range(0..ops.len())].as_ref(),
                            1 => {
                                // TODO: This shit

                                ops[rand::thread_rng().gen_range(0..ops.len())].as_ref()
                            },
                            _ => ops[rand::thread_rng().gen_range(0..ops.len())].as_ref(),
                        }
                        .unwrap();

                        let mut user = ctx.cache.current_user().await;

                        let avatar =
                            read_image(format!("./pfps/{}", new_pfp.file_name().into_string().unwrap())).unwrap();

                        user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();

                        println!("[PfpSwitcher] Changing pfps");

                        break;
                    }
                }

                delay_for(Duration::from_secs(60)).await;
            }
        }
    });
}

#[tokio::main]

async fn main() {
    let settings = if Path::exists(Path::new(&"config.toml")) {
        match load_settings() {
            Ok(settings) => settings,
            Err(why) => {
                println!("[Config] Error while loading config: {}", why);

                return;
            },
        }
    } else {
        setup_settings().await
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .prefix("")
                .dynamic_prefix(dynamic_prefix)
                .allow_dm(true)
                .case_insensitivity(true)
                .with_whitespace(true)
                .ignore_bots(false)
                .ignore_webhooks(true)
        })
        .before(before)
        .after(after)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&HELP)
        .group(&FUN_GROUP)
        .group(&NSFW_GROUP)
        .group(&IMAGEGEN_GROUP)
        .group(&INTERACTIONS_GROUP)
        .group(&CONFIG_GROUP)
        .group(&UTILITY_GROUP);

    println!("[Bot] Configured framework");

    let mut client = Client::builder(&settings.user_token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<CommandCounter>(HashMap::default());

        data.insert::<Settings>(Arc::new(Mutex::new(settings)));

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    println!("[Bot] Loaded client");

    println!("[Bot] Starting client");

    if let Err(why) = client.start().await {
        println!("[Bot] Client error: {:?}", why);
    }
}
