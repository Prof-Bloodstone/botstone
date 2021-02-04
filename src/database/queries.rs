use crate::structures::errors::DatabaseError;
use serenity::{
    model::id::{ChannelId, GuildId, MessageId, RoleId},
    prelude::{RwLock, TypeMapKey},
};
use sqlx::{Done, PgPool};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct GuildInfoStruct {
    guild_id: i64,
    prefix: String,
}

impl fmt::Display for GuildInfoStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GuildInfoStruct{{guild_id: {}, prefix: '{}'}}",
            self.guild_id, self.prefix
        )
    }
}

pub type GuildInfoMap = HashMap<GuildId, GuildInfoStruct>;

#[derive(Debug)]
pub struct GuildInfoTable {
    default_prefix: String,
    pool: PgPool,
    info: RwLock<GuildInfoMap>,
}

impl GuildInfoTable {
    pub async fn new(default_prefix: String, pool: PgPool) -> Result<Self, sqlx::Error> {
        let map = Self::get_all_guild_info(&pool).await?;
        Ok(Self {
            default_prefix,
            pool,
            info: RwLock::new(map),
        })
    }

    #[instrument]
    async fn get_all_guild_info(pool: &PgPool) -> Result<GuildInfoMap, sqlx::Error> {
        let data = sqlx::query_as!(GuildInfoStruct, "SELECT * FROM guild_info")
            .fetch_all(pool)
            .await?;
        let map: GuildInfoMap = data
            .iter()
            .map(|x| (GuildId(x.guild_id as u64), x.clone()))
            .collect();
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
            None => {
                self.write_info(guild_id, &prefix).await?;
            }
            Some(info) => {
                let mut writer = self.info.write().await;
                writer.insert(guild_id, info);
            }
        };

        Ok(())
    }

    #[instrument]
    pub async fn write_info(
        &self,
        guild_id: GuildId,
        prefix: &String,
    ) -> Result<GuildInfoStruct, sqlx::Error> {
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
        self.write_info(guild_id, &self.default_prefix).await
    }

    pub async fn get_guilds(&self) -> HashSet<GuildId> {
        self.info.read().await.keys().cloned().collect()
    }

    #[instrument]
    pub async fn remove_guild(&self, guild_id: GuildId) -> Result<(), DatabaseError> {
        let result = sqlx::query!("DELETE FROM guild_info WHERE guild_id = $1", i64::from(guild_id))
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatabaseError::NothingDeleted);
        }
        let mut writer = self.info.write().await;
        writer.remove(&guild_id);
        Ok(())
    }
}

impl TypeMapKey for GuildInfoTable {
    type Value = Arc<Self>;
}

#[derive(Debug)]
pub struct CustomCommands {
    pool: PgPool,
}

impl CustomCommands {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument]
    pub async fn set_command(
        &self,
        guild_id: GuildId,
        name: String,
        content: String,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO commands (guild_id, name, content) VALUES ($1, $2, $3)
            ON CONFLICT (guild_id, name) DO UPDATE SET content = EXCLUDED.content",
            i64::from(guild_id),
            name,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_command(
        &self,
        guild_id: GuildId,
        name: String,
    ) -> Result<Option<String>, DatabaseError> {
        let returned = sqlx::query!(
            "SELECT content FROM commands WHERE guild_id = $1 AND name = $2",
            i64::from(guild_id),
            name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(returned.map(|value| value.content))
    }

    #[instrument]
    pub async fn get_command_names(&self, guild_id: GuildId) -> Result<Vec<String>, DatabaseError> {
        let names = sqlx::query!(
            "SELECT name FROM commands WHERE guild_id = $1",
            i64::from(guild_id)
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|value| value.name)
        .collect::<Vec<String>>();
        Ok(names)
    }

    #[instrument]
    pub async fn delete_command(&self, guild_id: GuildId, name: String) -> Result<(), DatabaseError> {
        sqlx::query!(
            "DELETE FROM commands WHERE guild_id = $1 AND name = $2",
            i64::from(guild_id),
            name
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl TypeMapKey for CustomCommands {
    type Value = Arc<Self>;
}

#[derive(Debug)]
pub struct ReactionRoles {
    pool: PgPool,
}

impl ReactionRoles {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument]
    pub async fn set_react_role(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        message_id: MessageId,
        role_id: RoleId,
        reaction: String,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO react_roles (guild_id, channel_id, message_id, role_id, reaction_emoji)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (guild_id, channel_id, message_id, reaction_emoji)
            DO UPDATE SET role_id = EXCLUDED.role_id",
            i64::from(guild_id),
            i64::from(channel_id),
            i64::from(message_id),
            i64::from(role_id),
            reaction
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_react_role(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Option<RoleId>, DatabaseError> {
        let returned = sqlx::query!(
            "SELECT role_id FROM react_roles
            WHERE guild_id = $1
            AND channel_id = $2
            AND message_id = $3
            AND reaction_emoji = $4",
            i64::from(guild_id),
            i64::from(channel_id),
            i64::from(message_id),
            reaction
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(returned.map(|value| RoleId::from(value.role_id as u64)))
    }

    #[instrument]
    pub async fn delete_react_role(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<u64, DatabaseError> {
        sqlx::query!(
            "DELETE FROM react_roles
            WHERE guild_id = $1
            AND channel_id = $2
            AND message_id = $3
            AND reaction_emoji = $4",
            i64::from(guild_id),
            i64::from(channel_id),
            i64::from(message_id),
            reaction
        )
        .execute(&self.pool)
        .await
        .map(|done| done.rows_affected())
        .map_err(|err| err.into())
    }

    #[instrument]
    pub async fn delete_react_roles(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<u64, DatabaseError> {
        sqlx::query!(
            "DELETE FROM react_roles
            WHERE guild_id = $1
            AND channel_id = $2
            AND message_id = $3",
            i64::from(guild_id),
            i64::from(channel_id),
            i64::from(message_id)
        )
        .execute(&self.pool)
        .await
        .map(|done| done.rows_affected())
        .map_err(|err| err.into())
    }
}

impl TypeMapKey for ReactionRoles {
    type Value = Arc<Self>;
}
