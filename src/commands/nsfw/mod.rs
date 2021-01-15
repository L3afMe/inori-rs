mod general;

use general::*;

use serenity::framework::standard::macros::group;

#[group]
#[commands(image, imagebomb, rule34)]
#[description("**NSFW**")]
struct NSFW;
