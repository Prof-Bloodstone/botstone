use crate::{
    database::queries::{CustomCommands, GuildInfoTable},
    structures::context::PublicData,
    utils::{
        misc::{deserialize_rich_message, send_rich_serialized_message},
        permissions,
    },
};
use anyhow::Context as AnyContext;
use serenity::{
    builder::CreateMessage,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_channel,
};

/// Custom messages supporting embeds
/// You can edit existing message
#[command]
#[only_in("guilds")]
#[sub_commands(message_send, message_edit)]
async fn message(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands!")
        .await?;

    Ok(())
}

/// Send a message to given channel
/// Example: `message send #welcome Our website: https://www.example.com`
/// You can also define more complex messages using json5
/// Example:
/// ```
/// .message send #welcome { embed: {
///   colour: "RED",
///   description: "Visit us at https://www.example.com \nHope to see you there!",
///   footer: "Created with <3"
/// } }
/// ```
#[command("send")]
#[required_permissions(Administrator)]
#[aliases("new")]
#[min_args(2)]
async fn message_send(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let content = args.rest();
    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);

    if content.starts_with("{") {
        // Assume this is special content, which needs to be parsed
        send_rich_serialized_message(ctx, channel, content).await?;
    } else {
        channel.send_message(ctx, |msg| msg.content(content)).await?;
    }
    Ok(())
}

/// Edit previously sent message
/// Requires channel mention, followed by message ID and new message content
/// Example: `message edit #welcome 12345678 This is new content :)`
/// To be able to copy message ID, open **User Settings** by clicking cog wheel next to your name.
/// Then go to **Appearance** and enable **Developer Mode** at the bottom
#[command("edit")]
#[required_permissions(Administrator)]
#[aliases("update")]
#[min_args(3)]
async fn message_edit(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let message_id_str = args.single::<String>().context("Unable to get second argument")?;
    let content = args.rest();

    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);
    let message_id = u64::from_str_radix(message_id_str.as_str(), 10).context("Invalid channel number")?;

    let mut message = channel
        .message(ctx, message_id)
        .await
        .context("Unable to find message")?;

    let new_message = if content.starts_with("{") {
        // Assume this is special content, which needs to be parsed
        deserialize_rich_message(content)?
    } else {
        let mut builder = CreateMessage::default();
        builder.content(content).to_owned()
    };
    message
        .edit(ctx, |m| {
            m.0 = new_message.0;
            m
        })
        .await?;

    Ok(())
}
