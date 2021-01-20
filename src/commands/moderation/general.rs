use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::MessageId},
    prelude::Context,
};

use crate::{InoriChannelUtils, InoriMessageUtils, MessageCreator};

#[command]
#[aliases("prunechat", "clearchat")]
#[description("Deletes a specified amount of messages")]
#[usage("<amount>")]
#[example("20")]
#[required_permissions("MANAGE_MESSAGES")]
#[num_args(1)]
async fn purgechat(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let delete_num = args.single::<u64>();

    match delete_num {
        Err(_) => {
            return msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Purge Chat")
                        .content(":no_entry_sign: The value provided was not a valid number")
                })
                .await;
        },

        Ok(delete_n) => {
            let mut find_msg = msg
                .channel_id
                .send_loading(ctx, "Purge Chat", &format!("Finding and deleting {} messages", delete_n))
                .await
                .unwrap();

            let channel = &ctx.http.get_channel(msg.channel_id.0).await.unwrap().guild().unwrap();
            let messages = &channel.messages(ctx, |r| r.before(&msg.id).limit(delete_n)).await?;
            let message_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            for message_id in message_ids {
                ctx.http.delete_message(msg.channel_id.0, message_id.0).await?;
            }

            return find_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.title("Purge Chat")
                        .content(format!(":white_check_mark: Deleted {} messages", delete_n))
                })
                .await;
        },
    }
}
