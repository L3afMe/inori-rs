use std::{
    fs::{create_dir, remove_file},
    path::Path,
};

use colored::Colorize;
use once_cell::sync::Lazy;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    http::AttachmentType,
    model::prelude::*,
    prelude::*,
    utils::read_image,
};
use tokio::{fs::File, prelude::*};

use crate::{inori_info, parse_arg, save_settings, InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings};

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
            m.success().title("Profile Picture Switcher").content(content)
        })
        .await
}

#[command]
#[description("Enables profile picture switching")]
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.pfp_switcher.enabled {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("Profile Picture Switcher").content("Already enabled")
            })
            .await
    } else {
        settings.pfp_switcher.enabled = true;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("Profile Picture Switcher").content("Enabled")
            })
            .await
    }
}

#[command]
#[description("Disables profile picture switching")]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    if settings.pfp_switcher.enabled {
        settings.pfp_switcher.enabled = false;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success().title("Profile Picture Switcher").content("Disabled")
            })
            .await
    } else {
        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info().title("Profile Picture Switcher").content("Already disabled")
            })
            .await
    }
}

#[command]
#[aliases("m")]
#[description("Set the mode in which pictures get chosen\n**Modes**\n0 - Random order\n1 - Alphabetical order")]
#[usage("<mode>")]
#[example("1")]
#[num_args(1)]
async fn mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.write().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        let sort = match settings.pfp_switcher.mode {
            0 => "Random",
            _ => "Alphabetical",
        };

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Profile Picture Switcher")
                    .content(format!("Sort mode currently set to {}", sort))
            })
            .await;
    }

    let val = parse_arg!(ctx, msg, args, "mode", u8);
    if val <= 1 {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
        settings.pfp_switcher.mode = val;
        save_settings(&settings);

        drop(settings);
        drop(data);

        let sort = match val {
            0 => "Random",
            _ => "Alphabetical",
        };

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success()
                    .title("Profile Picture Switcher")
                    .content(format!("Sort mode set to `{}`", sort))
            })
            .await
    } else {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Invalid mode specified.\n**Valid modes**\n`0` - Random\n`1` - Alphabetical")
            })
            .await
    }
}

#[command]
#[description(
    "Get or set the delay between switching profile pictures in minutes. Minimum set to 30 minutes to avoid rate \
     limiting (Default 45 minutes)"
)]
#[usage("<minutes>")]
#[example("45")]
async fn delay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let data = ctx.data.write().await;
        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
        let delay = settings.pfp_switcher.delay;

        drop(settings);
        drop(data);

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.info()
                    .title("Profile Picture Switcher")
                    .content(format!("Delay currently set to {} minutes", delay))
            })
            .await;
    }

    let delay = parse_arg!(ctx, msg, args, "delay", u32);

    if delay < 10 {
        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Minimum delay is 10 minutes to avoid rate limiting")
            })
            .await
    } else {
        let data = ctx.data.write().await;
        let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;
        settings.pfp_switcher.delay = delay;
        save_settings(&settings);

        drop(settings);
        drop(data);

        msg.channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.success()
                    .title("Profile Picture Switcher")
                    .content(format!("Delay set to {} minutes", delay))
            })
            .await
    }
}

static JPEG_BYTES: Lazy<Vec<u8>> = Lazy::new(|| vec![255, 216, 255, 224]);
static PNG_BYTES: Lazy<Vec<u8>> = Lazy::new(|| vec![137, 80, 78, 71]);
static WEBP_BYTES: Lazy<Vec<u8>> = Lazy::new(|| vec![82, 73, 70, 70]);

#[command]
#[aliases("add", "a")]
#[description(
    "Add a new profile pic with a specific name. File name cannot contain spaces. Currently only supports JPEG, PNG \
     and WEBP"
)]
#[usage("<file name> [url]")]
#[example("rias")]
#[example("inori https://i.imgur.com/43huWfw.png")]
#[min_args(1)]
#[max_args(2)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 1 && msg.attachments.len() != 1 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("1 image attached or a link to an image is required to add a new profile picture")
            })
            .await;
    }

    let mut file_name = args.single::<String>().unwrap();
    let url = if args.len() == 1 {
        let atch = msg.attachments.get(0).unwrap();
        let atch_url = &atch.url;
        atch_url.split('?').next().unwrap().to_string()
    } else {
        args.single::<String>().unwrap()
    };

    if !Path::new("./pfps/").exists() && create_dir("pfps").is_err() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Unable to create `pfps/` directory")
            })
            .await;
    }

    if Path::new(&format!("./pfps/{}", file_name)).is_file() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
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

    let response = if let Ok(response) = reqwest::get(&url).await {
        response
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Profile Picture Switcher").content("Unable to download image")
            })
            .await;
    };

    let img = if let Ok(bytes) = response.bytes().await {
        bytes
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Unable to convert image to bytes")
            })
            .await;
    };

    let bytes = img.clone()[0..4].to_vec();
    println!("Bytes: {:?}", bytes);

    let ext = if JPEG_BYTES.eq(&bytes) {
        "jpg"
    } else if PNG_BYTES.eq(&bytes) {
        "png"
    } else if WEBP_BYTES.eq(&bytes) {
        "webp"
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Profile Picture Switcher")
                    .content("Link provided is not a valid image")
            })
            .await;
    };

    file_name = format!("{}.{}", file_name, ext);
    let mut file = File::create(format!("./pfps/{}", file_name)).await.unwrap();
    file.write_all(&img).await?;

    new_msg
        .update_tmp(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Profile Picture Switcher")
                .image(url)
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
async fn change(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let img_str = args.single::<String>().unwrap();

    if img_str.contains("../") || img_str.contains("//") {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
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

    let mut user = ctx.http.get_current_user().await.unwrap();
    let avatar = read_image((&path_str).to_string())?;
    user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();

    inori_info!("PfpSwitcher", "Changing pfps");

    let pfp_url = ctx.http.get_current_user().await.unwrap().avatar_url().unwrap();

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Profile Picture Switcher")
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
                m.warning()
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
            m.success()
                .title("Profile Picture Switcher")
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
                m.warning()
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

    // Temp fix while I figure out why
    // MessageCreator.attachment doesn't work
    let _ = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.add_file(AttachmentType::File {
                file:     &img,
                filename: img_str.clone(),
            });

            m.embed(|e| {
                e.title("[Profile Picture Switcher]")
                    .color(serenity::utils::Color::FABLED_PINK)
                    .description(format!("{} (Temporary fix)", img_str))
                    .attachment(img_str)
            })
        })
        .await;

    Ok(())

    // msg.channel_id
    // .send_tmp(ctx, |m: &mut MessageCreator| {
    // m.title("Profile Picture Switcher")
    // .content(img_str.clone())
    // .attachment(&img_str, AttachmentType::File
    // { file:     &img,
    // filename: img_str.clone(),
    // })
    // })
    // .await
}

#[command]
#[aliases("list", "l")]
#[description("List all picture names")]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut list = "".to_string();

    let dir_list: Vec<Result<std::fs::DirEntry, std::io::Error>> = if let Ok(path) = Path::new("./pfps/").read_dir() {
        path.collect()
    } else {
        Vec::new()
    };

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
