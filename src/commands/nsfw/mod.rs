use serenity::framework::standard::macros::group;

#[group]
#[commands(image, imagebomb, rule34)]
#[description("**NSFW**")]
struct NSFW;

use once_cell::sync::Lazy;
use rand::Rng;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{
    models::commands::{Img, NekoBotResponse, NekosLifeResponse, Rule34Post, Rule34Posts},
    utils::checks::{can_nsfw_moderate, can_nsfw_strict, NSFW_STRICT_CHECK},
    InoriChannelUtils, InoriMessageUtils, MessageCreator,
};

async fn get_rule_34_posts(tags: String) -> Rule34Posts {
    let xml = reqwest::get(&format!("https://rule34.xxx/index.php?page=dapi&s=post&q=index&tags={}", tags))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    serde_xml_rs::from_str::<Rule34Posts>(&xml).unwrap()
}

#[command]
#[aliases("r34")]
#[description("Gets an image from rule34.xxx with the specified tags. Tags are separated by spaces")]
#[usage("<tags>")]
#[example("catgirl thighs")]
#[checks(NSFW_Strict)]
#[min_args(1)]
async fn rule34(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, "Rule 34", "Loading some juicy images")
        .await
        .unwrap();

    let tags = args.rest().split(' ').collect::<Vec<&str>>();
    let res = get_rule_34_posts(tags.join("+")).await;

    let posts: Vec<Rule34Post> = if let Some(posts) = res.posts {
        posts
    } else {
        return new_msg
            .update_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
                    .title("Rule 34")
                    .content("Well you fucking did it, you found something that doesn't exist in porn")
            })
            .await;
    };

    let post: &Rule34Post = posts.get(rand::thread_rng().gen_range(0..posts.len())).unwrap();

    new_msg
        .update_noret(ctx, |m: &mut MessageCreator| {
            m.title("Rule 34")
                .content(format!("Tags: {}", tags.join(", ")))
                .image(post.file_url.clone())
        })
        .await
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

async fn do_image(ctx: &Context, msg: &Message, img: &Img, amount: u64, is_image_bomb: bool) -> CommandResult {
    let title = if is_image_bomb { "Image Bomb" } else { "Image" };
    let image_str = if amount > 1 { "images" } else { "image" };

    let mut new_msg = msg
        .channel_id
        .send_loading(ctx, title, &format!("Loading {} {}", amount, image_str))
        .await
        .unwrap();

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

                let res: NekosLifeResponse = serde_json::from_str(&res).expect("Couldn't parse response.");

                res.url
            },
            _ => {
                let res = reqwest::get(&format!("https://nekobot.xyz/api/image?type={}", img.link))
                    .await?
                    .text()
                    .await?;

                let res: NekoBotResponse = serde_json::from_str(&res).expect("Couldn't parse response.");

                res.message
            },
        };

        urls.push(url);
    }

    let _ = new_msg
        .update(ctx, |m: &mut MessageCreator| {
            m.title(title).content(format!("Tag: {}", img.link)).image(&urls[0])
        })
        .await;

    if urls.len() > 1 {
        for url in urls[1..urls.len()].iter() {
            let _ = msg
                .channel_id
                .send(ctx, |m: &mut MessageCreator| {
                    m.title(title).content(format!("Tag: {}", img.link)).image(url)
                })
                .await;
        }
    }

    Ok(())
}

#[command]
#[aliases("imgb", "imageb")]
#[description("Gets an image a specified amount of times")]
#[usage("<type> <amount>")]
#[example("erok 10")]
#[max_args(2)]
async fn imagebomb(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let valid_ids = VALID_IMAGES.iter().map(|s| String::from(&s.link)).collect::<Vec<String>>();

    if !args.is_empty() {
        let arg = args.single::<String>().unwrap().to_lowercase();

        let amount = match args.single::<u64>() {
            Ok(amount) => {
                if amount < 1 {
                    return msg
                        .channel_id
                        .send_tmp(ctx, |m: &mut MessageCreator| {
                            m.error().title("Image Bomb").content("Amount cannot be less than 1")
                        })
                        .await;
                } else {
                    amount
                }
            },
            Err(_) => {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.error().title("Image Bomb").content("Unable to parse given value to number")
                    })
                    .await;
            },
        };

        if valid_ids.contains(&arg) {
            let selected: &Img = VALID_IMAGES.iter().find(|&s| s.link.eq(&arg)).unwrap();

            let is_too_nsfw = match selected.level {
                0 => false,
                1 => !matches!(
                    can_nsfw_moderate(ctx, msg).await,
                    serenity::framework::standard::CheckResult::Success
                ),
                _ => !matches!(
                    can_nsfw_strict(ctx, msg).await,
                    serenity::framework::standard::CheckResult::Success
                ),
            };

            if is_too_nsfw {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.warning().title("Error").content(format!(
                            "This server is marked not marked as NSFW and you've specified a NSFW image.\nThis can be \
                             overriden by executing `nsfwfilter {}`",
                            selected.level
                        ))
                    })
                    .await;
            }

            return do_image(ctx, msg, &selected, amount, true).await;
        } else {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Image Bomb")
                        .content(format!("Unable to find image tagged '{}'", arg))
                })
                .await;
        }
    }

    msg.channel_id
        .send_noret(ctx, |m: &mut MessageCreator| {
            m.title("Image").content(format!(
                "**Valid images**\n`{}`",
                VALID_IMAGES
                    .iter()
                    .map(|s| String::from(&s.link))
                    .collect::<Vec<String>>()
                    .join("`, `")
            ))
        })
        .await
}

#[command]
#[aliases("img")]
#[description("Get an image")]
#[usage("<type>")]
#[example("erok")]
#[max_args(1)]
async fn image(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let valid_ids = VALID_IMAGES.iter().map(|s| String::from(&s.link)).collect::<Vec<String>>();

    if !args.is_empty() {
        let arg = args.current().unwrap().to_lowercase();

        if valid_ids.contains(&arg) {
            let selected: &Img = VALID_IMAGES.iter().find(|&s| s.link.eq(&arg)).unwrap();

            let is_too_nsfw = match selected.level {
                0 => false,
                1 => !matches!(
                    can_nsfw_moderate(ctx, msg).await,
                    serenity::framework::standard::CheckResult::Success
                ),
                _ => !matches!(
                    can_nsfw_strict(ctx, msg).await,
                    serenity::framework::standard::CheckResult::Success
                ),
            };

            if is_too_nsfw {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.warning().title("Error").content(format!(
                            "This server is marked not marked as NSFW and you've specified a NSFW image.\nThis can be \
                             overriden by executing `nsfwfilter {}`",
                            selected.level
                        ))
                    })
                    .await;
            }

            return do_image(ctx, msg, selected, 1, false).await;
        } else {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Image")
                        .content(format!("Unable to find image tagged '{}'", arg))
                })
                .await;
        }
    }

    msg.channel_id
        .send_noret(ctx, |m: &mut MessageCreator| {
            m.title("Image").content(format!(
                "**Valid images**\n`{}`",
                VALID_IMAGES
                    .iter()
                    .map(|s| String::from(&s.link))
                    .collect::<Vec<String>>()
                    .join("`, `")
            ))
        })
        .await
}
