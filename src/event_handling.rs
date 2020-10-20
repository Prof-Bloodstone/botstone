use crate::database::queries::GuildInfoTable;
use serenity::{
    async_trait,
    framework::{
        standard::{help_commands, macros::{hook, help}, CommandError, Args, CommandGroup, HelpOptions, CommandResult},
    },
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::{GuildId, UserId}},
    prelude::*,
};
use tracing::{debug, error, info};
use std::collections::HashSet;

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

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let guild_info = {
            let data = ctx.data.read().await;
            let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
            guild_info
        };

        for guild_id in guilds {
            let prefix = guild_info.get_prefix(guild_id).await;
            if prefix.is_none() {
                info!("Detected new guild while the bot was down: {}", guild_id);
                match guild_info.add_guild(guild_id).await {
                    Ok(_) => {}
                    Err(e) => error!("Issue while adding new guild: {}", e),
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
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
    let guild_id = msg.guild_id.unwrap();

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
        msg.react(ctx, '\u{274C}').await.map_or_else(|_| (), |_| ());
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
