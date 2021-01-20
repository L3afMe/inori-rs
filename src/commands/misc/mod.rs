mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(about, avatar, ratelimits, poll, hypesquad)]
#[description("**Miscellaneous**")]
struct Miscellaneous;
