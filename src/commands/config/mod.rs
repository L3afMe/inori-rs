mod autodelete;
mod general;

use autodelete::*;
use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(autodelete, embedmode, giveaway, nsfwfilter, prefix)]
#[description("**Config**")]
struct Config;
