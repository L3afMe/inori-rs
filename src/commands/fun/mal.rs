use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};
use urlencoding::encode;

use crate::models::{
    commands::{
        MALAnimeSearchResult, MALCharacterSearchResult, MALMangaSearchResult, MALPersonSearchResult, MALSearchResponse,
    },
    discord::{InoriChannelUtils, MessageCreator},
};

#[command]
#[aliases("mal")]
#[description("Search MyAnimeList for your favorite anime, manga, character or actor")]
#[usage("<subcommand>")]
#[example("character Inori Yuzuriha")]
#[example("anime Kimi no na wa")]
#[sub_commands(anime, manga, character, actor)]
#[min_args(1)]
async fn myanimelist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id
        .send_tmp(ctx, |m: &mut MessageCreator| {
            m.error()
                .title("MyAnimeList")
                .content(format!("Unknown subcommand: {}", args.current().unwrap()))
        })
        .await
}

static BASE_URL: &str = "https://api.jikan.moe/v3/";

#[command]
#[description("Search for anime and manga voice actors")]
#[usage("<name>")]
#[example("Yoshitsugu Matsuoka")]
async fn actor(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();

    if params.len() < 3 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("MyAnimeList")
                    .content("Search queries must be 3 characters or longer")
            })
            .await;
    }

    let new_msg = msg
        .channel_id
        .send_loading(ctx, "MyAnimeList", "Loading actor information")
        .await
        .unwrap();

    let res = reqwest::get(&format!("{}search/person?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALPersonSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.is_empty() {
        new_msg.delete(&ctx.http).await?;

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
                    .title("MyAnimeList")
                    .content(&format!("No results found for query: `{}`", params))
            })
            .await;
    } else {
        let mut msgs = Vec::new();

        for result in res.results {
            let mut msg = MessageCreator::default();

            msg.title("MyAnimeList")
                .content(format!("[{}]({})", result.name, result.url))
                .thumbnail(&result.image_url);

            if !result.alternate_names.is_empty() {
                msg.field("Alternative Names", result.alternate_names.join("\n"), true);
            }

            msg.field("MAL ID", result.mal_id, true);
            msgs.push(msg);
        }

        new_msg.delete(&ctx.http).await?;

        return msg.channel_id.send_paginator_noret(ctx, msg, msgs).await;
    }
}

#[command]
#[description("Search for character")]
#[usage("<name>")]
#[example("Zero Two")]
async fn character(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();

    if params.len() < 3 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("MyAnimeList")
                    .content("Search queries must be 3 characters or longer")
            })
            .await;
    }

    let new_msg = msg
        .channel_id
        .send_loading(ctx, "MyAnimeList", "Loading character information")
        .await
        .unwrap();

    let res = reqwest::get(&format!("{}search/character?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALCharacterSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.is_empty() {
        new_msg.delete(&ctx.http).await?;

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
                    .title("MyAnimeList")
                    .content(&format!("No results found for query: `{}`", params))
            })
            .await;
    } else {
        let mut msgs = Vec::new();

        for result in res.results {
            let mut msg = MessageCreator::default();

            msg.title("MyAnimeList")
                .thumbnail(&result.image_url)
                .content(format!("[{}]({})", result.name, result.url));

            if !result.alternate_names.is_empty() {
                msg.field("Alternative Names", result.alternate_names.join("\n"), true);
            }

            if !result.anime.is_empty() {
                let anime_list = result
                    .anime
                    .iter()
                    .map(|e| format!("[{}]({}) ({})", e.name, e.url, e.mal_id))
                    .collect::<Vec<String>>();

                msg.field("Anime", anime_list.join("\n"), true);
            }

            if !result.manga.is_empty() {
                let manga_list = result
                    .manga
                    .iter()
                    .map(|e| format!("[{}]({}) ({})", e.name, e.url, e.mal_id))
                    .collect::<Vec<String>>();

                msg.field("Manga", manga_list.join("\n"), true);
            }

            msg.field("MAL ID", result.mal_id, true);
            msgs.push(msg);
        }

        new_msg.delete(&ctx.http).await?;

        return msg.channel_id.send_paginator_noret(ctx, msg, msgs).await;
    }
}

#[command]
#[description("Search for manga")]
#[usage("<name>")]
#[example("One Piece")]
async fn manga(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();

    if params.len() < 3 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("MyAnimeList")
                    .content("Search queries must be 3 characters or longer")
            })
            .await;
    }

    let new_msg = msg
        .channel_id
        .send_loading(ctx, "MyAnimeList", "Loading manga information")
        .await
        .unwrap();

    let res = reqwest::get(&format!("{}search/manga?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALMangaSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.is_empty() {
        new_msg.delete(&ctx.http).await?;

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
                    .title("MyAnimeList")
                    .content(&format!("No results found for query: `{}`", params))
            })
            .await;
    } else {
        let mut msgs = Vec::new();

        for result in res.results {
            let mut msg = MessageCreator::default();

            msg.title("MyAnimeList")
                .thumbnail(&result.image_url)
                .content(format!("**[{}]({})**\n{}", result.title, result.url, result.synopsis))
                .field("Volumes", result.chapters, true)
                .field("Chapters", result.chapters, true)
                .field("Score", result.score, true)
                .field("Members", result.members, true)
                .field("Start Date", &result.start_date, true)
                .field("End Date", &result.end_date, true)
                .field("MAL ID", result.mal_id, true);

            msgs.push(msg);
        }

        new_msg.delete(&ctx.http).await?;

        return msg.channel_id.send_paginator_noret(ctx, msg, msgs).await;
    }
}

#[command]
#[description("Search for anime")]
#[usage("<name>")]
#[example("Shingeki no Kyojin")]
async fn anime(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();

    if params.len() < 3 {
        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.error()
                    .title("MyAnimeList")
                    .content("Search queries must be 3 characters or longer")
            })
            .await;
    }

    let new_msg = msg
        .channel_id
        .send_loading(ctx, "MyAnimeList", "Loading anime information")
        .await
        .unwrap();

    let res = reqwest::get(&format!("{}search/anime?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALAnimeSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.is_empty() {
        new_msg.delete(&ctx.http).await?;

        return msg
            .channel_id
            .send_tmp(ctx, |m: &mut MessageCreator| {
                m.warning()
                    .title("MyAnimeList")
                    .content(&format!("No results found for query: `{}`", params))
            })
            .await;
    } else {
        let mut msgs = Vec::new();

        for result in res.results {
            let mut msg = MessageCreator::default();

            msg.title("MyAnimeList")
                .thumbnail(&result.image_url)
                .content(format!("**[{}]({})**\n{}", result.title, result.url, result.synopsis))
                .field("Episodes", result.episodes, true)
                .field("Score", result.score, true)
                .field("Members", result.members, true)
                .field("Start Date", &result.start_date, true)
                .field("End Date", &result.end_date, true)
                .field("Rated", &result.rated, true)
                .field("MAL ID", result.mal_id, true);

            msgs.push(msg);
        }

        new_msg.delete(&ctx.http).await?;

        return msg.channel_id.send_paginator_noret(ctx, msg, msgs).await;
    }
}
