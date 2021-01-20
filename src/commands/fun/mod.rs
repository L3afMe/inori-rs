mod general;
mod mal;
mod pfpswitcher;
mod quote;

use general::*;
use mal::*;
use pfpswitcher::*;
use quote::*;
use serenity::framework::standard::macros::group;

#[group]
#[commands(
    balance,
    compatibility,
    dick,
    myanimelist,
    pfpswitcher,
    quote,
    sexuality,
    urbandictionary
)]
#[description("**Fun**")]

struct Fun;
