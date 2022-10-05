use crate::{structures::context::ShardManagerContainer};
use anyhow::anyhow;
use chrono::Utc;
use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tracing::{debug};

// Based on implementation by @Flat at https://github.com/Flat/Lupusregina-/blob/0abda1835625f1e4748cc2a9e89fbaf938877990/src/commands/general.rs#L201
#[command]
#[description = "Responds with the current latency to Discord."]
async fn ping(context: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let mut msg = msg.channel_id.say(&context, "**Pong!**").await?;
    let finish = Utc::now();
    let lping = ((finish.timestamp() - now.timestamp()) * 1000)
        + (i64::from(finish.timestamp_subsec_millis()) - i64::from(now.timestamp_subsec_millis()));
    let shard_manager = context
        .data
        .read()
        .await
        .get::<ShardManagerContainer>()
        .ok_or_else(|| anyhow!("Failed to get ShardManagerContainer"))?
        .clone();
    let shard_latency = shard_manager
        .lock()
        .await
        .runners
        .lock()
        .await
        .get(&ShardId(context.shard_id))
        .ok_or_else(|| anyhow!("Failed to get Shard."))?
        .latency // TODO: Getting latency fails for a minute after boot - add retries?
        .ok_or_else(|| anyhow!("Failed to get latency from shard."))?
        .as_millis();
    let msg_content = msg.content.clone();
    debug!(
        "Responding with API latency {} and shard latency {}",
        lping, shard_latency
    );
    msg.edit(context, |m| {
        m.content(&format!(
            "{}\nRest API: {}ms\nShard Latency: {}ms",
            msg_content, lping, shard_latency
        ))
    })
    .await?;
    Ok(())
}
