mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(about, color, avatar, ratelimits, poll, hypesquad)]
#[description("**Miscellaneous**")]
struct Miscellaneous;
