use serenity::{prelude::*, framework::standard::CommandResult, model::id::{ChannelId, GuildId, MessageId}};

pub fn get_message_url(guild_id: GuildId, channel_id: ChannelId, message_id: MessageId) -> String {
    format!(
        "https://discordapp.com/channels/{}/{}/{}",
        guild_id.0, channel_id.0, message_id.0
    )
}

pub async fn send_rich_serialized_message(ctx: &Context, channel_id: ChannelId, serialized_message: &str) -> CommandResult {
    let deserialize_result = serde_json::from_str::<Message>(content);
    match deserialize_result {
        Err(e) => {
            let error_msg = format!("Unable to deserialize rich response. The error was: {:#?}");
            warn!(format!("{}. The content was `{:?}`", error_msg, content));
            channel_id.say(ctx, error_msg).await?
        },
        Ok(deserialized_msg) => {

        }
    }
}
