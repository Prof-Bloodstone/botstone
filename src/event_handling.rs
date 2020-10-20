use crate::{
    database::queries::{CustomCommands, GuildInfoTable},
    utils::misc::send_rich_serialized_message,
};
use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{help, hook},
        Args,
        CommandError,
        CommandGroup,
        CommandResult,
        HelpOptions,
    },
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::Ready,
        guild::{Guild, GuildUnavailable},
        id::{GuildId, UserId},
    },
    prelude::*,
};
use std::collections::HashSet;
use tracing::{debug, error, info, instrument};

#[derive(Debug)]
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /*
    async fn message(&self, ctx: Context, msg: Message) {
       if !msg.author.bot {
           info!("Reacting to: {:?}", msg.content);
           match msg.react(&ctx, '👌').await {
               Ok(reaction) => info!("Successfully posted reaction {:?}", reaction.emoji.as_data()),
               Err(e) => error!("Emoji error {:?}", e)
           }
       }
    }*/

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    #[instrument(skip(ctx))]
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let guild_info = {
            let data = ctx.data.read().await;
            let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
            guild_info
        };

        let current_guilds = guilds.iter().cloned().collect::<HashSet<GuildId>>();
        let existing_guilds = guild_info.get_guilds().await;
        let new_guilds = current_guilds.difference(&existing_guilds);
        let deleted_guilds = existing_guilds.difference(&current_guilds);
        for guild_id in new_guilds {
            info!("Detected new guild while the bot was down: {}", guild_id);
            match guild_info.add_guild(*guild_id).await {
                Ok(_) => {}
                Err(e) => error!("Issue while adding new guild: {}", e),
            }
        }
        for guild_id in deleted_guilds {
            info!("Detected kicked from guild while the bot was down: {}", guild_id);
            match guild_info.remove_guild(*guild_id).await {
                Ok(_) => {}
                Err(e) => error!("Issue while deleting guild: {}", e),
            }
        }
    }

    #[instrument(skip(ctx))]
    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _full: Option<Guild>) {
        let guild_info = {
            let data = ctx.data.read().await;
            let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
            guild_info
        };
        if let Err(e) = guild_info.remove_guild(incomplete.id).await {
            error!("Error deleting guild: {:?}", e);
        }
    }

    #[instrument(skip(ctx))]
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if !is_new {
            return;
        }
        let guild_info = {
            let data = ctx.data.read().await;
            let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
            guild_info
        };
        if let Err(e) = guild_info.add_guild(guild.id).await {
            error!("Error adding guild: {:?}", e);
        }
    }
}

/*
 * The heart of custom prefixes
 * If the guild has a prefix in the DB, use that prefix
 * Otherwise, use the default prefix
 */
#[hook]
pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let guild_info = {
        let data = ctx.data.read().await;
        let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
        guild_info
    };
    let guild_id = msg.guild_id?;

    guild_info.get_prefix(guild_id).await
}

#[hook]
#[instrument] // Not supported on Commands, so need to use it here.
pub async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    debug!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
}

#[hook]
#[instrument(skip(ctx))]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        error!(
            "Command {:?} triggered by {}: {:?}",
            cmd_name,
            msg.author.tag(),
            why
        );
        let _ = msg.react(ctx, '\u{274C}').await;
    }
}

#[hook]
#[instrument(skip(ctx))]
pub async fn unrecognised_command(ctx: &Context, msg: &Message, command_name: &str) {
    if let Some(guild_id) = msg.guild_id {
        let custom_commands = {
            let data = ctx.data.read().await;
            data.get::<CustomCommands>().cloned()
        };
        match custom_commands {
            None => error!("Unable to get custom commands!"),
            Some(cc) => match cc.get_command(guild_id, command_name.to_string()).await {
                Err(e) => error!("Error getting custom command: {:?}", e),
                Ok(None) => {}
                Ok(Some(content)) => {
                    if content.starts_with("{") {
                        if let Err(e) = send_rich_serialized_message(ctx, msg.channel_id, &*content).await {
                            error!(
                                "Unable to send rich message, content: {:?}, error: {:?}",
                                content, e
                            );
                        }
                    } else {
                        if let Err(e) = msg.channel_id.say(ctx, content.clone()).await {
                            error!(
                                "Unable to send simple custom response, content: {:?}, error: {:?}",
                                content, e
                            );
                        }
                    }
                }
            },
        }
    }
}

#[help]
#[lacking_role(strike)]
#[lacking_permissions(strike)]
#[lacking_ownership(hide)]
pub async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
