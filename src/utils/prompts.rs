use crate::structures::errors::*;

use once_cell::sync::Lazy;
use serenity::{
    builder::CreateMessage,
    framework::standard::CommandResult,
    model::{
        channel::{Message, ReactionType},
        id::{ChannelId, GuildId, MessageId},
        prelude::User,
    },
    prelude::*,
};
use serenity_utils::prompt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub static USAGE_DESCRIPTION: Lazy<String> = Lazy::new(|| String::from("ABCD"));

/// Builds rich message from user inputs
pub async fn get_rich_message<'a>(
    ctx: &'a Context,
    channel_id: ChannelId,
    user: &'a User,
) -> Result<CreateMessage<'a>, BotstoneError> {
    let mut created_message = CreateMessage::default();

    let prompt_message = channel_id
        .send_message(ctx, |m| m.content("What should the content of the message be?"))
        .await?;
    match prompt_for_message_part(ctx, &prompt_message, user, 300.0).await? {
        PromptResult::Message(msg) => {
            created_message.content(msg);
            ()
        }
        _ => (),
    }

    Ok(created_message)
}

#[derive(EnumIter, Debug, PartialEq)]
pub enum PromptResult {
    Accept,
    Cancel,
    Preview,
    Skip,
    TimedOut,
    Message(String),
}

impl PromptResult {
    pub fn emoji_representation(&self) -> Option<ReactionType> {
        match self {
            Self::Accept => Some('\u{2705}'),   // :white_check_mark:
            Self::Cancel => Some('\u{274c}'),   // :x:
            Self::Preview => Some('\u{1f441}'), // :eye:
            Self::Skip => Some('\u{23e9}'),     // :fast_forward:
            Self::TimedOut => None,
            Self::Message(_) => None,
        }
        .map(|v| ReactionType::from(v))
    }

    pub fn all_representations() -> Vec<ReactionType> {
        Self::iter().filter_map(|x| x.emoji_representation()).collect()
    }

    pub fn from_emoji(emoji: ReactionType) -> Option<Self> {
        Self::iter()
            .filter(|x| matches!(x.emoji_representation(), Some(e) if e == emoji))
            .next()
    }
}

/// Await for reaction or user input, until timeout
pub async fn prompt_for_message_part(
    ctx: &Context,
    prompt_message: &Message,
    user: &User,
    timeout: f32,
) -> Result<PromptResult, BotstoneError> {
    let emojis = PromptResult::all_representations();
    loop {
        tokio::select! {
            opt_content = prompt::message_prompt_content(ctx, &prompt_message, &user, timeout) => {
                let result = match opt_content {
                    Some(response) => PromptResult::Message(response),
                    None => PromptResult::TimedOut,
                };
                return Ok(result)
            }
            res = prompt::reaction_prompt(ctx, &prompt_message, &user, &emojis, timeout) => {
                let result = match res {
                    Ok((_, reaction)) => PromptResult::from_emoji(reaction).ok_or(BotstoneError::ImpossibleError(Box::new(BotstoneError::Other("Unknown reaction!".into())))),
                    Err(serenity_utils::Error::TimeoutError) => Ok(PromptResult::TimedOut),
                    Err(serenity_utils::Error::SerenityError(e)) => Err(BotstoneError::SerenityError(e)),
                    Err(e) => Err(BotstoneError::ImpossibleError(Box::new(e))),
                };
                return result
            }
        }
    }
}
