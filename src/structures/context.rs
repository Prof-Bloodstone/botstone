use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::UserId,
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::version_data::VersionData;

// All command context data structures
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = Arc<PgPool>;
}

pub struct CommandNameMap;

impl TypeMapKey for CommandNameMap {
    type Value = Arc<Vec<String>>;
}

pub struct PublicData {
    pub default_prefix: String,
    pub hardcoded_commands: Vec<String>,
}

impl TypeMapKey for PublicData {
    type Value = Arc<Self>;
}

pub struct BotId;

impl TypeMapKey for BotId {
    type Value = Arc<UserId>;
}

pub struct VersionDataContainer;

impl TypeMapKey for VersionDataContainer {
    type Value = Arc<VersionData>;
}