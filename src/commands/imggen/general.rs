extern crate serde;
extern crate serde_json;
extern crate urlencoding;

use urlencoding::encode;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::{
    models::commands::NekoBotResponse,
    utils::{
        chat::{default_embed, is_mention, say_error, send_loading},
        user::get_av,
    },
};

async fn neko_bot(ctx: &Context, msg: &Message, url: &str, title: &str) {
    let mut msg = send_loading(ctx, &msg.channel_id, title, "Generating image").await;

    let res = reqwest::get(url).await.unwrap().text().await.unwrap();

    let res = serde_json::from_str::<NekoBotResponse>(&res).expect("Couldn't parse response.");

    let mut embed = default_embed(title);
    embed.image(res.message);

    msg.edit(&ctx, |m| {
        m.embed(|e| {
            e.0 = embed.0;
            e
        })
    })
    .await
    .unwrap();
}

#[command]
#[description("Generate an image of a message from Clyde")]
#[usage("<message>")]
#[example("Stop being such an idiot")]
#[min_args(1)]
async fn clyde(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    neko_bot(
        ctx,
        msg,
        &format!(
            "https://nekobot.xyz/api/imagegen?type=clyde&text={}",
            encode(args.rest())
        ),
        "Clyde",
    )
    .await;

    Ok(())
}

#[command]
#[aliases("kanna")]
#[description = "Generate an image of Kanna holdinga specified message"]
#[usage("<message>")]
#[example("Thighs are life")]
#[min_args(1)]
async fn kannagen(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    neko_bot(
        ctx,
        msg,
        &format!(
            "https://nekobot.xyz/api/imagegen?type=kannagen&text={}",
            encode(args.rest())
        ),
        "Kanna Gen",
    )
    .await;

    Ok(())
}

#[command]
#[aliases("ph")]
#[description("Generate a PornHub comment with a specified message")]
#[usage("[@user] <message>")]
#[example("@L3af#0001 If only she was a cat girl")]
#[example("What a beautiful body")]
#[min_args(1)]
async fn phcomment(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut user = &msg.author;
    let first_arg = args.single::<String>()?;
    let mut message = args.rest().to_string();

    if is_mention(&first_arg) && msg.mentions.len() >= 1 {
        user = msg.mentions.get(0).unwrap();

        if message.len() == 0 {
            return say_error(
                ctx,
                &msg.channel_id,
                "PornHub Comment",
                "You need to specify a message",
            )
            .await;
        }
    } else {
        message = format!("{} {}", first_arg, message);
    }

    neko_bot(
        ctx,
        msg,
        &format!(
            "https://nekobot.xyz/api/imagegen?type=phcomment&text={}&image={}&username={}",
            encode(&message),
            encode(&get_av(user).await),
            encode(&user.name)
        ),
        "PornHub Comment",
    )
    .await;

    Ok(())
}

#[command]
#[aliases("tt")]
#[description("Generate a Trump tweet with a specified message")]
#[usage("<message>")]
#[example("Catgirls are all that matter")]
#[min_args(1)]
async fn trumptweet(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    neko_bot(
        ctx,
        msg,
        &format!(
            "https://nekobot.xyz/api/imagegen?type=trumptweet&text={}",
            encode(args.rest())
        ),
        "Trump Tweet",
    )
    .await;

    Ok(())
}

#[command]
#[aliases("cmm")]
#[description("Generate a Change My Mind image with a specified message")]
#[usage("<opinion>")]
#[example("SelfBots shouldn't be against TOS")]
#[min_args(1)]
async fn changemymind(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    neko_bot(
        ctx,
        msg,
        &format!(
            "https://nekobot.xyz/api/imagegen?type=changemymind&text={}",
            encode(args.rest())
        ),
        "Change My Mind",
    )
    .await;

    Ok(())
}

#[command]
#[description("Lolice Chief'ify mentioned users profile picture")]
#[usage("[@user]")]
#[example("@L3af#0001")]
async fn lolice(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.is_empty() {
        neko_bot(
            ctx,
            msg,
            &format!(
                "https://nekobot.xyz/api/imagegen?type=lolice&url={}",
                encode(&get_av(&msg.author).await)
            ),
            "Lolice",
        )
        .await;
    } else {
        for mention in &msg.mentions {
            neko_bot(
                ctx,
                msg,
                &format!(
                    "https://nekobot.xyz/api/imagegen?type=lolice&url={}",
                    encode(&get_av(&mention).await)
                ),
                "Lolice",
            )
            .await;
        }
    }

    Ok(())
}

#[command]
#[description("Generates a captcha with a users profile picture")]
#[usage("<@user>")]
#[example("@L3af#0001")]
async fn cutie(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.is_empty() {
        neko_bot(
            ctx,
            msg,
            &format!(
                "https://nekobot.xyz/api/imagegen?type=captcha&url={}&username=a%20cute%20in%20them",
                encode(&get_av(&msg.author).await)
            ),
            "Cutie",
        )
        .await;
    } else {
        for mention in &msg.mentions {
            neko_bot(
                ctx,
                msg,
                &format!(
                    "https://nekobot.xyz/api/imagegen?type=captcha&url={}&username=a%20cutie%20in%20them",
                    encode(&get_av(&mention).await)
                ),
                "Cutie",
            )
            .await;
        }
    }

    Ok(())
}
