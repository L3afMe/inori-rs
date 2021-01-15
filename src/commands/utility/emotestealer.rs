use serenity::{
    builder::CreateMessage,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::read_image,
};

use serenity_utils::menu::*;

use rand::Rng;
use std::sync::Arc;
use tokio::{
    fs::{remove_file, File},
    prelude::*,
};

use crate::{
    save_settings,
    utils::chat::{default_embed, delete, get_emotes, has_emotes, say, say_error, send_loading},
    Settings,
};

#[command]
#[aliases("s")]
#[description("Set the server which emotes will be uploaded to")]
#[required_permissions("MANAGE_EMOJIS")]
#[only_in("guild")]
async fn server(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    settings.emoteserver = msg.guild_id.unwrap().0;
    save_settings(&settings);

    drop(settings);
    drop(data);

    say(ctx, &msg.channel_id, "Emote Sniper", "Updated emote server").await
}

#[command]
#[aliases("es")]
#[description("Steals emotes from the last 10 messages in chat or sent in the message")]
#[sub_commands(server)]
async fn emotestealer(ctx: &Context, msg: &Message) -> CommandResult {
    let emotes = if let Some(msg_ref) = &msg.message_reference {
        let msg: Message = if let Ok(msg) = ctx
            .http
            .get_message(msg_ref.channel_id.0, msg_ref.message_id.unwrap().0)
            .await
        {
            msg
        } else {
            say(
                ctx,
                &msg.channel_id,
                "Emote Stealer",
                "Couldn't get message (Known bug)",
            )
            .await?;
            return Ok(());
        };

        let emotes = get_emotes(&msg.content);

        emotes
    } else {
        if has_emotes(&msg.content) {
            get_emotes(&msg.content)
        } else {
            let channel = &ctx
                .http
                .get_channel(msg.channel_id.0)
                .await
                .unwrap()
                .guild()
                .unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(10))
                .await?;

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
        say_error(ctx, &msg.channel_id, "Emote Stealer", "No emotes found").await?;
        return Ok(());
    }

    let controls = vec![
        Control::new(
            ReactionType::from('◀'),
            Arc::new(|m, r| Box::pin(prev_page(m, r))),
        ),
        Control::new(
            ReactionType::from('❌'),
            Arc::new(|m, r| Box::pin(close_menu(m, r))),
        ),
        Control::new(
            ReactionType::from('✅'),
            Arc::new(|m, r| Box::pin(save(m, r))),
        ),
        Control::new(
            ReactionType::from('▶'),
            Arc::new(|m, r| Box::pin(next_page(m, r))),
        ),
    ];

    let mut embeds = Vec::new();
    for (idx, emote) in emotes.iter().enumerate() {
        let mut embed = CreateMessage::default();
        embed.embed(|e| {
            e.0 = default_embed("Emote Stealer").0;

            e.description(format!("**{}**", emote.name))
                .image(&emote.url)
                .footer(|f| f.text(format!("Page {} of {}", idx + 1, emotes.len())))
        });

        embeds.push(embed);
    }

    let server_set = {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

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

    let menu = Menu::new(ctx, msg, &embeds[..], options);
    menu.run().await?;

    Ok(())
}

async fn save<'a>(menu: &mut Menu<'a>, _reaction: Reaction) {
    let mut new_msg = send_loading(
        menu.ctx,
        &menu.msg.channel_id,
        "Emote Stealer",
        "Adding image",
    )
    .await;
    let embed = &menu.pages[menu.options.page].0["embed"];

    let mut emote_url = embed["image"]["url"].to_string();
    emote_url = emote_url[1..emote_url.len() - 1].to_string();

    let mut emote_name = embed["description"].to_string();
    emote_name = emote_name[3..emote_name.len() - 3].to_string();

    let url = emote_url.split("?").next().unwrap().to_string();
    let ext = &url.to_string()[url.rfind(".").unwrap()..];

    let img = reqwest::get(&emote_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    let path_str = &format!(
        "./tmp_{}.{}",
        rand::thread_rng().gen_range(100000000..999999999),
        ext
    );
    let mut file = File::create(path_str.to_string()).await.unwrap();
    file.write_all(&img).await.unwrap();
    let emote_image = read_image(path_str.to_string()).unwrap();

    let data = menu.ctx.data.read().await;
    let settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    match GuildId(settings.emoteserver)
        .create_emoji(&menu.ctx.http, &emote_name, &emote_image)
        .await
    {
        Ok(_) => {}
        Err(why) => {
            return say_error(
                &menu.ctx,
                &menu.msg.channel_id,
                "Emote Stealer",
                &format!("Couldn't add new emote\nError: {:?}", why),
            )
            .await
            .unwrap_or(());
        }
    }

    remove_file(path_str.to_string()).await.unwrap();

    drop(settings);
    drop(data);

    new_msg
        .edit(&menu.ctx.http, |m| {
            m.embed(|e| {
                e.0 = default_embed("Emote Stealer").0;

                e.description(format!("Added emote '{}'", emote_name))
                    .image(emote_url)
            })
        })
        .await
        .unwrap();

    delete(menu.ctx, &new_msg)
        .await
        .expect("Unable to delete message")
}
