use crate::parsers::message::Message;
use serenity::{
    builder::CreateMessage,
    framework::standard::CommandResult,
    model::id::{ChannelId, GuildId, MessageId},
    prelude::*,
};
use std::convert::TryFrom;
use tracing::warn;

pub fn get_message_url(guild_id: GuildId, channel_id: ChannelId, message_id: MessageId) -> String {
    format!(
        "https://discordapp.com/channels/{}/{}/{}",
        guild_id.0, channel_id.0, message_id.0
    )
}

pub async fn send_rich_serialized_message(
    ctx: &Context,
    channel_id: ChannelId,
    serialized_message: &str,
) -> CommandResult {
    let deserialize_result = serde_json::from_str::<Message>(serialized_message);
    return match deserialize_result {
        Err(e) => {
            let error_msg = format!("Unable to deserialize rich response. The error was: {:#?}", e);
            warn!("{}. The content was `{:?}`", error_msg, serialized_message);
            channel_id.say(ctx, error_msg).await?;
            Ok(())
        }
        Ok(deserialized_msg) => {
            let result = CreateMessage::try_from(deserialized_msg);
            match result {
                Ok(cm) => {
                    channel_id
                        .send_message(ctx, |m| {
                            m.0 = cm.0;
                            m
                        })
                        .await?;
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Unable to create message. The error was: {:#?}", e);
                    warn!("{}. The content was `{:?}`", error_msg, serialized_message);
                    channel_id.say(ctx, error_msg).await?;
                    Ok(())
                }
            }
        }
    };
}
