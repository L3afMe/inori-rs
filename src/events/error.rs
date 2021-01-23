use serenity::{
    framework::standard::{macros::hook, DispatchError, Reason},
    model::channel::Message,
    prelude::Context,
};

use crate::models::discord::{InoriChannelUtils, MessageCreator};

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    ctx.http.delete_message(msg.channel_id.0, msg.id.0).await.unwrap();

    match error {
        DispatchError::Ratelimited(duration) => {
            let content = format!("Try this again in {} seconds.", duration.as_secs());

            println!("[Error] Ratelimit, {}", content);
            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| m.error().title("Ratelimit").content(content))
                .await;
        },

        DispatchError::CheckFailed(_, reason) => {
            if let Reason::User(err) = reason {
                let content = match err.as_ref() {
                    "nsfw_moderate" => "This channel is not marked as NSFW and you've specified a NSFW image.\nThis \
                                        can be overriden by executing `nsfwfilter 1`"
                        .to_string(),
                    "nsfw_strict" => "This channel is not marked as NSFW and you've specified a NSFW image.\nThis can \
                                      be overriden by executing `nsfwfilter 2`"
                        .to_string(),
                    _ => {
                        let content = format!("Undocumted error, please report this to L3af#0001\nError: `{:?}``", err);
                        println!("{}", content);

                        content
                    },
                };

                let _ = msg
                    .channel_id
                    .send_tmp(ctx, |m: &mut MessageCreator| m.warning().title("Error").content(content))
                    .await;
            }
        },

        DispatchError::TooManyArguments {
            max,
            given,
        } => {
            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Error")
                        .content(&format!("Too many args given!\nMaximum: {}, Given: {}", max, given))
                })
                .await;
        },

        DispatchError::NotEnoughArguments {
            min,
            given,
        } => {
            let _ = msg
                .channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Error")
                        .content(&format!("To few args given!\nMinimum: {}, Given: {}", min, given))
                })
                .await;
        },

        _ => {
            println!(
                "Unhandled dispatch error, please contact #L3af#0001 about this.\nError: {:?}",
                error
            );
        },
    };
}
