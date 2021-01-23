use std::{
    fs::{create_dir, remove_file},
    path::Path,
};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    http::AttachmentType,
    model::prelude::*,
    prelude::*,
    utils::read_image,
};
use tokio::{fs::File, prelude::*};

use crate::{save_settings, InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings};

#[command]
#[aliases("ps")]
#[description("Automatically switch profile pictures at a specified interval")]
#[min_args(1)]
#[sub_commands(enable, disable, toggle, change, delay, mode, list, view, add, delete)]
async fn pfpswitcher(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("Profile Picture Switcher")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

#[command]
#[aliases("t")]
#[description("Toggles profile picture switching")]
async fn toggle(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.pfp_switcher.enabled = !settings.pfp_switcher.enabled;
    save_settings(&settings);

    let content = if settings.pfp_switcher.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[description("Enables profile picture switching")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if settings.pfp_switcher.enabled {
        "Already enabled"
    } else {
        settings.pfp_switcher.enabled = true;
        save_settings(&settings);

        "Enabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[description("Disables profile picture switching")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if settings.pfp_switcher.enabled {
        settings.pfp_switcher.enabled = false;
        save_settings(&settings);

        "Disabled"
    } else {
        "Already disabled"
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[aliases("m")]
#[description("Set the mode of which pictures get chosen\n**Modes**\n0 - Random order\n1 - Alphabetical order")]
#[usage("<mode>")]
#[example("1")]
#[num_args(1)]
async fn mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if args.is_empty() {
        let sort = match settings.pfp_switcher.mode {
            0 => "Random",
            _ => "Alphabetical",
        };

        format!("Sort mode currently set to {}", sort)
    } else if let Ok(val) = args.single::<u8>() {
        if val <= 1 {
            settings.pfp_switcher.mode = val;
            save_settings(&settings);

            let sort = match val {
                0 => "Random",
                _ => "Alphabetical",
            };

            format!("Sort mode set to `{}`", sort)
        } else {
            drop(settings);
            drop(data);

            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Profile Picture Switcher")
                        .content("Invalid mode specified.\n**Valid modes**\n`0` - Random\n`1` - Alphabetical")
                })
                .await;
        }
    } else {
        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Profile Picture Switcher").content("Unable to parse mode")
            })
            .await;
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[description(
    "Get or set the delay between switching profile pictures in minutes. Minimum set to 30 minutes to avoid rate \
     limiting (Default 45 minutes)"
)]
#[usage("<minutes>")]
#[example("10")]
async fn delay(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg = if args.is_empty() {
        "current"
    } else {
        args.current().unwrap()
    };

    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    let content = if arg.to_lowercase().eq("current") {
        format!("Delay currently set to {} minutes", settings.pfp_switcher.delay)
    } else if let Ok(delay) = arg.parse::<u32>() {
        if delay < 10 {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Profile Picture Switcher")
                        .content("Minimum delay is 45 minutes to avoid rate limiting")
                })
                .await;
        } else {
            settings.pfp_switcher.delay = delay;
            save_settings(&settings);

            format!("Delay set to {} minutes", delay)
        }
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Unable to parse delay to number")
            })
            .await;
    };

    drop(settings);
    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[aliases("add", "a")]
#[description("Add a new profile pic with a specific name. File name cannot contain spaces")]
#[usage("<file name>")]
#[example("rias.png")]
#[num_args(1)]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.attachments.len() != 1 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("**1 image** attached is required to add a new profile picture")
            })
            .await;
    }

    let mut file_name = args.current().unwrap().to_string();
    let atch = msg.attachments.get(0).unwrap();
    let atch_url = &atch.url;
    let url = atch_url.split('?').next().unwrap().to_string();
    let ext = &url.to_string()[url.rfind('.').unwrap()..];

    if !file_name.ends_with(ext) {
        file_name = format!("{}{}", file_name, ext);
    }

    if !Path::new("./pfps/").exists() {
        create_dir("pfps").unwrap();
    }

    if Path::new(&format!("./pfps/{}", file_name)).is_file() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content(format!("File named '{}' already exists", file_name))
            })
            .await;
    }

    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Profile Picture Switcher", "Uploading new profile picture")
        .await
        .unwrap();

    let img = reqwest::get(atch_url).await?.bytes().await?;
    let mut file = File::create(format!("./pfps/{}", file_name)).await.unwrap();
    file.write_all(&img).await?;

    new_msg
        .update_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher")
                .image(atch_url)
                .content(format!("Successfully added '{}'", file_name))
        })
        .await
}

#[command]
#[aliases("c")]
#[description("Set your pfp to one from the Pfp Switcher selection")]
#[usage("<file name>")]
#[example("rias.png")]
#[num_args(1)]
async fn change(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img_str = args.current().unwrap().to_string();

    if img_str.contains("../") || img_str.contains("//") {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("File contains disallowed characters")
            })
            .await;
    }

    let path_str = format!("./pfps/{}", img_str);
    let path = Path::new(&path_str);

    if !path.exists() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Specified file does not exist")
            })
            .await;
    }

    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Profile Picture Switcher", "Uploading image")
        .await
        .unwrap();

    let mut user = ctx.cache.current_user().await;
    let avatar = read_image((&path_str).to_string())?;
    user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();

    println!("[PfpSwitcher] Changing pfps");

    let pfp_url = ctx.http.get_current_user().await.unwrap().avatar_url().unwrap();

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher")
                .content(format!("Switched profile picture to `{}`", img_str))
                .image(pfp_url)
        })
        .await
}

#[command]
#[aliases("remove", "delete", "rem", "del", "rm")]
#[description("Remove a profile picture with a specific name")]
#[usage("<file name>")]
#[example("rias")]
#[num_args(1)]
async fn delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img_str = args.current().unwrap().to_string();

    if img_str.contains("../") || img_str.contains("//") {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("File contains disallowed characters")
            })
            .await;
    }

    let path_str = format!("./pfps/{}", img_str);
    let path = Path::new(&path_str);

    if !path.exists() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Specified file does not exist")
            })
            .await;
    }

    remove_file(path)?;

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher")
                .content(format!("Successfully removed '{}'", img_str))
        })
        .await
}

#[command]
#[aliases("view", "v")]
#[description("Sends profile picture with specified name")]
#[usage("<file name>")]
#[example("rias")]
#[num_args(1)]
async fn view(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let img_str = args.current().unwrap().to_string();

    if img_str.contains("../") || img_str.contains("//") {
        return msg
            .channel_id
            .send_tmp(ctx, |m| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("File contains disallowed characters")
            })
            .await;
    }

    let path_str = format!("./pfps/{}", img_str);

    if !Path::new(&path_str).exists() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Specified file does not exist")
            })
            .await;
    }

    let img = File::open(path_str).await.unwrap();

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Profile Picture Switcher")
                .content(img_str.clone())
                .attachment(AttachmentType::File {
                    file:     &img,
                    filename: img_str.clone(),
                })
        })
        .await
}

#[command]
#[aliases("list", "l")]
#[description("List all picture names")]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut list = "".to_string();

    let dir_list: Vec<Result<std::fs::DirEntry, std::io::Error>> =
        Path::new("./pfps/").read_dir().expect("Unable to execute read_dir").collect();

    if dir_list.is_empty() {
        list = "Nothing to see here".to_string();
    }

    for file in dir_list {
        if let Ok(img) = file {
            list = format!("{}\n{}", list, img.file_name().into_string().unwrap());
        }
    }

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("Profile Picture Switcher").content(list))
        .await
}
