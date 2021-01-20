mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(changemymind, clyde, cutie, kannagen, lolice, phcomment, trumptweet)]
#[description("**Image Gen**")]
struct ImageGen;
