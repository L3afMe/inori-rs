#![feature(async_closure)]
mod commands;
mod events;
mod macros;
mod models;
mod settings;
mod utils;

use std::{collections::HashMap, fs::File, io::Write, path::Path, sync::Arc};

pub use colored::Colorize;
use serenity::{
    framework::standard::{macros::hook, Command, CommandGroup, OnlyIn, StandardFramework},
    model::channel::Message,
    prelude::{Client, Context, Mutex},
};

use crate::{
    commands::*,
    events::{
        chat::{after, before, normal_message},
        error::dispatch_error,
        help::HELP,
        Handler,
    },
    models::{
        commands::{CommandCounter, ShardManagerContainer},
        discord::{InoriChannelUtils, InoriMessageUtils, MessageCreator},
        settings::Settings,
    },
    settings::{load_settings, save_settings, setup_settings},
    utils::{
        consts::{AUTHOR_DISC, GITHUB_LINK, PROG_NAME},
        version::check_is_latest,
    },
};

#[hook]
async fn dynamic_prefix(ctx: &Context, _msg: &Message) -> Option<String> {
    let data = ctx.data.read().await;
    let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    Some(settings.clone().command_prefix)
}

#[tokio::main]
async fn main() {
    utils::logging::setup_logger().expect("Unable to setup logger");

    check_is_latest().await;

    let settings = if Path::exists(Path::new(&"config.toml")) {
        match load_settings().await {
            Ok(settings) => settings,
            Err(why) => {
                inori_panic!("Config", "Error while loading config: {}", why);

                return;
            },
        }
    } else {
        println!(
            "Welcome to {}!\n\nI was unable to find a config file\nso I'll walk you through making a new one.\n\nIf \
             you have any issues during setup or\nwhile using the bot, feel free to contact\n{} on Discord for \
             support!\n\nIf you wish to stop the bot at any time,\npress Control+C and the bot will force stop.\n\nIf \
             you have a GitHub account don't forget\nto star the repo!\n{}\n\nThis will only take a minute!",
            PROG_NAME, AUTHOR_DISC, GITHUB_LINK
        );

        let settings = setup_settings(&toml::map::Map::new()).await;
        inori_info!(
            "Config",
            "Config setup and ready to use\n[Bot] Make sure to run {}setup which will create an new server and add \
             emotes that are used throughout the bot",
            &settings.command_prefix,
        );

        settings
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
        .group(&MISCELLANEOUS_GROUP)
        .group(&UTILITY_GROUP)
        .group(&PROGRAMMING_GROUP)
        .group(&MODERATION_GROUP);

    inori_info!("Bot", "Configured framework");

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

    inori_info!("Bot", "Loaded client");
    inori_info!("Bot", "Starting {} v{}", utils::consts::PROG_NAME, utils::consts::PROG_VERSION);

    if let Err(why) = client.start().await {
        inori_error!("Bot", "Client error: {:?}", why);
    }
}

#[allow(dead_code)]
fn titlize<D: ToString>(inp: D) -> String {
    let inp = inp.to_string();

    let first = inp[..1].to_string();
    let rest = inp[1..].to_string();

    format!("{}{}", first.to_uppercase(), rest.to_lowercase())
}

#[allow(dead_code)]
fn format_command(parent_command: &str, command: &Command) -> String {
    let mut output = String::new();

    let names = command.options.names;
    let mut cmd_name = format!("{} {}", parent_command, titlize(names.get(0).unwrap_or(&"Unknown")));
    cmd_name = cmd_name.trim().to_string();

    output = format!("{}\n\n### {}", output, cmd_name);
    if let Some(desc) = command.options.desc {
        output = format!("{}\n\n{}", output, desc);
    }

    let mut ending = String::new();
    if names.len() >= 2 {
        let mut names = names.to_vec();
        names.remove(0);
        ending = format!("- Aliases: `{}`", names.join("`, `"));
    }

    if let Some(usage) = command.options.usage {
        ending = format!("{}\n- Usage: `{} {}`", ending, cmd_name.to_lowercase(), usage);
    }

    if !command.options.examples.is_empty() {
        let examples = command.options.examples.to_vec();
        ending = format!(
            "{}\n- Examples:\n  - `{} {}`",
            ending,
            cmd_name.to_lowercase(),
            examples.join(&format!("`\n  - `{} ", cmd_name.to_lowercase()))
        );
    }

    if !command.options.sub_commands.is_empty() {
        let subcmds = command.options.sub_commands.to_vec();
        let mapped = subcmds
            .iter()
            .map(|cmd| {
                let name = cmd.options.names.get(0).unwrap_or(&"Unknown");
                let ending = format!("#{}{}", parent_command.replace(' ', " ").to_lowercase(), name.to_lowercase());

                format!("[{}]({})", name, ending)
            })
            .collect::<Vec<String>>();

        ending = format!("{}\n- Subcommands: {}", ending, mapped.join(", "));
    }

    let only_in = match command.options.only_in {
        OnlyIn::Dm => "DMs",
        OnlyIn::Guild => "Guilds",
        OnlyIn::None => "DMs and Guilds",
        _ => "Unknown",
    };

    ending = format!("{}\n- In: {}", ending, only_in);
    output = format!("{}\n\n{}", output, ending.trim());

    for sub_command in command.options.sub_commands {
        output = format!("{}{}", output, format_command(&cmd_name, sub_command));
    }

    output
}

#[allow(dead_code)]
fn commands_to_md(groups: &[&'static CommandGroup]) {
    let groups = groups.to_vec();
    let mut output = String::new();

    for group in groups {
        output = format!("{}\n\n\n## {}", output, titlize(group.name));

        for command in group.options.commands {
            output = format!("{}\n\n{}", output, format_command("", command).trim());
        }
    }

    output = format!(
        "{}\n\nThis file was autogenerated using commands_to_md in [main.rs](src/main.rs) using the commands help \
         menus\n",
        output.trim()
    );

    let mut file = File::create("COMMANDS.md").expect("Unable to create COMMANDS.md");
    file.write_all(output.as_bytes()).expect("Unable to write to COMMANDS.md");

    inori_success!("COMMANDS.md Generator", "Output saved to COMMANDS.md")
}
