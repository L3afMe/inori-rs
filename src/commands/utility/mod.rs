mod automsg;
mod emotestealer;
mod general;
mod tags;

// use automsg::*;
use emotestealer::*;
use general::*;
use tags::*;

use serenity::framework::standard::macros::group;

#[group]
#[commands(
    avatar,
    emotestealer,
    exchange,
    hypesquad,
    math,
    ping,
    poll,
    purge,
    purgechat,
    ratelimits,
    rustdoc,
    serverinfo,
    setup,
    tags,
    usages
)]
#[description("**Utilities**")]
struct Utility;
