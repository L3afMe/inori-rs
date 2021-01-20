mod automsg;
mod emotestealer;
mod general;
mod tags;

// use automsg::*;
use emotestealer::*;
use general::*;
use serenity::framework::standard::macros::group;
use tags::*;

#[group]
#[commands(
    base64,
    emotestealer,
    exchange,
    math,
    ping,
    purge,
    rustdoc,
    serverinfo,
    setup,
    tags,
    usages
)]
#[description("**Utilities**")]
struct Utility;
