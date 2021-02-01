use std::collections::HashSet;

use serenity::{
    framework::standard::{help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions},
    model::{channel::Message, id::UserId},
    prelude::Context,
};

#[help]
#[individual_command_tip(
    "**Help**\nArgument keys\n`<>` - Required\n`[]` - Options\nTo get help for a specific command, subcommand or \
     group, use `help <command>`."
)]
#[suggestion_text("**Error** Unable to find command. Similar commands: `{}`")]
#[no_help_available_text("**Error** Unable to find command")]
#[command_not_found_text("**Error** Unable to find command")]
#[dm_only_text("DMs")]
#[guild_only_text("Servers")]
#[dm_and_guild_text("DMs and Servers")]
#[max_levenshtein_distance(2)]
#[indention_prefix("-")]
#[lacking_permissions("Strike")]
#[lacking_role("Strike")]
#[wrong_channel("Strike")]
#[strikethrough_commands_tip_in_dm("Commands with a ~~`strikethrough`~~ are required to be executed in a guild.")]
#[strikethrough_commands_tip_in_guild(
    "Commands with a ~~`strikethrough`~~ require certain lacking permissions or are required to be executed in DMs."
)]
#[embed_error_colour(MEIBE_PINK)]
#[embed_success_colour(FABLED_PINK)]
pub async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;

    Ok(())
}
