use crate::{parsers::message::Message, structures::errors::*, utils::prompts};

use serenity::{
    builder::CreateMessage,
    framework::standard::{Args, CommandResult},
    model::{
        id::{ChannelId, GuildId, MessageId},
        prelude::*,
    },
    prelude::*,
    utils::parse_role,
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

pub async fn get_rich_from_args_or_prompt<'a>(
    ctx: &'a Context,
    channel: ChannelId,
    author: &'a User,
    args: &'a Args,
) -> Result<Option<CreateMessage<'a>>, BotstoneError> {
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
    return Ok(Some(rich_message));
}

pub async fn role_from_name_or_mention(
    ctx: &Context,
    guild_id: &GuildId,
    role_string: String,
) -> Result<RoleId, BotstoneError> {
    let role_id = if role_string.starts_with('<') {
        // Assume it's mention
        parse_role(role_string)
            .ok_or(CommandError::UserError("Invalid role mention".to_string()))?
            .into()
    } else {
        guild_id
            .to_partial_guild(ctx)
            .await
            .map_err(|e| CommandError::UserDiscordError("Error getting partial guild".to_string(), e))?
            .role_by_name(role_string.as_str())
            .ok_or(CommandError::UserError("Invalid role name".to_string()))?
            .id
    };
    return Ok(role_id);
}
