#![deny(clippy::all)]
#![deny(unsafe_code)]
mod commands;
mod database;
mod parsers;
mod structures;
mod utils;
mod version_data;

use crate::{
    database::queries::GuildInfoTable,
    structures::{
        commands::*,
        context::{ConnectionPool, PublicData, ShardManagerContainer, VersionDataContainer},
    },
    version_data::VersionData,
};
use dotenv;
use serenity::{
    async_trait,
    framework::{
        standard::{macros::hook, CommandError},
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::GuildId},
    prelude::*,
};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashSet, env, error::Error, sync::Arc};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info, instrument, warn};

struct Handler;

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
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
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
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    debug!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
}

#[hook]
#[instrument]
async fn after(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        info!("Error in {}: {:?}", cmd_name, why);
    }
}

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    // This will load the environment variables located at `./.env`, relative to CWD
    if let Err(_) = dotenv::dotenv() {
        warn!("Failed to load .env file!")
    }

    // Initialize the logger to use environment variables.
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    tracing_subscriber::fmt::init();

    info!("Booting up...");
    let version_string = include_str!(concat!(env!("OUT_DIR"), "/version.json"));
    let build_data =
        serde_json::from_str::<VersionData>(version_string).expect("Unable to retrieve VersionData");
    info!("Running {}", build_data);
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let prefix = env::var("COMMAND_PREFIX").unwrap_or(String::from("."));
    let db_url = env::var("DATABASE_URL").expect("Expected database url in the environment");
    debug!("Will connect to database: {}", db_url);

    let command_groups = ALL_GROUP.options.sub_groups;

    let hardcoded_commands = command_groups
        .iter()
        .flat_map(|x| {
            x.options
                .commands
                .iter()
                .flat_map(|i| i.options.names.iter().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<String>>();

    let pool = PgPoolOptions::new().max_connections(8).connect(&db_url).await?;

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let mut framework = StandardFramework::new()
        .configure(|c| c.owners(owners).dynamic_prefix(dynamic_prefix))
        .before(before)
        .after(after);
    for group in command_groups {
        framework = framework.group(group);
    }

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let guild_info = GuildInfoTable::new(prefix.clone(), &pool).await?;
    {
        let mut data = client.data.write().await;
        // Init shard manager
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<PublicData>(Arc::new(PublicData {
            default_prefix: prefix,
            hardcoded_commands,
            bot_id,
        }));
        data.insert::<ConnectionPool>(Arc::new(pool.clone()));
        data.insert::<VersionDataContainer>(Arc::new(build_data));
        data.insert::<GuildInfoTable>(Arc::new(guild_info))
    }

    // Listen to interrupts
    let signals_to_handle = vec![
        SignalKind::hangup(),
        SignalKind::interrupt(),
        SignalKind::terminate(),
    ];
    for kind in signals_to_handle {
        let mut stream = signal(kind).unwrap();
        let shard_manager = client.shard_manager.clone();
        tokio::spawn(async move {
            stream.recv().await;
            info!("Signal received - shutting down!");
            shard_manager.lock().await.shutdown_all().await;
        });
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
    Ok(())
}
