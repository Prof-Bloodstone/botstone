use crate::{
    database::queries::{CustomCommands, GuildInfoTable},
    structures::context::PublicData,
    utils::{misc::send_rich_serialized_message, permissions},
};
use anyhow::{anyhow, Context as AnyContext};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

/// Changes prefix in current guild.
#[command]
#[only_in("guilds")]
#[num_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions::check_permission(ctx, msg, Permissions::MANAGE_MESSAGES).await {
        return Ok(());
    }
    let guild_info = {
        let data = ctx.data.read().await;
        let guild_info = data.get::<GuildInfoTable>().unwrap().clone();
        guild_info
    };
    let guild_id = msg.guild_id.unwrap();
    let guild_name = msg.guild(ctx).await.unwrap().name;

    let new_prefix = args.single::<String>().unwrap();

    guild_info.set_prefix(guild_id, new_prefix.clone()).await?;

    msg.channel_id
        .say(
            ctx,
            format!("My new prefix for `{}` is `{}`!", guild_name, new_prefix),
        )
        .await?;
    Ok(())
}

/// Custom commands for your server that output a message
/// Usage to set: `command set <name> <content to be said>`
/// Usage to remove: `command remove <name>`
#[command]
#[only_in("guilds")]
#[sub_commands(set, remove, list)]
async fn command(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands! (set, remove, list)")
        .await?;

    Ok(())
}

/// set/update a custom command
/// command set website https://www.example.com
#[command]
#[required_permissions(Administrator)]
#[aliases("add")]
#[min_args(2)]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.single::<String>().context("Unable to get first argument")?;
    let (command_names, custom_commands) = {
        let data = ctx.data.read().await;
        let command_names = data
            .get::<PublicData>()
            .context("Can't get public data")?
            .hardcoded_commands
            .clone();
        let custom_commands = data
            .get::<CustomCommands>()
            .context("Can't get custom commands")?
            .clone();
        (command_names.clone(), custom_commands)
    };

    if command_names.contains(&command_name) {
        msg.channel_id
            .say(
                ctx,
                "This command is already hardcoded! Please choose a different name!",
            )
            .await?;
        Err(anyhow!("Command {} is already hardcoded.", command_name))?;
    }

    let guild_id = msg.guild_id.with_context(|| format!("Not in guild: {:?}", msg))?;

    let content = args.rest();

    if content.starts_with("{") {
        // Assume this is special content, which needs to be parsed
        // So check if it can be deserialized
        send_rich_serialized_message(ctx, msg.channel_id, content).await?;
    }

    custom_commands
        .set_command(guild_id, command_name.clone(), content.to_string())
        .await?;

    msg.channel_id
        .say(ctx, format!("Command `{}` successfully set!", command_name))
        .await?;

    Ok(())
}

// Subcommand used to remove a custom command
#[command]
#[required_permissions(Administrator)]
#[num_args(1)]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.single::<String>().unwrap();
    let custom_commands = {
        let data = ctx.data.read().await;
        let custom_commands = data
            .get::<CustomCommands>()
            .context("Can't get custom commands")?
            .clone();
        custom_commands
    };
    let guild_id = msg.guild_id.with_context(|| format!("Not in guild: {:?}", msg))?;

    custom_commands
        .delete_command(guild_id, command_name.to_string())
        .await?;

    msg.channel_id
        .say(ctx, format!("Command {} successfully deleted!", command_name))
        .await?;

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let custom_commands = {
        let data = ctx.data.read().await;
        let custom_commands = data
            .get::<CustomCommands>()
            .context("Can't get custom commands")?
            .clone();
        custom_commands
    };
    let guild_id = msg.guild_id.with_context(|| format!("Not in guild: {:?}", msg))?;
    let commands = custom_commands.get_command_names(guild_id).await?;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Custom commands");
                e.description(format!("```{} \n```", commands.join(" \n")))
            });

            m
        })
        .await?;

    Ok(())
}
