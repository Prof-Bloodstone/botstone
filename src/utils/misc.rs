use crate::{parsers::message::Message, structures::errors::*};
use serenity::{
    builder::CreateMessage,
    framework::standard::CommandResult,
    model::id::{ChannelId, GuildId, MessageId},
    prelude::*,
};
use std::convert::TryFrom;

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
    let cm = deserialize_rich_message(serialized_message)?;
    channel_id
        .send_message(ctx, |m| {
            m.0 = cm.0;
            m
        })
        .await?;
    Ok(())
}

pub fn deserialize_rich_message(serialized_message: &str) -> Result<CreateMessage<'_>, ParseError> {
    let deserialized_message =
        json5::from_str::<Message>(serialized_message).map_err(|err| ParseError::InvalidJson(err))?;
    CreateMessage::try_from(deserialized_message)
}
