use brainfrick::Brainfuck;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

use crate::{InoriChannelUtils, MessageCreator};

#[group]
#[commands(brainfuck)]
#[description("**Programming**")]
struct Programming;

static BRAINFUCK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[+-<>.,\[\]]*$").unwrap());

#[command]
#[aliases("bf")]
#[description(
    "Interpret Brainfuck code and return the result. Stepping through iterations will come when paginator is more \
     stable"
)]
async fn brainfuck(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let bf = args.rest().to_string();

    if !BRAINFUCK_REGEX.is_match(&bf) {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error().title("Brainfuck").content(
                    "Invalid Brainfuck code given, code can only contain  `+`, `-`, `<`, `>`, `,`, `.`, `[` and `]`",
                )
            })
            .await;
    }

    match Brainfuck::execute(&bf) {
        Ok(out) => {
            msg.channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.title("Brainfuck")
                        .content(&format!("Input: ```brainfuck\n{}\n```\nOutput: ```\n{}\n```", bf, out))
                })
                .await
        },
        Err(why) => {
            msg.channel_id
                .send_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Brainfuck").content(&format!(
                        "Unable to interpret Brainfuck.\nError: {}\nInput: ```brainfuck\n{}\n```",
                        why, bf,
                    ))
                })
                .await
        },
    }
}
