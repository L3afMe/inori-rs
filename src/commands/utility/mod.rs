mod automsg;
mod emotestealer;
mod general;
pub mod purge;
mod tags;

// use automsg::*;
use emotestealer::*;
use general::*;
use purge::*;
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
