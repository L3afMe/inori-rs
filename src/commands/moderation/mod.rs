mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(purgechat)]
#[description("**Moderation**")]
struct Moderation;
