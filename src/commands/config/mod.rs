mod autodelete;
mod general;
mod slotbot;
mod slotbot_blacklist;
mod slotbot_whitelist;

use autodelete::*;
use general::*;
use serenity::framework::standard::macros::group;
use slotbot::*;

#[group]
#[commands(autodelete, embedmode, giveaway, nsfwfilter, prefix, slotbot)]
#[description("**Config**")]
struct Config;
