use crate::{
    database::queries::{CustomCommands, GuildInfoTable, JoinRoles},
    structures::{context::PublicData, errors::*},
    unwrap_or_return,
    utils::{
        misc::{role_from_name_or_mention, send_rich_serialized_message},
        permissions,
    },
};
use anyhow::{anyhow, Context as AnyContext};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tracing::error;

/// Changes prefix in current guild.
#[command]
#[only_in("guilds")]
#[num_args(1)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions::check_permission(ctx, msg, Permissions::MANAGE_MESSAGES).await {
        return Err(CommandError::UserError("Lacking permissions to run this command".to_string()).into());
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
#[sub_commands(command_set, command_remove, command_list)]
async fn command(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands! (set, remove, list)")
        .await?;

    Ok(())
}

/// Set or update a custom command
/// Example: `command set website https://www.example.com`
/// You can also define more complex messages using json5
/// Example:
/// ```
/// .command set website { embed: {
///   colour: "RED",
///   description: "Visit us at https://www.example.com \nHope to see you there!",
///   footer: "Created with <3"
/// } }
/// ```
#[command("set")]
#[required_permissions(Administrator)]
#[aliases("add")]
#[min_args(2)]
async fn command_set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    } else {
        msg.channel_id
            .send_message(ctx, |msg| msg.content(content))
            .await?;
    }

    custom_commands
        .set_command(guild_id, command_name.clone(), content.to_string())
        .await?;

    msg.channel_id
        .say(ctx, format!("Command `{}` successfully set!", command_name))
        .await?;

    Ok(())
}

/// Remove custom command, by its name
#[command("remove")]
#[required_permissions(Administrator)]
#[aliases("delete", "del")]
#[num_args(1)]
async fn command_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

/// List custom commands
#[command("list")]
async fn command_list(ctx: &Context, msg: &Message) -> CommandResult {
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
                // Using `fix` to color it light yellow
                e.description(format!("```fix\n{}\n```", commands.join("\n")))
            });

            m
        })
        .await?;

    Ok(())
}

/// Manage join roles
/// Every time a new member joins the guild, they receive given roles
#[command]
#[only_in("guilds")]
#[required_permissions(Manage_Roles)]
#[sub_commands(join_role_add, join_role_remove, join_role_list)]
async fn join_role(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands! (add, remove, list)")
        .await?;
    Ok(())
}

/// Add new role that should be given to everyone that joins
/// Requires one argument - either role name or mention
#[command("add")]
#[num_args(1)]
async fn join_role_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role_str = args.single::<String>().context("Unable to get first argument")?;
    let join_roles = {
        let data = ctx.data.read().await;
        let join_roles = data.get::<JoinRoles>().context("Can't get join roles")?.clone();
        join_roles
    };
    let guild = msg
        .guild(&ctx)
        .await
        .with_context(|| format!("Not in guild: {:?}", msg))?;
    let guild_id = guild.id;
    let role_id = role_from_name_or_mention(&ctx, &guild_id, role_str).await?;
    let role = guild
        .roles
        .get(&role_id)
        .with_context(|| format!("Unable to find role with id {}", role_id))?;

    join_roles.add_join_role(guild_id, role_id).await?;

    msg.channel_id
        .say(ctx, format!("Will add {} role on join!", role.name))
        .await?;
    Ok(())
}

#[command("remove")]
#[aliases("delete", "del")]
#[num_args(1)]
async fn join_role_remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role_str = args.single::<String>().context("Unable to get first argument")?;
    let join_roles = {
        let data = ctx.data.read().await;
        let join_roles = data.get::<JoinRoles>().context("Can't get join roles")?.clone();
        join_roles
    };
    let guild = msg
        .guild(&ctx)
        .await
        .with_context(|| format!("Not in guild: {:?}", msg))?;
    let guild_id = guild.id;
    let role_id = role_from_name_or_mention(&ctx, &guild_id, role_str).await?;
    let role = guild
        .roles
        .get(&role_id)
        .with_context(|| format!("Unable to find role with id {}", role_id))?;

    join_roles.delete_join_role(guild_id, role_id).await?;

    msg.channel_id
        .say(ctx, format!("Will no longer add {} role on join!", role.name))
        .await?;
    Ok(())
}

#[command("list")]
async fn join_role_list(ctx: &Context, msg: &Message) -> CommandResult {
    let join_roles = {
        let data = ctx.data.read().await;
        let join_roles = data.get::<JoinRoles>().context("Can't get join roles")?.clone();
        join_roles
    };
    let guild = msg
        .guild(&ctx)
        .await
        .with_context(|| format!("Not in guild: {:?}", msg))?;
    let guild_id = guild.id;
    let role_ids = join_roles.get_join_roles(guild_id).await?;

    let role_names = role_ids
        .iter()
        .filter_map(|rid| guild.roles.get(&rid).map(|role| &*role.name))
        .collect::<Vec<&str>>();

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Join roles");
                // Using `fix` to color it light yellow
                e.description(format!("```fix\n{}\n```", role_names.join("\n")))
            });

            m
        })
        .await?;
    Ok(())
}

pub async fn join_role_handler(ctx: &Context, guild_id: &GuildId, new_member: &mut Member) {
    // TODO: Replace error logging here with some timed-out messages and lower priority logs
    let join_roles = {
        let data = ctx.data.read().await;
        match data.get::<JoinRoles>() {
            Some(rr) => rr.clone(),
            None => return,
        }
    };

    let roles = unwrap_or_return!(
        join_roles.get_join_roles(guild_id.clone()).await,
        |e| { error!("Error retrieving list of join roles: {:?}", e) },
        {}
    );
    for role_id in roles {
        if let Err(e) = new_member.add_role(&ctx, role_id).await {
            error!("Error assigning role to user: {:?}", e);
        };
    }
}
