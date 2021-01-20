use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize};
use serenity::{client::bridge::gateway::ShardManager, prelude::TypeMapKey};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]

pub struct NekosLifeResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]

pub struct NekoBotResponse {
    pub message: String,
}

pub struct Img {
    pub website_type: u8,
    pub link:         String,
    pub level:        u8,
}

impl Img {
    pub fn new(website_type: u8, link: &str, level: u8) -> Img {
        Img {
            website_type,
            link: link.to_string(),
            level,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct Rule34Post {
    pub file_url: String,
    pub tags:     String,
    pub md5:      String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct Rule34Posts {
    #[serde(rename = "post")]
    pub posts: Option<Vec<Rule34Post>>,
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct FrankFurterResponse {
    pub amount: f64,
    pub base:   String,
    pub date:   String,
    pub rates:  HashMap<String, f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALMangaSearchResult {
    pub mal_id:     u64,
    pub url:        String,
    pub image_url:  String,
    pub title:      String,
    pub publishing: bool,
    pub synopsis:   String,
    pub chapters:   u64,
    pub volumes:    u64,
    pub score:      f64,
    #[serde(deserialize_with = "parse_start_date")]
    pub start_date: String,
    #[serde(deserialize_with = "parse_end_date")]
    pub end_date:   String,
    pub members:    u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALAnimeSearchResult {
    pub mal_id:     u64,
    pub url:        String,
    pub image_url:  String,
    pub title:      String,
    pub airing:     bool,
    pub synopsis:   String,
    pub episodes:   u64,
    pub score:      f64,
    #[serde(deserialize_with = "parse_start_date")]
    pub start_date: String,
    #[serde(deserialize_with = "parse_end_date")]
    pub end_date:   String,
    pub members:    u64,
    #[serde(deserialize_with = "parse_rated")]
    pub rated:      String,
}

fn parse_start_date<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>, {
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or("NYA".to_string()))
}

fn parse_end_date<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>, {
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or("TBD".to_string()))
}

fn parse_rated<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>, {
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or("Unrated".to_string()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALPreview {
    pub mal_id: u64,
    pub name:   String,
    pub url:    String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALCharacterSearchResult {
    pub mal_id:          u64,
    pub url:             String,
    pub image_url:       String,
    pub name:            String,
    pub alternate_names: Vec<String>,
    pub anime:           Vec<MALPreview>,
    pub manga:           Vec<MALPreview>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALPersonSearchResult {
    pub mal_id:          u64,
    pub url:             String,
    pub image_url:       String,
    pub name:            String,
    pub alternate_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct MALSearchResponse<T> {
    pub last_page: u64,
    pub results:   Vec<T>,
}
