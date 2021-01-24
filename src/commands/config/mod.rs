mod autodelete;
mod general;
mod giveaway;
mod giveaway_blacklist;
mod giveaway_whitelist;
mod slotbot;
mod slotbot_blacklist;
mod slotbot_whitelist;

use autodelete::*;
use general::*;
use giveaway::*;
use serenity::framework::standard::macros::group;
use slotbot::*;

#[group]
#[commands(autodelete, embedmode, giveaway, nsfwfilter, prefix, slotbot)]
#[description("**Config**")]
struct Config;
