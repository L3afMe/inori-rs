use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
#[aliases("delete", "remove", "del", "rem", "d", "r")]
#[description("Delete an AutoMessage")]
#[num_args(1)]

async fn delete(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    Ok(())
}

#[command]
#[aliases("add", "a")]
#[description("Add a new AutoMessage")]
#[num_args(1)]

async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":dog:").await?;

    Ok(())
}

#[command]
#[aliases("list", "l")]
#[description("List all AutoMessages")]

async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":dog:").await?;

    Ok(())
}

#[command]
#[aliases("automsg")]
#[description("Send messages at specific intervals")]
#[min_args(1)]
#[sub_commands(add, delete, list)]

async fn automessage(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        "".to_string()
    } else {
        format!("Unknown automessage command: {}", args.rest())
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}
