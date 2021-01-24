use colorsys::{ColorAlpha, ColorTransform, Hsl, Rgb};
use rand::Rng;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, prelude::ReactionType},
    prelude::Context,
    utils::Colour,
};

use crate::{
    parse_arg,
    utils::{
        chat::{get_user, is_user},
        consts,
    },
    InoriChannelUtils, MessageCreator,
};

#[command]
#[description("Display information about Inori-rs")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_noret(ctx, |m: &mut MessageCreator| {
            m.title("About")
                .content(
                    "Originally starting as a personal project to learn Rust, \
                    Inori-rs is now a fully open source SelfBot available to the public. \
                    (Named after \
                    [Inori Yuzuriha](https://guiltycrown.fandom.com/wiki/Inori_Yuzuriha) \
                    from Guilty Crown)",
                )
                .field("Version", consts::PROG_VERSION, true)
                .field(
                    "Library",
                    "[Serenity v0.9.4](https://github.com/serenity-rs/serenity) with \
                    [SelfBot support](https://github.com/L3afMe/serenity-selfbot-support)",
                    true,
                )
                .field(
                    "Author",
                    format!("{} ({})", consts::AUTHOR_NAME, consts::AUTHOR_DISC),
                    true,
                )
                .field("GitHub Repo", consts::GITHUB_LINK, true)
                .image(
                    "https://static.wikia.nocookie.net/guiltycrown/mages\
                    /a/a5/Guilty_Crown_-_01_-_Large_17.jpg",
                )
        })
        .await
}

async fn print_av(ctx: &Context, msg: &Message, user: u64) -> CommandResult {
    let user = if let Ok(user) = ctx.http.get_user(user).await {
        user
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Avatar").content("Couldn't get user")
            })
            .await;
    };

    let av = match user.avatar_url() {
        Some(av) => av,
        None => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Avatar")
                        .content(format!("Unable to get {}'s avatar URL", user.name))
                })
                .await;
        },
    };

    return msg
        .channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.title("Avatar")
                .content(format!(
                    "{}#{}'s profile picture",
                    user.name,
                    format!("{:0>4}", user.discriminator)
                ))
                .image(av)
                .footer_text(format!("ID: {}", user.id.0))
        })
        .await;
}

#[command]
#[aliases("av", "pfp")]
#[description("Gets the pfp(s) of the mentioned user(s), if no one mentioned then gets self")]
#[usage("[@users]")]
#[example("@L3af#0001")]
async fn avatar(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        let _ = print_av(ctx, msg, msg.author.id.0).await;
    } else {
        for arg in args.iter::<String>() {
            let arg = arg.unwrap_or_default();

            if is_user(&arg) {
                let user = if let Ok(user) = get_user(&arg).parse::<u64>() {
                    user
                } else {
                    let _ = msg
                        .channel_id
                        .send_tmp(ctx, |m: &mut MessageCreator| {
                            m.error().title("Avatar").content("Could not parse user ID")
                        })
                        .await;
                    continue;
                };

                let _ = print_av(ctx, msg, user).await.unwrap_or(());
            }
        }
    }

    Ok(())
}

#[command]
#[aliases("ratelimit", "rl")]
#[description("List Discords ratelimits")]
async fn ratelimits(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_noret(ctx, |m: &mut MessageCreator| {
            m.title("Discord Ratelimits")
                .content("Ratelimits are in request/seconds")
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
                .field("Presence Update", "5/60\nPer Account", true)
        })
        .await
}

#[command]
#[description("Run a poll, options split with `|`. Max of 10 options")]
#[usage("<question> | <option> | <option> | <option>")]
#[example("Do you like chocolate? | Yes | No")]
async fn poll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = args.rest();
    let spl = args.split('|').collect::<Vec<&str>>();

    if args.contains('|') && spl.len() >= 3 && spl.len() <= 11 {
        let emojis = (0..(spl.len() - 1))
            .map(|i| std::char::from_u32('🇦' as u32 + i as u32).expect("Failed to format emoji"))
            .collect::<Vec<_>>();

        let poll_msg = msg
            .channel_id
            .send(ctx, |m: &mut MessageCreator| {
                m.title("Poll").content(format!("**{}**", spl[0]));

                for i in 1..spl.len() {
                    m.field(emojis[i - 1], spl[i], true);
                }

                m
            })
            .await
            .unwrap();

        for &emoji in &emojis {
            poll_msg.react(&ctx.http, ReactionType::Unicode(emoji.to_string())).await?;
        }

        Ok(())
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Poll").content("Invalid arguments")
            })
            .await;
    }
}

#[command]
#[description("Force set your Hype Squad house.\nAvailable houses:\nBravery\nBrilliance\nRandom")]
#[usage("<house>")]
#[example("Bravery")]
#[num_args(1)]
async fn hypesquad(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let houses = vec!["Bravery".to_string(), "Brilliance".to_string(), "Balance".to_string()];
    let houses_lower = houses.iter().map(|h| h.to_lowercase()).collect::<Vec<String>>();

    let house = args.current().unwrap_or("").to_lowercase();
    let house_id = if house.to_lowercase().eq("random") {
        rand::thread_rng().gen_range(1..4)
    } else if houses_lower.contains(&house) {
        houses_lower.iter().position(|r| r.eq(&house)).unwrap_or(0) + 1
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Hype Squad Changer").content("Invalid house specified")
            })
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
                msg.channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.success()
                            .title("Hype Squad Changer")
                            .content(format!("Set house to {}", house))
                    })
                    .await
            } else {
                msg.channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.error()
                            .title("Hype Squad Changer")
                            .content(format!("Invalid response status: {}", status))
                    })
                    .await
            }
        },
        Err(_) => {
            msg.channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Hype Squad Changer")
                        .content("Error occurred while changing house")
                })
                .await
        },
    };
}

macro_rules! parse_color {
    ($color_name:literal, $ctx:expr, $msg:expr, $arg:expr) => {
        if let Ok(color) = $arg.parse::<f64>() {
            color
        } else {
            return $msg
                .channel_id
                .send_tmp($ctx, |m: &mut MessageCreator| {
                    m.error().title("Color").content(format!("Unable to parse {}", $color_name))
                })
                .await;
        }
    };
}

#[command]
#[aliases("colour")]
#[description("View a color")]
#[usage("<color>")]
#[example("#FAB1ED")]
#[example("150 177 237")]
#[example("150 177 237 0.5")]
#[min_args(1)]
#[max_args(4)]
async fn color(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let split = args.rest().split(',').map(|f| f.trim().to_string()).collect::<Vec<String>>();
    let rgb = if args.len() == 4 || split.len() == 4 {
        let args = if split.len() == 4 {
            split
        } else {
            args.rest().split(' ').map(|f| f.to_string()).collect::<Vec<String>>()
        };

        let r = parse_color!("red", ctx, msg, args[0]);
        let g = parse_color!("green", ctx, msg, args[1]);
        let b = parse_color!("blue", ctx, msg, args[2]);
        let a = parse_color!("alpha", ctx, msg, args[3]);

        Rgb::from((r, g, b, a))
    } else if args.len() == 3 || split.len() == 3 {
        let args = if split.len() == 3 {
            split
        } else {
            args.rest().split(' ').map(|f| f.to_string()).collect::<Vec<String>>()
        };

        let r = parse_color!("red", ctx, msg, args[0]);
        let g = parse_color!("green", ctx, msg, args[1]);
        let b = parse_color!("blue", ctx, msg, args[2]);

        Rgb::from((r, g, b))
    } else if args.len() == 1 {
        if let Ok(color) = Rgb::from_hex_str(args.rest()) {
            color
        } else {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Error").content("Unable to parse hexadecimal value")
                })
                .await;
        }
    } else {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("Error")
                    .content("Invalid args given!\nExpected: 1, 3 or 4, Given: 2")
            })
            .await;
    };

    let mut rgb_invert = rgb.clone();
    rgb_invert.invert();

    let hsl = Hsl::from(rgb.clone());
    let mut hsl_invert = hsl.clone();
    hsl_invert.invert();

    let serenity_color = Colour::from_rgb(
        rgb.get_red().floor() as u8,
        rgb.get_green().round() as u8,
        rgb.get_blue().floor() as u8,
    );

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.colour(serenity_color)
                .title("Color")
                .field(
                    "RGBA",
                    format!(
                        "{:.2}, {:.2}, {:.2}, {:.2}",
                        rgb.get_red(),
                        rgb.get_green(),
                        rgb.get_blue(),
                        rgb.get_alpha()
                    ),
                    true,
                )
                .field(
                    "RGBA Inverted",
                    format!(
                        "{:.2}, {:.2}, {:.2}, {:.2}",
                        rgb_invert.get_red(),
                        rgb_invert.get_green(),
                        rgb_invert.get_blue(),
                        rgb_invert.get_alpha()
                    ),
                    true,
                )
                .field("Hexadecimal", rgb.to_hex_string(), true)
                .field("Hexadecimal Inverted", rgb_invert.to_hex_string(), true)
                .field(
                    "HSLA",
                    format!(
                        "{:.2}, {:.2}, {:.2}, {:.2}",
                        hsl.get_hue(),
                        hsl.get_saturation(),
                        hsl.get_lightness(),
                        hsl.get_alpha()
                    ),
                    true,
                )
                .field(
                    "HSLA Inverted",
                    format!(
                        "{:.2}, {:.2}, {:.2}, {:.2}",
                        hsl_invert.get_hue(),
                        hsl_invert.get_saturation(),
                        hsl_invert.get_lightness(),
                        hsl_invert.get_alpha()
                    ),
                    true,
                )
        })
        .await
}

#[command]
#[aliases("spam")]
#[description(
    "Send a message a set amount of times. Delay is in milliseconds.\nWARNING: THIS CANNOT BE STOPPED WITHOUT FORCE \
     STOPPING THE BOT"
)]
#[usage("<amount> <delay> <message>")]
#[example("50 500 Inori is best waifu")]
#[min_args(3)]
async fn spammer(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = parse_arg!(ctx, msg, args, "amount", u64);
    let delay = parse_arg!(ctx, msg, args, "delay", u64);
    let message = args.rest();

    let mut count = 0;
    for _ in 0..amount {
        if msg.channel_id.say(&ctx.http, message).await.is_ok() {
            count += 1;
        }
        tokio::time::delay_for(tokio::time::Duration::from_millis(delay)).await;
    }

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.success()
                .title("Spammer")
                .content(format!("Successfully send {} messages", count))
        })
        .await
}

#[command]
#[description("A test command for development, if this is in a release build please let me know.")]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    let content;

    let current_user = ctx.cache.current_user().await;

    content = format!("{:?}", current_user);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| m.title("Test").content(content))
        .await
}
