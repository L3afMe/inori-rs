use crate::{
    models::{discord::BasicUser, settings::Settings},
    try_or_msg,
    utils::consts::{AUTHOR_DISC, PROG_NAME},
};

use tokio::io::{self, AsyncBufReadExt};

use core::future::Future;
use std::{
    fs::File,
    io::{Read, Write},
};

pub async fn get_valid_input<T, F, Fut>(msg: &str, f: F) -> Option<T>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let mut reader = io::BufReader::new(io::stdin());

    #[allow(while_true)]
    while true {
        println!("\nPlease input {}", msg);
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        reader.read_line(&mut buffer).await.unwrap();

        let input = &buffer[..buffer.len() - 1];

        if let Some(res) = f(input.to_string()).await {
            return Some(res);
        }

        println!("Invalid input specified, please try again");
    }

    // This should never happen but it makes the compiler happy
    None
}

pub async fn setup_settings() -> Settings {
    println!(
        "Welcome to {}!\n\
        \n\
        I was unable to find a config file\n\
        so I'll walk you through making a new one.\n\
        \n\
        If you have any issues during setup or\n\
        while using the bot, feel free to contact\n\
        {} on Discord for support!\n\
        \n\
        If you wish to stop the bot at any time,\n\
        press Control+C and the bot will force stop.
        \n\
        This will only take a minute!",
        PROG_NAME, AUTHOR_DISC
    );

    let token = get_valid_input("your Discord token", async move |tkn: String| {
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
                }
                200 => {
                    let user =
                        serde_json::from_str::<BasicUser>(&res.text().await.unwrap()).unwrap();
                    println!(
                        "\nNice to meet you {}#{}!",
                        user.username, user.discriminator
                    );
                    Some(tkn)
                }
                _ => {
                    println!("\nUnexpected response: {}", res.status().as_u16());
                    None
                }
            },
            Err(_) => {
                println!(
                    "\nUnable to check token with Discord,\n\
                    check your internet connection and try again."
                );
                None
            }
        }
    })
    .await
    .unwrap_or("<TOKEN HERE>".to_string());

    let prefix = get_valid_input(
        "preferred prefix (Default: ~)",
        async move |prefix: String| Some(prefix),
    )
    .await
    .unwrap_or("~".to_string());

    let global_nsfw_level = get_valid_input(
        "NSFW level for channels not marked as NSFW (Default: 1)\n\
    0 - Strict filtering\n\
    1 - Moderate filtering\n\
    2 - Disable filtering",
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
        "your gender, 'male' or 'female' (There are only two genders)\n\
        Used for referring to yourself, i.e. himself, herself, etc",
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

    load_settings().unwrap()
}

pub fn load_settings() -> Result<Settings, String> {
    let mut contents = String::new();
    let mut f = match File::open("config.toml") {
        Ok(file) => file,
        Err(why) => {
            match why.kind() {
                std::io::ErrorKind::NotFound => {
                    return Err("Unable to find 'config.toml', \
                        copy 'config.toml.bak' and setup config"
                        .to_string());
                }
                _ => {}
            }

            return Err(format!(
                "Unknown error occured while opening 'config.toml'\n[Config] {}",
                why
            ));
        }
    };

    if let Err(why) = f.read_to_string(&mut contents) {
        match why.kind() {
            std::io::ErrorKind::NotFound => {
                return Err("Unable to find 'config.toml', \
                    copy 'config.toml.bak' and setup config"
                    .to_string());
            }
            _ => {}
        }

        return Err(format!(
            "Unknown error occured while opening 'config.toml'.\n[Config] {}",
            why
        ));
    }

    let res = match toml::from_str(&contents) {
        Ok(res) => res,
        Err(why) => return Err(format!("Unable to deserialize settings.\n[Config] {}", why)),
    };

    println!("[Config] Load successful");

    Ok(res)
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let contents = try_or_msg!(
        toml::to_string(settings),
        "Unable to serialize config".to_string()
    );

    let mut f = match File::create("config.toml") {
        Ok(file) => file,
        Err(why) => return Err(format!("Unable to create 'config.toml'\n[Config] {}", why)),
    };

    try_or_msg!(
        f.write_all(&contents.as_bytes()),
        "Unable to write config to buffer".to_string()
    );
    try_or_msg!(
        f.sync_data(),
        "Unable to write config to 'config.toml'".to_string()
    );
    println!("[Config] Save successful");

    Ok(())
}
