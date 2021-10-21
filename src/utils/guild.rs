use serenity::{client::Context, model::{channel::Message, guild::Member}};

use crate::structures::errors::BotstoneError;


pub async fn get_message_author_member(ctx: &Context, msg: &Message) -> Result<Member, BotstoneError> {
    msg.guild_id.ok_or(BotstoneError::Other("message guild_id is None".to_string()))?
    .member(&ctx, msg.author.id).await.map_err(|e| BotstoneError::SerenityError(e))
}

