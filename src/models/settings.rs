use std::{collections::HashMap, sync::Arc};

use serde_derive::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct PfpSwitcher {
    pub delay:   u32,
    pub mode:    u8,
    pub enabled: bool,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GiveawayConfig {
    pub enabled: bool,
    pub min_delay: u64,
    pub max_delay: u64,
    pub mode: u8,
    pub whitelisted_guilds: Vec<u64>,
    pub blacklisted_guilds: Vec<u64>,
    pub whitelisted_words: Vec<String>,
    pub blacklisted_words: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AutoDeleteConfig {
    pub enabled: bool,
    pub delay:   u64,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SlotBotConfig {
    pub enabled: bool,
    pub dynamic_prefix: bool,
    pub mode: u8,
    pub whitelisted_guilds: Vec<u64>,
    pub blacklisted_guilds: Vec<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub user_token: String,
    pub command_prefix: String,
    pub global_nsfw_level: u8,
    pub is_male: bool,
    pub embed_mode: u8,
    pub emoteserver: u64,
    pub nitrosniper: bool,
    pub pfp_switcher: PfpSwitcher,
    pub giveaway: GiveawayConfig,
    pub autodelete: AutoDeleteConfig,
    pub slotbot: SlotBotConfig,
    pub tags: HashMap<String, String>,
    pub sb_emotes: HashMap<String, u64>,
}

impl TypeMapKey for Settings {
    type Value = Arc<Mutex<Settings>>;
}
