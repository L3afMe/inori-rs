mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(about, avatar, color, hypesquad, poll, ratelimits, spammer)]
#[description("**Miscellaneous**")]
struct Miscellaneous;
