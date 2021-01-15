use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use urlencoding::encode;

use crate::{
    models::commands::{
        MALAnimeSearchResult, MALCharacterSearchResult, MALMangaSearchResult,
        MALPersonSearchResult, MALSearchResponse,
    },
    utils::chat::{default_embed, say_error, send_embed_paginator, send_loading},
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
    say_error(
        ctx,
        &msg.channel_id,
        "MyAnimeList",
        &format!("Unknown subcommand: {}", args.current().unwrap()),
    )
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
        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            "Search queries must be 3 characters or longer",
        )
        .await;
    }

    let new_msg = send_loading(
        ctx,
        &msg.channel_id,
        "MyAnimeList",
        "Loading actor information",
    )
    .await;
    let res = reqwest::get(&format!("{}search/person?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALPersonSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.len() == 0 {
        new_msg.delete(&ctx.http).await?;

        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            &format!("No results found for query: `{}`", params),
        )
        .await;
    } else {
        let mut embeds = Vec::new();

        for result in res.results {
            let mut embed = default_embed("MyAnimeList");

            embed
                .thumbnail(&result.image_url)
                .description(format!("[{}]({})", result.name, result.url));

            if result.alternate_names.len() > 0 {
                embed.field("Alternative Names", result.alternate_names.join("\n"), true);
            }

            embed.field("MAL ID", result.mal_id, true);

            embeds.push(embed);
        }

        new_msg.delete(&ctx.http).await?;

        return send_embed_paginator(ctx, msg, embeds).await;
    }
}

#[command]
#[description("Search for character")]
#[usage("<name>")]
#[example("Zero Two")]
async fn character(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();
    if params.len() < 3 {
        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            "Search queries must be 3 characters or longer",
        )
        .await;
    }

    let new_msg = send_loading(
        ctx,
        &msg.channel_id,
        "MyAnimeList",
        "Loading character information",
    )
    .await;
    let res = reqwest::get(&format!(
        "{}search/character?q={}",
        BASE_URL,
        encode(&params)
    ))
    .await?
    .text()
    .await?;

    let res: MALSearchResponse<MALCharacterSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.len() == 0 {
        new_msg.delete(&ctx.http).await?;

        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            &format!("No results found for query: `{}`", params),
        )
        .await;
    } else {
        let mut embeds = Vec::new();

        for result in res.results {
            let mut embed = default_embed("MyAnimeList");

            embed
                .thumbnail(&result.image_url)
                .description(format!("[{}]({})", result.name, result.url));

            if result.alternate_names.len() > 0 {
                embed.field("Alternative Names", result.alternate_names.join("\n"), true);
            }

            if result.anime.len() > 0 {
                let anime_list = result
                    .anime
                    .iter()
                    .map(|e| format!("[{}]({}) ({})", e.name, e.url, e.mal_id))
                    .collect::<Vec<String>>();
                embed.field("Anime", anime_list.join("\n"), true);
            }

            if result.manga.len() > 0 {
                let manga_list = result
                    .manga
                    .iter()
                    .map(|e| format!("[{}]({}) ({})", e.name, e.url, e.mal_id))
                    .collect::<Vec<String>>();
                embed.field("Manga", manga_list.join("\n"), true);
            }

            embed.field("MAL ID", result.mal_id, true);

            embeds.push(embed);
        }

        new_msg.delete(&ctx.http).await?;

        return send_embed_paginator(ctx, msg, embeds).await;
    }
}

#[command]
#[description("Search for manga")]
#[usage("<name>")]
#[example("One Piece")]
async fn manga(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();
    if params.len() < 3 {
        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            "Search queries must be 3 characters or longer",
        )
        .await;
    }

    let new_msg = send_loading(
        ctx,
        &msg.channel_id,
        "MyAnimeList",
        "Loading manga information",
    )
    .await;
    let res = reqwest::get(&format!("{}search/manga?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALMangaSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.len() == 0 {
        new_msg.delete(&ctx.http).await?;

        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            &format!("No results found for query: `{}`", params),
        )
        .await;
    } else {
        let mut embeds = Vec::new();

        for result in res.results {
            let mut embed = default_embed("MyAnimeList");

            embed
                .thumbnail(&result.image_url)
                .description(format!(
                    "**[{}]({})**\n{}",
                    result.title, result.url, result.synopsis
                ))
                .field("Volumes", result.chapters, true)
                .field("Chapters", result.chapters, true)
                .field("Score", result.score, true)
                .field("Members", result.members, true)
                .field("Start Date", &result.start_date, true)
                .field("End Date", &result.end_date, true)
                .field("MAL ID", result.mal_id, true);

            embeds.push(embed);
        }

        new_msg.delete(&ctx.http).await?;

        return send_embed_paginator(ctx, msg, embeds).await;
    }
}

#[command]
#[description("Search for anime")]
#[usage("<name>")]
#[example("Shingeki no Kyojin")]
async fn anime(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let params = args.rest();
    if params.len() < 3 {
        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            "Search queries must be 3 characters or longer",
        )
        .await;
    }

    let new_msg = send_loading(
        ctx,
        &msg.channel_id,
        "MyAnimeList",
        "Loading anime information",
    )
    .await;

    let res = reqwest::get(&format!("{}search/anime?q={}", BASE_URL, encode(&params)))
        .await?
        .text()
        .await?;

    let res: MALSearchResponse<MALAnimeSearchResult> = serde_json::from_str(&res).unwrap();

    if res.results.len() == 0 {
        new_msg.delete(&ctx.http).await?;

        return say_error(
            ctx,
            &msg.channel_id,
            "MyAnimeList",
            &format!("No results found for query: `{}`", params),
        )
        .await;
    } else {
        let mut embeds = Vec::new();

        for result in res.results {
            let mut embed = default_embed("MyAnimeList");

            embed
                .thumbnail(&result.image_url)
                .description(format!(
                    "**[{}]({})**\n{}",
                    result.title, result.url, result.synopsis
                ))
                .field("Episodes", result.episodes, true)
                .field("Score", result.score, true)
                .field("Members", result.members, true)
                .field("Start Date", &result.start_date, true)
                .field("End Date", &result.end_date, true)
                .field("Rated", &result.rated, true)
                .field("MAL ID", result.mal_id, true);

            embeds.push(embed);
        }

        new_msg.delete(&ctx.http).await?;

        return send_embed_paginator(ctx, msg, embeds).await;
    }
}
