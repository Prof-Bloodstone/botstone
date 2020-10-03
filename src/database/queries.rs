use serenity::model::id::GuildId;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use serenity::prelude::{TypeMapKey, RwLock};
use tracing::instrument;
use std::fmt;

#[derive(Clone, Debug)]
pub struct GuildInfoStruct {
    guild_id: i64,
    prefix: String,
}

impl fmt::Display for GuildInfoStruct{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GuildInfoStruct{{guild_id: {}, prefix: '{}'}}", self.guild_id, self.prefix)
    }
}

pub type GuildInfoMap = HashMap<GuildId, GuildInfoStruct>;

#[derive(Debug)]
pub struct GuildInfoTable {
    pool: PgPool,
    info: RwLock<GuildInfoMap>,
}

impl GuildInfoTable {
    pub async fn new(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let map = Self::get_all_guild_info(pool).await?;
        Ok(Self {
            pool: pool.clone(),
            info: RwLock::new(map),
        })
    }

    #[instrument]
    async fn get_all_guild_info(pool: &PgPool) -> Result<GuildInfoMap, sqlx::Error> {
        let data = sqlx::query_as!(GuildInfoStruct, "SELECT * FROM guild_info")
            .fetch_all(pool)
            .await?;
        let map: GuildInfoMap = data.iter().map(|x| (GuildId(x.guild_id as u64), x.clone())).collect();
        Ok(map)
    }

    #[instrument]
    pub async fn get_prefix(&self, guild_id: GuildId) -> Option<String> {
        let guild_info_map = self.info.read().await;
        guild_info_map.get(&guild_id).map(|gis| gis.prefix.clone())
    }

    #[instrument]
    pub async fn set_prefix(&self, guild_id: GuildId, prefix: String) -> Result<(), sqlx::Error> {

        let data = sqlx::query_as!(
            GuildInfoStruct,
            "UPDATE guild_info SET prefix = $1 WHERE guild_id = $2 RETURNING *",
            prefix,
            i64::from(guild_id)
        )
        .fetch_optional(&self.pool)
        .await?;

        match data {
            None => {self.write_info(guild_id, prefix).await?;},
            Some(info) => {
                let mut writer = self.info.write().await;
                writer.insert(guild_id, info);
            }
        };

        Ok(())
    }

    #[instrument]
    pub async fn write_info(&self, guild_id: GuildId, prefix: String) -> Result<GuildInfoStruct, sqlx::Error> {
        let data = sqlx::query_as!(
            GuildInfoStruct,
            "INSERT INTO guild_info (guild_id, prefix) VALUES ($1, $2) RETURNING *",
            i64::from(guild_id),
            prefix
        )
            .fetch_optional(&self.pool)
            .await?
            .expect("INSERT to guild_info didn't return anything!");

        let mut writer = self.info.write().await;
        writer.insert(guild_id, data.clone());

        Ok(data)
    }

    #[instrument]
    pub async fn add_guild(&self, guild_id: GuildId) -> Result<GuildInfoStruct, sqlx::Error> {
        self.write_info(guild_id, ".".to_string()).await
    }
}

impl TypeMapKey for GuildInfoTable { type Value = Arc<Self>; }