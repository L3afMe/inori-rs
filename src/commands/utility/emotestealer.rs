use std::sync::Arc;

use rand::Rng;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::read_image,
};
use serenity_utils::menu::*;
use tokio::{
    fs::{remove_file, File},
    prelude::*,
};

use crate::{
    save_settings,
    utils::chat::{get_emotes, has_emotes},
    InoriChannelUtils, InoriMessageUtils, MessageCreator, Settings,
};

#[command]
#[aliases("s")]
#[description("Set the server which emotes will be uploaded to")]
#[required_permissions("MANAGE_EMOJIS")]
#[only_in("guild")]

async fn server(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;

    let mut settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

    settings.emoteserver = msg.guild_id.unwrap().0;

    save_settings(&settings);

    drop(settings);

    drop(data);

    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error().title("Emote Stealer").content("Updated emote server")
        })
        .await
}

#[command]
#[aliases("es")]
#[description("Steals emotes from the last 10 messages in chat or sent in the message")]
#[sub_commands(server)]

async fn emotestealer(ctx: &Context, msg: &Message) -> CommandResult {
    let emotes = if let Some(msg_ref) = &msg.message_reference {
        let msg: Message =
            if let Ok(msg) = ctx.http.get_message(msg_ref.channel_id.0, msg_ref.message_id.unwrap().0).await {
                msg
            } else {
                return msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| {
                        m.error().title("Emote Stealer").content("Couldn't get message (Known bug)")
                    })
                    .await;
            };

        let emotes = get_emotes(&msg.content);

        emotes
    } else {
        if has_emotes(&msg.content) {
            get_emotes(&msg.content)
        } else {
            let channel = &ctx.http.get_channel(msg.channel_id.0).await.unwrap().guild().unwrap();

            let messages = &channel.messages(ctx, |r| r.before(&msg.id).limit(10)).await?;

            let mut emotes = Vec::new();

            for message in messages {
                if has_emotes(&message.content) {
                    let new_emotes = get_emotes(&message.content);

                    for emote in new_emotes {
                        if !emotes.contains(&emote) {
                            emotes.push(emote);
                        }
                    }
                }
            }

            emotes
        }
    };

    if emotes.is_empty() {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Emote Stealer").content("No emotes found")
            })
            .await;
    }

    let controls = vec![
        Control::new(ReactionType::from('◀'), Arc::new(|m, r| Box::pin(prev_page(m, r)))),
        Control::new(ReactionType::from('❌'), Arc::new(|m, r| Box::pin(close_menu(m, r)))),
        Control::new(ReactionType::from('✅'), Arc::new(|m, r| Box::pin(save(m, r)))),
        Control::new(ReactionType::from('▶'), Arc::new(|m, r| Box::pin(next_page(m, r)))),
    ];

    let mut msgs = Vec::new();

    for emote in emotes {
        let mut msg = MessageCreator::default();

        msg.title("Emote Stealer")
            .content(format!("**{}**", emote.name))
            .image(emote.url);

        msgs.push(msg);
    }

    let server_set = {
        let data = ctx.data.read().await;

        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        settings.emoteserver != 0
    };

    let options = if server_set {
        MenuOptions {
            controls,
            ..Default::default()
        }
    } else {
        MenuOptions::default()
    };

    msg.channel_id.send_paginatorwo_noret(ctx, msg, msgs, options).await
}

async fn save<'a>(menu: &mut Menu<'a>, _reaction: Reaction) {
    let mut new_msg = menu
        .msg
        .channel_id
        .send_loading(menu.ctx, "Emote Stealer", "Adding image")
        .await
        .unwrap();

    let embed = &menu.pages[menu.options.page].0["embed"].clone();

    let mut emote_url = embed["image"]["url"].to_string();

    emote_url = emote_url[1..emote_url.len() - 1].to_string();

    let mut emote_name = embed["description"].to_string();

    emote_name = emote_name[3..emote_name.len() - 3].to_string();

    let url = emote_url.split("?").next().unwrap().to_string();

    let ext = url[url.rfind(".").unwrap()..].to_string();

    let path_str = format!("./tmp_{}.{}", rand::thread_rng().gen_range(100000000..999999999), ext);

    let ctx = menu.ctx.clone();

    let msg = menu.msg.clone();

    let _ = tokio::task::spawn(async move {
        let img = reqwest::get(&emote_url).await.unwrap().bytes().await.unwrap();

        let mut file = File::create(path_str.to_string()).await.unwrap();

        file.write_all(&img).await.unwrap();

        let emote_image = read_image(path_str.to_string()).unwrap();

        let data = ctx.data.read().await;

        let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

        match GuildId(settings.emoteserver)
            .create_emoji(&ctx.http, &emote_name, &emote_image)
            .await
        {
            Ok(_) => {},
            Err(why) => {
                remove_file(path_str.to_string()).await.unwrap();

                drop(settings);

                drop(data);

                let _ = msg
                    .channel_id
                    .send_tmp(&ctx, |m: &mut MessageCreator| {
                        m.error()
                            .title("Emote Stealer")
                            .content(format!("Couldn't add new emote\nError: {:?}", why))
                    })
                    .await;

                return;
            },
        }

        remove_file(path_str.to_string()).await.unwrap();

        drop(settings);

        drop(data);

        let _ = new_msg
            .update_tmp(&ctx, |m: &mut MessageCreator| {
                m.title("Emote Creator")
                    .title(format!("Added emote '{}'", emote_name))
                    .image(emote_url)
            })
            .await;
    })
    .await;
}
