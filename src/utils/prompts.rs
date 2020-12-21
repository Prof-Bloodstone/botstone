use crate::structures::errors::*;
use crate::parsers::message::{Message as MessageBuilder, Embed as EmbedBuilder};
use once_cell::sync::Lazy;
use serenity::{
    builder::CreateMessage,
    model::{
        channel::{Message, ReactionType},
        id::ChannelId,
        prelude::User,
    },
    prelude::*,
};
use serenity_utils::prompt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use std::convert::TryFrom;

pub static PROMPT_USAGE_DESCRIPTION: Lazy<CreateMessage<'static>> = Lazy::new(|| {
    let mut msg = CreateMessage::default();
    msg.content(format!(
        concat!(
            "Accept and immediately finish by clicking {}\n",
            "Cancel message creation by pressing {}\n",
            "Skip current part of the prompt by using {}\n",
            "Preview changes so far with {}\n",
            "If you take too long, the process is cancelled automatically"
        ),
        PromptResult::Accept
            .emoji_representation()
            .unwrap_or("UNKNOWN".to_string()),
        PromptResult::Cancel
            .emoji_representation()
            .unwrap_or("UNKNOWN".to_string()),
        PromptResult::Skip
            .emoji_representation()
            .unwrap_or("UNKNOWN".to_string()),
        PromptResult::Preview
            .emoji_representation()
            .unwrap_or("UNKNOWN".to_string()),
    ));
    msg.clone()
});

pub static PROMPT_MESSAGE_CONTENT: Lazy<CreateMessage<'static>> = Lazy::new(|| {
    let mut msg = CreateMessage::default();
    msg.content("What should the content of the message be?");
    msg.clone()
});

pub static PROMPT_EMBED_DESCRIPTION: Lazy<CreateMessage<'static>> = Lazy::new(|| {
    let mut msg = CreateMessage::default();
    msg.content("What should the content of the embed be?");
    msg.clone()
});

/// Builds rich message from user inputs
pub async fn get_rich_message<'a>(
    ctx: &'a Context,
    channel_id: ChannelId,
    user: &'a User,
) -> Result<Option<CreateMessage<'a>>, BotstoneError> {
    let mut builder = MessageBuilder::default();

    channel_id
        .send_message(ctx, |m| {
            m.0 = PROMPT_USAGE_DESCRIPTION.clone().0;
            m
        })
        .await?;

    match prompt_for_message_part_previewed(
        ctx,
        &PROMPT_MESSAGE_CONTENT,
        channel_id,
        &CreateMessage::default(), // TODO: Make it accept None
        user,
        300.0,
    )
    .await?
    {
        PromptResult::Message(msg) => {
            builder.content = Some(msg);
            ()
        }
        PromptResult::Accept => return CreateMessage::try_from(builder).map(|cm| Some(cm)).map_err(|e| e.into()),
        PromptResult::Cancel | PromptResult::TimedOut => return Ok(None),
        PromptResult::Skip => (),
        PromptResult::Preview => {
            return Err(BotstoneError::ImpossibleError(Box::new(BotstoneError::Other(
                "Returned preview as result!".to_string(),
            ))))
        }
    }
    let created_messsage = CreateMessage::try_from(builder.clone())?;
    match prompt_for_message_part_previewed(
        ctx,
        &PROMPT_EMBED_DESCRIPTION,
        channel_id,
        &created_messsage,
        user,
        300.0,
    )
    .await?
    {
        PromptResult::Message(msg) => {
            let mut embed = EmbedBuilder::default();
            embed.description = Some(msg);
            builder.embed = Some(embed);
            ()
        }
        PromptResult::Accept => return CreateMessage::try_from(builder).map(|cm| Some(cm)).map_err(|e| e.into()),
        PromptResult::Cancel | PromptResult::TimedOut => return Ok(None),
        PromptResult::Skip => (),
        PromptResult::Preview => {
            return Err(BotstoneError::ImpossibleError(Box::new(BotstoneError::Other(
                "Returned preview as result!".to_string(),
            ))))
        }
    }

    CreateMessage::try_from(builder).map(|cm| Some(cm)).map_err(|e| e.into())
}

#[derive(EnumIter, Debug, PartialEq)]
pub enum PromptResult {
    Accept,
    Cancel,
    Skip,
    Preview,
    TimedOut,
    Message(String),
}

impl PromptResult {
    pub fn emoji_representation(&self) -> Option<String> {
        match self {
            Self::Accept => Some('\u{2705}'),   // :white_check_mark:
            Self::Cancel => Some('\u{274c}'),   // :x:
            Self::Skip => Some('\u{23e9}'),     // :fast_forward:
            Self::Preview => Some('\u{1f441}'), // :eye:
            Self::TimedOut => None,
            Self::Message(_) => None,
        }
        .map(|v| v.to_string())
    }

    pub fn all_representations() -> Vec<String> {
        Self::iter().filter_map(|x| x.emoji_representation()).collect()
    }

    pub fn from_emoji(emoji: ReactionType) -> Option<Self> {
        Self::iter()
            .filter(
                |x| matches!(x.emoji_representation(), Some(e) if ReactionType::Unicode(e.clone()) == emoji),
            )
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
    let emojis: Vec<ReactionType> = PromptResult::all_representations()
        .into_iter()
        .map(|v| ReactionType::Unicode(v))
        .collect();
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

/// Like `prompt_for_message_part`, but handles previews automatically
pub async fn prompt_for_message_part_previewed(
    ctx: &Context,
    prompt_message: &CreateMessage<'_>,
    channel_id: ChannelId,
    current_state: &CreateMessage<'_>,
    user: &User,
    timeout: f32,
) -> Result<PromptResult, BotstoneError> {
    loop {
        let message = channel_id
            .send_message(ctx, |m| {
                m.0 = prompt_message.clone().0;
                m
            })
            .await?;
        let result = prompt_for_message_part(ctx, &message, user, timeout).await?;
        if result == PromptResult::Preview {
            channel_id
                .send_message(ctx, |m| {
                    m.0 = current_state.clone().0;
                    m
                })
                .await?;
            continue;
        } else {
            return Ok(result);
        }
    }
}
