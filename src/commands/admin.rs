use crate::{
    database::queries::ReactionRoles,
    unwrap_or_return,
    utils::misc::{get_rich_from_args_or_prompt, role_from_name_or_mention},
};
use anyhow::Context as AnyContext;
use core::convert::TryFrom;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_channel,
};

use serenity::futures::StreamExt;
use std::fmt::Debug;
use tracing::error;

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
#[min_args(1)]
async fn message_send(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);

    let rich_message = match get_rich_from_args_or_prompt(ctx, msg.channel_id, &msg.author, &args).await? {
        Some(msg) => msg,
        None => return Ok(()),
    };

    channel
        .send_message(ctx, |msg| {
            msg.0 = rich_message.0;
            msg
        })
        .await?;
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
#[min_args(2)]
async fn message_edit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let message_id_str = args.single::<String>().context("Unable to get second argument")?;

    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);
    let message_id = u64::from_str_radix(message_id_str.as_str(), 10).context("Invalid message number")?;

    let mut message = channel
        .message(ctx, message_id)
        .await
        .context("Unable to find message")?;

    let new_message = match get_rich_from_args_or_prompt(ctx, msg.channel_id, &msg.author, &args).await? {
        Some(msg) => msg,
        None => return Ok(()),
    };

    message
        .edit(ctx, |m| {
            m.0 = new_message.0;
            m
        })
        .await?;

    Ok(())
}

// TODO: Add handlers to automatically delete RR, when:
// Message with assigned RR is deleted
// Channel with message with RR is deleted
// Role is deleted
/// React to message to get a role
#[command]
#[only_in("guilds")]
#[sub_commands(reaction_role_set, reaction_role_remove)]
#[aliases("rr")]
async fn reaction_role(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands!")
        .await?;
    Ok(())
}

/// Sets reaction role toggle.
/// React once to get the role assigned, react second time to get it removed.
/// Requires channel mention, followed by message ID, role name and reaction (emoji)
/// Example: `reaction_role set #welcome 12345678 CatPeople :cat:`
#[command("set")]
#[required_permissions(Administrator)]
#[aliases("new", "add", "create")]
#[num_args(4)]
async fn reaction_role_set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reaction_roles = {
        let data = ctx.data.read().await;
        let reaction_roles = data
            .get::<ReactionRoles>()
            .context("Can't get reaction roles")?
            .clone();
        reaction_roles
    };
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let message_id_str = args.single::<String>().context("Unable to get second argument")?;
    let role_name = args.single::<String>().context("Unable to get third argument")?;
    let reaction_string = args.single::<String>().context("Unable to get fourth argument")?;

    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);
    let message_id = u64::from_str_radix(message_id_str.as_str(), 10).context("Invalid message number")?;

    let message = channel
        .message(ctx, message_id)
        .await
        .context("Unable to find message")?;

    let guild_id = msg.guild_id.context("Not in a guild")?;
    let role_id = role_from_name_or_mention(&ctx, &guild_id, role_name).await?;
    let reaction = ReactionType::try_from(reaction_string.clone())
        .with_context(|| format!("Invalid emoji: {:?}", reaction_string.clone()))?;

    message
        .react(ctx, reaction)
        .await
        .context("Unable to react to message. Is the emoji valid?")?;
    reaction_roles
        .set_react_role(guild_id, channel, message.id, role_id, reaction_string)
        .await
        .context("Unable to save reaction role")?;
    msg.channel_id
        .send_message(ctx, |m| m.content("Reaction role set sucessfully"))
        .await?;

    Ok(())
}

/// Removes reaction role toggle.
/// Requires channel mention, followed by message ID, role name and reaction (emoji)
/// Example: `reaction_role remove #welcome 12345678 :cat:`
#[command("remove")]
#[required_permissions(Administrator)]
#[aliases("rm", "delete")]
#[num_args(3)]
async fn reaction_role_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reaction_roles = {
        let data = ctx.data.read().await;
        let reaction_roles = data
            .get::<ReactionRoles>()
            .context("Can't get reaction roles")?
            .clone();
        reaction_roles
    };
    let channel_mention = args.single::<String>().context("Unable to get first argument")?;
    let message_id_str = args.single::<String>().context("Unable to get second argument")?;
    let reaction_string = args.single::<String>().context("Unable to get third argument")?;

    let channel_number = parse_channel(channel_mention.clone())
        .with_context(|| format!("Not a valid channel mention: {:?}", channel_mention))?;
    let channel = ChannelId(channel_number);
    let message_id = u64::from_str_radix(message_id_str.as_str(), 10).context("Invalid message number")?;

    let guild_id = msg.guild_id.context("Not in a guild")?;

    let deleted = reaction_roles
        .delete_react_role(guild_id, channel, message_id.into(), reaction_string)
        .await
        .context("Error removing reaction role")?;
    let delete_msg = if deleted == 0 {
        "Reaction role not found"
    } else {
        "Reaction role removed sucessfully"
    };
    msg.channel_id
        .send_message(ctx, |m| m.content(delete_msg))
        .await?;

    Ok(())
}

pub async fn reaction_role_handler(ctx: &Context, reaction: &Reaction) {
    // TODO: Replace error logging here with some timed-out messages and lower priority logs
    let reaction_roles = {
        let data = ctx.data.read().await;
        match data.get::<ReactionRoles>() {
            Some(rr) => rr.clone(),
            None => return,
        }
    };

    let guild_id = match reaction.guild_id {
        Some(guild_id) => guild_id,
        None => return,
    };
    let role_result = reaction_roles
        .get_react_role(
            guild_id,
            reaction.channel_id,
            reaction.message_id,
            reaction.emoji.to_string(),
        )
        .await;
    let maybe_role_id = unwrap_or_return!(
        role_result,
        |e: &dyn Debug| {
            error!("Error getting ReactRole: {:?}", e);
        },
        {}
    );
    let role_id = unwrap_or_return!(maybe_role_id);
    let user = unwrap_or_return!(
        reaction.user(&ctx).await,
        |e: &dyn Debug| {
            error!("Error getting ReactRole user: {:?}", e);
        },
        {}
    );
    let mut member = unwrap_or_return!(
        guild_id.member(&ctx, user.id).await,
        |e: &dyn Debug| {
            error!("Error getting ReactRole member: {:?}", e);
        },
        {}
    );
    if member.roles.contains(&role_id) {
        if let Err(e) = member.remove_role(&ctx, role_id).await {
            error!("Error removing role from user: {:?}", e);
        };
    } else {
        if let Err(e) = member.add_role(&ctx, role_id).await {
            error!("Error assigning role to user: {:?}", e);
        };
    }

    if let Err(e) = reaction.delete(&ctx).await {
        error!("Error deleting ReactRole reaction: {:?}", e);
    }
}

/// Give role in bulk to people with another role
/// Usage: .bulk_role NewSuperRole OldRole
#[command]
#[only_in("guilds")]
#[required_permissions(Manage_Roles)]
#[num_args(2)]
async fn bulk_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.context("Not in a guild")?;
    let mut role_ids = Vec::new();
    for role_name_result in args.iter::<String>() {
        let role_name = role_name_result.context("Unable to iterate over arguments!")?;
        let role_id = role_from_name_or_mention(&ctx, &guild_id, role_name.clone())
            .await
            .context(format!("Unable to parse role: `{}`", role_name))?;
        if u64::from(role_id) != u64::from(guild_id) {
            // Special case for @everyone
            role_ids.push(role_id);
        }
    }
    let role_to_add = role_ids.swap_remove(0);

    let mut role_add_counter = 0;
    let mut members_stream = guild_id.members_iter(&ctx).boxed();
    while let Some(member_result) = members_stream.next().await {
        let mut member = member_result
            .context("Error getting member information - role application might be in partial state")?;
        if role_ids.iter().all(|role_id| member.roles.contains(role_id)) {
            member.add_role(&ctx, role_to_add).await.context(format!(
                "Error giving {} a role.",
                member.nick.unwrap_or(member.user.name)
            ))?;
            role_add_counter += 1;
        }
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "Succesfully applied role to {} members.",
                role_add_counter
            ))
        })
        .await?;

    Ok(())
}
