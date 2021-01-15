use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::ChannelId},
    prelude::*,
};

use crate::{save_settings, utils::chat, Settings};

async fn say(ctx: &Context, channel: &ChannelId, content: &str) -> CommandResult {
    chat::say(ctx, channel, "Tags", content).await
}

async fn say_error(ctx: &Context, channel: &ChannelId, content: &str) -> CommandResult {
    chat::say_error(ctx, channel, "Tags", content).await
}

async fn send(ctx: &Context, channel: &ChannelId, content: &str) -> Message {
    chat::send(ctx, channel, "Tags", content).await
}

#[command]
#[aliases("delete", "remove", "del", "rem", "d", "r")]
#[description("Delete a specific tag")]
#[usage("<tag>")]
#[example("TODO")]
#[num_args(1)]
async fn delete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();

    if settings.tags.contains_key(&name) {
        settings.tags.remove(&name);
        save_settings(&settings);

        drop(settings);
        drop(data);

        say(
            ctx,
            &msg.channel_id,
            &format!("Removed tag with name '{}'", name),
        )
        .await
    } else {
        drop(settings);
        drop(data);

        say_error(ctx, &msg.channel_id, "Invalid tag name").await
    }
}

#[command]
#[description("Preppend text to the end of a tag")]
#[usage("<tag> <message>")]
#[example("TODO I'm at the start!")]
#[min_args(2)]
async fn preppend(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();
    let message = args.rest().replace("\\n", "\n");

    if !settings.tags.contains_key(&name) {
        drop(settings);
        drop(data);

        say_error(
            ctx,
            &msg.channel_id,
            &format!("Tag with name '{}' doesn't exists", name),
        )
        .await
    } else {
        let old_msg = (&settings.tags.get(&name).unwrap()).to_string();
        settings.tags.remove(&name);

        let new_msg = format!("{} {}", message.to_string(), old_msg);
        settings.tags.insert(name.to_string(), new_msg);
        save_settings(&settings);

        drop(settings);
        drop(data);

        say(
            ctx,
            &msg.channel_id,
            &format!("Updated tag with name '{}'", name),
        )
        .await
    }
}

#[command]
#[description("Append text to the end of a tag")]
#[usage("<tag> <message>")]
#[example("TODO I'm at the end!")]
#[min_args(2)]
async fn append(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();
    let message = args.rest().replace("\\n", "\n");

    if !settings.tags.contains_key(&name) {
        drop(settings);
        drop(data);

        say_error(
            ctx,
            &msg.channel_id,
            &format!("Tag with name '{}' doesn't exists", name),
        )
        .await
    } else {
        let old_msg = (&settings.tags.get(&name).unwrap()).to_string();
        settings.tags.remove(&name);

        let new_msg = format!("{} {}", old_msg, message.to_string());
        settings.tags.insert(name.to_string(), new_msg);
        save_settings(&settings);

        drop(settings);
        drop(data);

        say(
            ctx,
            &msg.channel_id,
            &format!("Updated tag with name '{}'", name),
        )
        .await
    }
}

#[command]
#[description("Replace text in a tag")]
#[usage("<tag> <search text> | <replacement text>")]
#[example("Hello, World | Goodbye, World")]
#[min_args(3)]
async fn replace(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();
    let message = args.rest();

    if !settings.tags.contains_key(&name) {
        drop(settings);
        drop(data);

        say_error(
            ctx,
            &msg.channel_id,
            &format!("Tag with name '{}' doesn't exists", name),
        )
        .await
    } else {
        if message.contains("|") {
            let split = message.split("|").collect::<Vec<&str>>();
            let search_text = split.get(0).unwrap().trim();
            let replacement_text = split.get(1).unwrap();

            let old_msg = (&settings.tags.get(&name).unwrap()).to_string();
            settings.tags.remove(&name);

            let new_msg = old_msg.replace(search_text, replacement_text);
            settings.tags.insert(name.to_string(), new_msg);
            save_settings(&settings);

            drop(settings);
            drop(data);

            return say(
                ctx,
                &msg.channel_id,
                &format!("Updated tag with name '{}'", name),
            )
            .await;
        } else {
            drop(settings);
            drop(data);

            return say_error(ctx, &msg.channel_id, "No replacement text found").await;
        }
    }
}

#[command]
#[description("Edit a tags message")]
#[usage("<tag> <message>")]
#[example("TODO Something new I need to do")]
#[min_args(2)]
async fn edit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();
    let message = args.rest();

    if !settings.tags.contains_key(&name) {
        drop(settings);
        drop(data);

        say_error(
            ctx,
            &msg.channel_id,
            &format!("Tag with name '{}' doesn't exists", name),
        )
        .await
    } else {
        settings.tags.remove(&name);

        settings.tags.insert(name.to_string(), message.to_string());
        save_settings(&settings);

        drop(settings);
        drop(data);

        return say(
            ctx,
            &msg.channel_id,
            &format!("Updated tag with name '{}'", name),
        )
        .await;
    }
}

#[command]
#[aliases("add", "a")]
#[description("Add a new tag")]
#[usage("<tag> <message>")]
#[example("TODO Something I need to do")]
#[min_args(2)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let mut settings = data
        .get::<Settings>()
        .expect("Expected Setting in TypeMap.")
        .lock()
        .await;

    let name = args.single::<String>().unwrap();
    let message = args.rest().replace("\\n", "\n");

    if settings.tags.contains_key(&name) {
        drop(settings);
        drop(data);

        return say_error(
            ctx,
            &msg.channel_id,
            &format!("Tag with name '{}' already exists", name),
        )
        .await;
    } else {
        settings.tags.insert(name.to_string(), message.to_string());
        save_settings(&settings);

        drop(settings);
        drop(data);

        return say(
            ctx,
            &msg.channel_id,
            &format!("Added tag with name '{}'", name),
        )
        .await;
    }
}

async fn _list(ctx: &Context, msg: &Message) -> CommandResult {
    let content = {
        let data = ctx.data.read().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;

        let mut content = "".to_string();

        if settings.tags.is_empty() {
            content = "Nothing to see here".to_string();
        }

        for (name, message) in &settings.tags {
            content = format!("{}\n\n**{}**\n{}", content, name, message);
        }

        content
    };

    say(ctx, &msg.channel_id, &content).await
}

#[command]
#[aliases("list", "l")]
#[description = "List all tags"]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    _list(ctx, msg).await
}

#[command]
#[aliases("tag")]
#[description("Display a specified tag")]
#[usage("<tag/subcommand>")]
#[example("add TODO Something I need to do")]
#[example("TODO")]
#[example("delete TODO")]
#[sub_commands(add, delete, list, preppend, append, edit, replace)]
async fn tags(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        _list(ctx, msg).await?;
    } else {
        let data = ctx.data.write().await;
        let settings = data
            .get::<Settings>()
            .expect("Expected Setting in TypeMap.")
            .lock()
            .await;
        let name = args.rest();

        if settings.tags.contains_key(name) {
            let message = settings.tags.get(name).unwrap().to_string();
            drop(settings);
            drop(data);
            send(ctx, &msg.channel_id, &format!("**{}**\n{}", name, message)).await;
        } else {
            drop(settings);
            drop(data);
            say_error(
                ctx,
                &msg.channel_id,
                &format!("Unknown tag: {}", args.rest()),
            )
            .await?;
        }
    };

    Ok(())
}
