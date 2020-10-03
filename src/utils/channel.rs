use serenity::model::channel::{ReactionConversionError, ReactionType};
use serenity::static_assertions::_core::convert::TryFrom;
use unic_emoji_char::is_emoji;

pub trait AsEmoji {
    fn as_emoji(&self) -> Result<ReactionType, ReactionConversionError>;
}

impl AsEmoji for &str {
    fn as_emoji(&self) -> Result<ReactionType, ReactionConversionError> {
        return self.to_string().as_emoji();
    }
}

impl AsEmoji for String {
    fn as_emoji(&self) -> Result<ReactionType, ReactionConversionError> {
        return ReactionType::try_from((*self).clone()).and_then(|reaction| match &reaction {
            ReactionType::Unicode(string) => {
                let chars = string.chars().collect::<Vec<char>>();
                return if chars.len() != 1 || !is_emoji(chars[0].clone()) {
                    Err(ReactionConversionError)
                } else {
                    Ok(reaction)
                };
            }
            _ => Ok(reaction),
        });
    }
}
