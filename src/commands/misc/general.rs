use rand::Rng;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, prelude::ReactionType, user::User},
    prelude::Context,
};

use crate::{utils::consts, InoriChannelUtils, MessageCreator};

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
                    "[Serenity](https://github.com/serenity-rs/serenity) with \
                    [SelfBot support](https://github.com/L3afMe/serenity-selfbot-support)",
                    true,
                )
                .field(
                    "Author",
                    format!("{} ({})", consts::AUTHOR_NAME, consts::AUTHOR_DISC),
                    true,
                )
                .field("GitHub Repo", "https://github.com/L3afMe/inori-rs", true)
                .image(
                    "https://static.wikia.nocookie.net/guiltycrown/mages\
                    /a/a5/Guilty_Crown_-_01_-_Large_17.jpg",
                )
        })
        .await
}

async fn print_av(ctx: &Context, msg: &Message, user: &User) -> CommandResult {
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
async fn avatar(ctx: &Context, msg: &Message) -> CommandResult {
    if !msg.mentions.is_empty() {
        for mention in &msg.mentions {
            let _ = print_av(ctx, msg, mention).await.unwrap_or(());
        }
    } else {
        let _ = print_av(ctx, msg, &msg.author).await;
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
                        m.title("Hype Squad Changer").content(format!("Set house to {}", house))
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
