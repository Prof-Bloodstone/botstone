#![deny(clippy::all)]
#![deny(unsafe_code)]
mod commands;
mod database;
mod parsers;
mod structures;
mod utils;
mod version_data;
mod event_handling;

use crate::{
    database::queries::GuildInfoTable,
    structures::{
        commands::*,
        context::{ConnectionPool, PublicData, ShardManagerContainer, VersionDataContainer},
    },
    version_data::VersionData,
    event_handling::{
        Handler, before, after, MY_HELP, dynamic_prefix
    },
};
use dotenv;
use serenity::{
    framework::StandardFramework,
    http::Http,
    prelude::*,
};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashSet, env, error::Error, sync::Arc};
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info, instrument, warn};


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
        .configure(|c| c.owners(owners).dynamic_prefix(dynamic_prefix).on_mention(Some(bot_id)))
        .before(before)
        .after(after)
        .help(&MY_HELP);
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
