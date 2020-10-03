use crate::version_data::VersionData;
use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;
use std::sync::Arc;

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

pub struct VersionDataContainer;

impl TypeMapKey for VersionDataContainer {
    type Value = Arc<VersionData>;
}

