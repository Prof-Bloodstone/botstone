mod commands;
mod parsers;
mod structures;
mod utils;
mod version_data;

use tracing::{error, info, warn, debug, instrument};
use serenity::{
    async_trait,
    framework::{standard::macros::hook, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};

use crate::structures::context::{ConnectionPool, ShardManagerContainer, VersionDataContainer};
use crate::structures::{commands::*, context::PublicData};
use crate::version_data::VersionData;
use dotenv;
use serenity::model::channel::Message;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;
use tokio::signal::unix::{signal, SignalKind};

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

    async fn ready(&self, _: Context, ready: Ready) {
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
    let (pool, default_prefix) = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap().clone();
        let default_prefix = data.get::<PublicData>().unwrap().default_prefix.clone();
        (pool, default_prefix)
    };
    let guild_id = msg.guild_id.unwrap();

    let cur_prefix = commands::config::get_prefix(&*pool, guild_id, default_prefix)
        .await
        .unwrap();

    Some(cur_prefix)
}

#[hook]
#[instrument] // Not supported on Commands, so need to use it here.
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    debug!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
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

    let hardcoded_commands = ALL_GROUP
        .options
        .sub_groups
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
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).dynamic_prefix(dynamic_prefix))
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP)
        .group(&CONFIG_GROUP)
        .group(&SUPPORT_GROUP)
        .before(before);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        // Init shard manager
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<PublicData>(Arc::new(PublicData {
            default_prefix: prefix,
            hardcoded_commands,
        }));
        data.insert::<ConnectionPool>(Arc::new(pool));
        data.insert::<VersionDataContainer>(Arc::new(build_data));
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
