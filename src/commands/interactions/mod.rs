mod general;

use general::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(anal, blowjob, cuddle, feed, hug, kiss, pat, poke, slap, spank, tickle)]
#[description = "**Interactions**"]

struct Interactions;
