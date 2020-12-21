use crate::{parsers::message::Message, structures::errors::*, utils::prompts};
use serenity::{
    builder::CreateMessage,
    framework::standard::{CommandResult, Args},
    model::id::{ChannelId, GuildId, MessageId},
    prelude::*,
    model::prelude::*,
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


pub async fn get_rich_from_args_or_prompt<'a>(ctx: &'a Context, channel: ChannelId, author: &'a User, args: &'a Args) -> Result<Option<CreateMessage<'a>>, BotstoneError> {
    let rich_message = if args.is_empty() {
        match prompts::get_rich_message(ctx, channel, author).await? {
            Some(rich_message) => rich_message,
            None => return Ok(None),
        }
    } else {
        let content = args.rest();
        if content.starts_with("{") {
            // Assume this is special content, which needs to be parsed
            deserialize_rich_message(content)?
        } else {
            let mut message = CreateMessage::default();
            message.content(content).to_owned()
        }
    };
    return Ok(Some(rich_message))
}
