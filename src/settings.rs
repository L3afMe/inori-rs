use core::future::Future;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use tokio::io::{self, AsyncBufReadExt};

use crate::{
    models::{
        discord::BasicUser,
        settings::{AutoDeleteConfig, GiveawayConfig, PfpSwitcher, Settings, SlotBotConfig},
    },
    try_or_msg,
    utils::consts::{AUTHOR_DISC, PROG_NAME},
};

pub async fn get_valid_input<T, D: ToString, F, Fut>(msg: D, f: F) -> Option<T>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Option<T>>, {
    let mut reader = io::BufReader::new(io::stdin());

    #[allow(while_true)]
    while true {
        println!("\nPlease input {}", msg.to_string());
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        reader.read_line(&mut buffer).await.unwrap();
        let input = &buffer[..buffer.len() - 1];
        let input = input.trim();

        if let Some(res) = f(input.to_string()).await {
            return Some(res);
        }

        println!("Invalid input specified, please try again");
    }

    // This should never happen but it makes the
    // compiler happy
    None
}

pub async fn setup_settings() -> Settings {
    println!(
        "Welcome to {}!\n\nI was unable to find a config file\nso I'll walk you through making a new one.\n\nIf you \
         have any issues during setup or\nwhile using the bot, feel free to contact\n{} on Discord for support!\n\nIf \
         you wish to stop the bot at any time,\npress Control+C and the bot will force stop.
        \nThis will only take a minute!",
        PROG_NAME, AUTHOR_DISC
    );

    let user_token: String = get_valid_input("your Discord token", async move |tkn: String| {
        let res = reqwest::Client::new()
            .get("https://discord.com/api/v8/users/@me")
            .header("Authorization", &tkn)
            .send()
            .await;

        match res {
            Ok(res) => match res.status().as_u16() {
                401 => {
                    println!("\nInvalid token response from Discord");

                    None
                },
                200 => {
                    let user = serde_json::from_str::<BasicUser>(&res.text().await.unwrap()).unwrap();
                    println!("\nNice to meet you {}#{}!", user.username, user.discriminator);

                    Some(tkn)
                },
                _ => {
                    println!("\nUnexpected response: {}", res.status().as_u16());

                    None
                },
            },
            Err(_) => {
                println!("\nUnable to check token with Discord,\ncheck your internet connection and try again.");

                None
            },
        }
    })
    .await
    .unwrap_or_else(|| "<TOKEN HERE>".to_string());

    let command_prefix = get_valid_input("preferred prefix (Default: ~)", async move |prefix: String| Some(prefix))
        .await
        .unwrap_or_else(|| "~".to_string());

    let global_nsfw_level: u8 = get_valid_input(
        "NSFW level for channels not marked as NSFW (Default: 1)\n0 - Strict filtering\n1 - Moderate filtering\n2 - \
         Disable filtering",
        async move |level: String| {
            if let Ok(level) = level.parse::<u8>() {
                if level <= 2 {
                    if level == 2 {
                        println!("How promiscuous you are");
                    }

                    Some(level)
                } else {
                    None
                }
            } else {
                None
            }
        },
    )
    .await
    .unwrap_or(1);

    let is_male: bool = get_valid_input(
        "your gender, 'male' or 'female' (There are only two genders)\nUsed for referring to yourself, i.e. himself, \
         herself, etc",
        async move |input: String| {
            let gender = input.to_lowercase();

            if gender.eq("male") {
                Some(true)
            } else if gender.eq("female") {
                Some(false)
            } else {
                None
            }
        },
    )
    .await
    .unwrap_or(true);

    let nitrosniper: bool = get_valid_input(
        "if you would like to snipe nitro.\n1 - Enabled\n2 - Disabled",
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let send_embeds: bool = get_valid_input(
        "if you would like to use rich embeds.\n1 - Enabled\n2 - Disabled",
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let slotbot_enabled: bool = get_valid_input(
        "if you would like to snipe SlotBot wallet drops.\n1 - Enabled\n2 - Disabled",
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let slotbot_dynamic_prefix: bool = if slotbot_enabled {
        get_valid_input(
            "if you would like to use dymamix prefixes for SlotBot, this will make it slightly slower and only needs \
             to be enabled if you're in a server which has changed the prefix.\n1 - Enabled\n2 - Disabled",
            async move |input: String| match input.parse::<u8>() {
                Ok(op) => {
                    if (1..=2).contains(&op) {
                        Some(op == 1)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            },
        )
        .await
        .unwrap_or(false)
    } else {
        false
    };

    let slotbot_mode: u8 = if slotbot_enabled {
        get_valid_input(
            format!(
                "the prefered snipe mode.\n0 - All servers\n1 - Whitelist; Only in specified servers ({}help slotbot \
                 whitelist)\n2 - Blacklist; Only not in specified servers ({}help slotbot blacklist)",
                command_prefix, command_prefix
            ),
            async move |input: String| match input.parse::<u8>() {
                Ok(op) => {
                    if op <= 2 {
                        Some(op)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            },
        )
        .await
        .unwrap_or(0)
    } else {
        0
    };

    let pfp_switcher_enabled: bool = get_valid_input(
        format!(
            "if you would like to enable profile picture switching ({}help pfpswitcher).\n1 - Enabled\n2 - Disabled",
            command_prefix
        ),
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let pfp_switcher_delay: u32 = if pfp_switcher_enabled {
        get_valid_input(
            "the delay in minutes between switching profile pictures, minimum of 10 minutes.",
            async move |input: String| match input.parse::<u32>() {
                Ok(op) => {
                    if op >= 10 {
                        Some(op)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            },
        )
        .await
        .unwrap_or(45)
    } else {
        45
    };

    let pfp_switcher_mode: u8 = if pfp_switcher_enabled {
        get_valid_input(
            "the prefered switching method.\n0 - Random\n1 - Alphabetical (Not currently implemented)",
            async move |input: String| match input.parse::<u8>() {
                Ok(op) => {
                    if op <= 1 {
                        Some(op)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            },
        )
        .await
        .unwrap_or(0)
    } else {
        0
    };

    let giveaway_enabled: bool = get_valid_input(
        "if you would like to automatically join giveaways.\n1 - Enabled\n2 - Disabled",
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let giveaway_delay: u64 = if pfp_switcher_enabled {
        get_valid_input("the delay in seconds before joining a giveaway.", async move |input: String| {
            match input.parse::<u64>() {
                Ok(op) => Some(op),
                Err(_) => None,
            }
        })
        .await
        .unwrap_or(20)
    } else {
        120
    };

    let autodelete_enabled: bool = get_valid_input(
        "if you would like messages to automatically delete.\n1 - Enabled\n2 - Disabled",
        async move |input: String| match input.parse::<u8>() {
            Ok(op) => {
                if (1..=2).contains(&op) {
                    Some(op == 1)
                } else {
                    None
                }
            },
            Err(_) => None,
        },
    )
    .await
    .unwrap_or(false);

    let autodelete_delay: u64 = if pfp_switcher_enabled {
        get_valid_input(
            "the delay in seconds before deleting bot messages. Note: this doesn't include messages like interations, \
             tags, etc.",
            async move |input: String| match input.parse::<u64>() {
                Ok(op) => {
                    if op >= 1 {
                        Some(op)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            },
        )
        .await
        .unwrap_or(10)
    } else {
        10
    };

    let pfp_switcher: PfpSwitcher = PfpSwitcher {
        enabled: pfp_switcher_enabled,
        delay:   pfp_switcher_delay,
        mode:    pfp_switcher_mode,
    };

    let giveaway: GiveawayConfig = GiveawayConfig {
        enabled: giveaway_enabled,
        delay:   giveaway_delay,
    };
    let slotbot: SlotBotConfig = SlotBotConfig {
        enabled: slotbot_enabled,
        dynamic_prefix: slotbot_dynamic_prefix,
        mode: slotbot_mode,
        whitelisted_guilds: Vec::new(),
        blacklisted_guilds: Vec::new(),
    };

    let autodelete: AutoDeleteConfig = AutoDeleteConfig {
        enabled: autodelete_enabled,
        delay:   autodelete_delay,
    };

    // Clone prefix so we can use in the message below
    // after it's been moved into the config
    let prefix = command_prefix.clone();
    let settings: Settings = Settings {
        user_token,
        command_prefix,
        global_nsfw_level,
        is_male,
        send_embeds,
        emoteserver: 0,
        nitrosniper,
        pfp_switcher,
        giveaway,
        autodelete,
        slotbot,
        tags: HashMap::new(),
        sb_emotes: HashMap::new(),
    };

    if let Err(why) = _save_settings(&settings) {
        panic!("[Config] Error while saving config: {}", why);
    }

    println!(
        "[Config] Config setup and ready to use\n[Bot] Make sure to run {}setup which will create an new server and \
         add emotes that are used throughout the bot",
        prefix
    );

    return settings;
}
pub fn load_settings() -> Result<Settings, String> {
    let mut contents = String::new();

    let mut f = match File::open("config.toml") {
        Ok(file) => file,
        Err(why) => {
            if let std::io::ErrorKind::NotFound = why.kind() {
                return Err("Unable to find 'config.toml', copy 'config.toml.bak' and setup config".to_string());
            }

            return Err(format!("Unknown error occured while opening 'config.toml'\n[Config] {}", why));
        },
    };

    if let Err(why) = f.read_to_string(&mut contents) {
        if let std::io::ErrorKind::NotFound = why.kind() {
            return Err("Unable to find 'config.toml', copy 'config.toml.bak' and setup config".to_string());
        }

        return Err(format!("Unknown error occured while opening 'config.toml'.\n[Config] {}", why));
    }

    let res: Settings = match toml::from_str(&contents) {
        Ok(res) => res,
        Err(why) => return Err(format!("Unable to deserialize settings.\n[Config] Error: {}", why)),
    };

    println!("[Config] Load successful");

    Ok(res)
}

pub fn save_settings(settings: &Settings) {
    match _save_settings(settings) {
        Ok(_) => {},
        Err(err) => println!("[Config] Error while saving config: {}", err),
    }
}

pub fn _save_settings(settings: &Settings) -> Result<(), String> {
    let contents = try_or_msg!(toml::to_string(settings), "Unable to serialize config".to_string());

    let mut f = match File::create("config.toml") {
        Ok(file) => file,
        Err(why) => return Err(format!("Unable to create 'config.toml'\n[Config] {}", why)),
    };

    try_or_msg!(
        f.write_all(&contents.as_bytes()),
        "Unable to write config to buffer".to_string()
    );

    try_or_msg!(f.sync_data(), "Unable to write config to 'config.toml'".to_string());
    println!("[Config] Save successful");
    Ok(())
}
