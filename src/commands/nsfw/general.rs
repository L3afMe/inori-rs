extern crate serde;
extern crate serde_xml_rs;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{
    models::commands::{Img, NekoBotResponse, NekosLifeResponse, Rule34Post, Rule34Posts},
    utils::{
        chat::{default_embed, say_error, send, send_loading},
        checks::{can_nsfw_moderate, can_nsfw_strict, NSFW_STRICT_CHECK},
    },
};

use once_cell::sync::Lazy;

use rand::Rng;

async fn get_rule_34_posts(tags: String) -> Rule34Posts {
    println!(
        "{}",
        format!(
            "https://rule34.xxx/index.php?page=dapi&s=post&q=index&tags={}",
            tags
        )
    );
    let xml = reqwest::get(&format!(
        "https://rule34.xxx/index.php?page=dapi&s=post&q=index&tags={}",
        tags
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    serde_xml_rs::from_str::<Rule34Posts>(&xml).unwrap()
}

#[command]
#[aliases("r34")]
#[description(
    "Gets an image from rule34.xxx with the specified tags. Tags are separated by spaces"
)]
#[usage("<tags>")]
#[example("catgirl thighs")]
#[checks(NSFW_Strict)]
#[min_args(1)]
async fn rule34(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let tags = args.rest().split(" ").collect::<Vec<&str>>();
    let res = get_rule_34_posts(tags.join("+")).await;
    let posts: Vec<Rule34Post> = if let Some(posts) = res.posts {
        posts
    } else {
        return say_error(
            ctx,
            &msg.channel_id,
            "Rule 34",
            "Well you fucking did it, you found something that doesn't exist in porn",
        )
        .await;
    };

    let post: &Rule34Post = posts
        .get(rand::thread_rng().gen_range(0..posts.len()))
        .unwrap();

    let embed = default_embed("Rules 34");
    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;

                e.description(&format!("Tags: {}", tags.join(", ")))
                    .image((&post.file_url).to_string())
            })
        })
        .await?;

    Ok(())
}

static VALID_IMAGES: Lazy<Vec<Img>> = Lazy::new(|| {
    vec![
        Img::new(1, "4k", 2),
        Img::new(0, "8ball", 0),
        Img::new(0, "anal", 2),
        Img::new(1, "ass", 1),
        Img::new(0, "avatar", 0),
        Img::new(0, "baka", 0),
        Img::new(0, "bj", 2),
        Img::new(0, "blowjob", 2),
        Img::new(0, "boobs", 2),
        Img::new(0, "classic", 2),
        Img::new(1, "coffee", 0),
        Img::new(0, "cuddle", 0),
        Img::new(0, "cum", 2),
        Img::new(0, "cum_jpg", 2),
        Img::new(0, "ero", 1),
        Img::new(0, "erofeet", 1),
        Img::new(0, "erok", 1),
        Img::new(0, "erokemo", 1),
        Img::new(0, "eron", 1),
        Img::new(0, "eroyuri", 1),
        Img::new(0, "feed", 0),
        Img::new(0, "feet", 1),
        Img::new(0, "feetg", 2),
        Img::new(0, "femdom", 1),
        Img::new(1, "food", 0),
        Img::new(0, "fox_girl", 0),
        Img::new(0, "futanari", 2),
        Img::new(1, "gah", 0),
        Img::new(0, "gasm", 1),
        Img::new(0, "gecg", 0),
        Img::new(1, "gonewild", 1),
        Img::new(0, "goose", 0),
        Img::new(1, "hanal", 2),
        Img::new(1, "hass", 2),
        Img::new(1, "hboobs", 2),
        Img::new(0, "hentai", 2),
        Img::new(1, "hkitsune", 2),
        Img::new(1, "hmidriff", 2),
        Img::new(1, "hneko", 2),
        Img::new(0, "holo", 0),
        Img::new(0, "holoero", 1),
        Img::new(0, "hololewd", 2),
        Img::new(1, "hthigh", 1),
        Img::new(0, "hug", 0),
        Img::new(1, "kanna", 0),
        Img::new(0, "kemonomimi", 1),
        Img::new(0, "keta", 1),
        Img::new(0, "kiss", 0),
        Img::new(0, "kuni", 2),
        Img::new(0, "les", 2),
        Img::new(0, "lewd", 2),
        Img::new(0, "lewdk", 2),
        Img::new(0, "lewdkemo", 2),
        Img::new(0, "lizard", 0),
        Img::new(0, "neko", 0),
        Img::new(0, "ngif", 0),
        Img::new(0, "nsfw_avatar", 2),
        Img::new(0, "nsfw_neko_gif", 2),
        Img::new(1, "paizuri", 2),
        Img::new(0, "pat", 0),
        Img::new(1, "pgif", 2),
        Img::new(0, "poke", 0),
        Img::new(0, "pussy", 2),
        Img::new(0, "pussy_jpg", 2),
        Img::new(0, "pwankg", 2),
        Img::new(0, "random_hentai_gif", 2),
        Img::new(0, "slap", 0),
        Img::new(0, "smallboobs", 2),
        Img::new(0, "smug", 0),
        Img::new(0, "solo", 2),
        Img::new(0, "solog", 2),
        Img::new(0, "spank", 1),
        Img::new(1, "tentacle", 2),
        Img::new(1, "thigh", 1),
        Img::new(0, "tickle", 0),
        Img::new(0, "tits", 2),
        Img::new(0, "trap", 2),
        Img::new(0, "waifu", 0),
        Img::new(0, "wallpaper", 0),
        Img::new(0, "woof", 0),
        Img::new(0, "yuri", 2),
    ]
});

async fn do_image(
    ctx: &Context,
    msg: &Message,
    img: &Img,
    amount: u64,
    is_image_bomb: bool,
) -> CommandResult {
    let title = if is_image_bomb { "Image Bomb" } else { "Image" };
    let image_str = if amount > 1 { "images" } else { "image" };
    let mut new_msg = send_loading(
        ctx,
        &msg.channel_id,
        title,
        &format!("Loading {} {}", amount, image_str),
    )
    .await;

    let mut urls = Vec::new();
    for _ in 0..amount {
        let url = match img.website_type {
            0 => {
                let target = if img.link.eq("random_hentai_gif") {
                    "/img/Random_hentai_gif".to_string()
                } else {
                    format!("/img/{}", img.link)
                };

                let res = reqwest::get(&format!("https://nekos.life/api/v2{}", target))
                    .await?
                    .text()
                    .await?;

                let res: NekosLifeResponse =
                    serde_json::from_str(&res).expect("Couldn't parse response.");

                res.url
            }
            _ => {
                let res = reqwest::get(&format!("https://nekobot.xyz/api/image?type={}", img.link))
                    .await?
                    .text()
                    .await?;

                let res: NekoBotResponse =
                    serde_json::from_str(&res).expect("Couldn't parse response.");

                res.message
            }
        };

        urls.push(url);
    }

    let embed = default_embed(title);
    new_msg
        .edit(&ctx, |m| {
            m.embed(|e| {
                e.0 = embed.0;

                e.description(format!("Tag: {}", img.link)).image(&urls[0])
            })
        })
        .await
        .unwrap_or(());

    if urls.len() > 1 {
        for url in urls[1..urls.len()].iter() {
            let embed = default_embed(title);
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.0 = embed.0;

                        e.description(format!("Tag: {}", img.link)).image(url)
                    })
                })
                .await
                .unwrap();
        }
    }

    Ok(())
}

#[command]
#[aliases("imgb", "imageb")]
#[description("Get's an image a specified amount of times")]
#[usage("<type> <amount>")]
#[example("erok 10")]
#[max_args(2)]
async fn imagebomb(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let valid_ids = VALID_IMAGES
        .iter()
        .map(|s| String::from(&s.link))
        .collect::<Vec<String>>();

    if !args.is_empty() {
        let arg = args.single::<String>().unwrap().to_lowercase();

        let amount = match args.single::<u64>() {
            Ok(amount) => {
                if amount < 1 {
                    return say_error(
                        ctx,
                        &msg.channel_id,
                        "Image Bomb",
                        "Amount cannot be less than 1",
                    )
                    .await;
                } else {
                    amount
                }
            }
            Err(_) => {
                return say_error(
                    ctx,
                    &msg.channel_id,
                    "Image Bomb",
                    "Unable to parse given value to number",
                )
                .await;
            }
        };

        if valid_ids.contains(&arg) {
            let selected: &Img = VALID_IMAGES
                .iter()
                .filter(|&s| s.link.eq(&arg))
                .next()
                .unwrap();

            let is_too_nsfw = match selected.level {
                0 => false,
                1 => match can_nsfw_moderate(ctx, msg).await {
                    serenity::framework::standard::CheckResult::Success => false,
                    _ => true,
                },
                2 => match can_nsfw_strict(ctx, msg).await {
                    serenity::framework::standard::CheckResult::Success => false,
                    _ => true,
                },
                _ => false,
            };

            if is_too_nsfw {
                return say_error(
                    ctx,
                    &msg.channel_id,
                    "Image Bomb",
                    &format!(
                        "This server is marked not marked as NSFW and \
                        you've specified a NSFW image.\nThis can be overriden \
                        by executing `nsfwfilter {}`",
                        selected.level
                    ),
                )
                .await;
            }

            return do_image(ctx, msg, &selected, amount, true).await;
        } else {
            return say_error(
                ctx,
                &msg.channel_id,
                "Image",
                &format!("Unable to find image tagged '{}'", arg),
            )
            .await;
        }
    }

    send(
        ctx,
        &msg.channel_id,
        "Image",
        &format!(
            "**Valid images**\n`{}`",
            VALID_IMAGES
                .iter()
                .map(|s| String::from(&s.link))
                .collect::<Vec<String>>()
                .join("`, `")
        ),
    )
    .await;

    Ok(())
}

#[command]
#[aliases("img")]
#[description("Get an image")]
#[usage("<type>")]
#[example("erok")]
#[max_args(1)]
async fn image(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let valid_ids = VALID_IMAGES
        .iter()
        .map(|s| String::from(&s.link))
        .collect::<Vec<String>>();

    if !args.is_empty() {
        let arg = args.current().unwrap().to_lowercase();

        if valid_ids.contains(&arg) {
            let selected: &Img = VALID_IMAGES
                .iter()
                .filter(|&s| s.link.eq(&arg))
                .next()
                .unwrap();

            let is_too_nsfw = match selected.level {
                0 => false,
                1 => match can_nsfw_moderate(ctx, msg).await {
                    serenity::framework::standard::CheckResult::Success => false,
                    _ => true,
                },
                2 => match can_nsfw_strict(ctx, msg).await {
                    serenity::framework::standard::CheckResult::Success => false,
                    _ => true,
                },
                _ => false,
            };

            if is_too_nsfw {
                return say_error(
                    ctx,
                    &msg.channel_id,
                    "Image",
                    &format!(
                        "This server is marked not marked as NSFW and \
                        you've specified a NSFW image.\nThis can be overriden \
                        by executing `nsfwfilter {}`",
                        selected.level
                    ),
                )
                .await;
            }

            return do_image(ctx, msg, selected, 1, false).await;
        } else {
            return say_error(
                ctx,
                &msg.channel_id,
                "Image",
                &format!("Unable to find image tagged '{}'", arg),
            )
            .await;
        }
    }

    send(
        ctx,
        &msg.channel_id,
        "Image",
        &format!(
            "**Valid images**\n`{}`",
            VALID_IMAGES
                .iter()
                .map(|s| String::from(&s.link))
                .collect::<Vec<String>>()
                .join("`, `")
        ),
    )
    .await;

    Ok(())
}
