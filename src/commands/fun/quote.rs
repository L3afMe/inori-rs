use once_cell::sync::Lazy;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{models::quotes::*, InoriChannelUtils, InoriMessageUtils, MessageCreator};

#[command]
#[description("Get a random quote from several people like Chuck Norris, Donald Trump and Kanye")]
#[sub_commands(kanyewest, donaldtrump, chucknorris)]
#[min_args(1)]
async fn quote(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("Quote")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

async fn get_result<T>(url: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned, {
    let res = if let Ok(res) = reqwest::get(url).await {
        if let Ok(res) = res.text().await {
            res
        } else {
            return Err("Unable to get string from response".to_string());
        }
    } else {
        return Err("Unable to send GET request".to_string());
    };

    if let Ok(res) = serde_json::from_str::<T>(&res) {
        return Ok(res);
    } else {
        return Err("Unable to format reponse into type T".to_string());
    }
}

#[command]
#[aliases("kanye")]
#[description("Random Kanye West quotes")]
async fn kanyewest(ctx: &Context, msg: &Message) -> CommandResult {
    let mut new_msg = msg.channel_id.send_loading(ctx, "Quote", "Loading quote").await.unwrap();
    let res = get_result::<KanyeRestResponse>("https://api.kanye.rest").await;

    return match res {
        Ok(result) => {
            new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.title("Quote").content(format!("{}\n ~ Kanye West", result.quote))
                })
                .await
        },
        Err(why) => {
            new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Quote").content(format!("Error getting quote: {}", why))
                })
                .await
        },
    };
}

#[command]
#[aliases("trump")]
#[description("Random stupid shit Donald Trump has said")]
async fn donaldtrump(ctx: &Context, msg: &Message) -> CommandResult {
    let mut new_msg = msg.channel_id.send_loading(ctx, "Quote", "Loading quote").await.unwrap();
    let res = get_result::<TronaldDumpReponse>("https://api.tronalddump.io/random/quote").await;

    return match res {
        Ok(result) => {
            new_msg
                .update_noret(ctx, |m: &mut MessageCreator| {
                    m.title("Quote").content(format!("{}\n ~ Donald Trump", result.value))
                })
                .await
        },
        Err(why) => {
            new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Quote").content(format!("Error getting quote: {}", why))
                })
                .await
        },
    };
}

static TAGS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "animal",
        "career",
        "celebrity",
        "dev",
        "explicit",
        "fashion",
        "food",
        "history",
        "money",
        "movie",
        "music",
        "political",
        "religion",
        "science",
        "sport",
        "travel",
    ]
    .iter()
    .map(|e| e.to_string())
    .collect::<Vec<String>>()
});

#[command]
#[aliases("chuck")]
#[description("Random Chuck Norris quotes")]
#[min_args(0)]
#[max_args(1)]
async fn chucknorris(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut new_msg = msg.channel_id.send_loading(ctx, "Quote", "Loading quote").await.unwrap();

    let res = if args.is_empty() {
        get_result::<ChuckNorrisIoResponse>("https://api.chucknorris.io/jokes/random").await
    } else {
        let tag = args.current().unwrap().to_lowercase();

        if TAGS.contains(&tag) {
            get_result::<ChuckNorrisIoResponse>(&format!("https://api.chucknorris.io/jokes/random?category={}", tag))
                .await
        } else {
            return new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.error()
                        .title("Quote")
                        .content(format!("Invalid tag specified.\nValid tags: `{}`", TAGS.join("`, `")))
                })
                .await;
        }
    };

    return match res {
        Ok(result) => {
            new_msg
                .update_noret(ctx, |m: &mut MessageCreator| {
                    m.title("Quote").content(format!("{}\n ~ Chuck Norris", result.value))
                })
                .await
        },
        Err(why) => {
            new_msg
                .update_tmp(ctx, |m: &mut MessageCreator| {
                    m.error().title("Quote").content(format!("Error getting quote: {}", why))
                })
                .await
        },
    };
}

// #[command]
// #[aliases("ron")]
// #[description("Random Ron Swanson quotes")]
// async fn ronswanson(ctx: &Context, msg:
// &Message) -> CommandResult {
// https://ron-swanson-quotes.herokuapp.com/v2/quotes
// Ok(())
// }
