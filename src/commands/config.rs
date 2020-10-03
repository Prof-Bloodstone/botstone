use crate::{
    database::queries::GuildInfoTable,
    structures::context::{CommandNameMap, ConnectionPool},
    utils::defaults::*,
    utils::permissions,
};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

/// Sets the prefix for the server using the first message argument
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

pub async fn prefix_help(ctx: &Context, channel_id: ChannelId) {
    let mut content = String::new();
    content.push_str("prefix <character(s)>: Sets the server's prefix (Can be one or multiple characters)");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(DEFAULT_HELP_EMBED_COLOUR);
                e.title("Custom Prefix Help");
                e.description("Description: Commands for custom bot prefixes");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}

/// Custom commands for your server that output a message
/// Usage to set: `command set <name> <content to be said>`
/// Usage to remove: `command remove <name>`
#[command]
#[sub_commands(set, remove, list)]
async fn command(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "Please use one of the subcommands! (set, remove, list)")
        .await?;

    Ok(())
}

// Subcommand to set/update a custom command
#[command]
#[required_permissions(Administrator)]
#[aliases("add")]
#[min_args(2)]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.single::<String>().unwrap();
    let (command_names, pool) = {
        let data = ctx.data.read().await;
        let command_names = data.get::<CommandNameMap>().unwrap().clone();
        let pool = data.get::<ConnectionPool>().unwrap().clone();
        (command_names.clone(), pool)
    };

    if command_names.contains(&command_name) {
        msg.channel_id
            .say(
                ctx,
                "This command is already hardcoded! Please choose a different name!",
            )
            .await?;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap().0 as i64;

    sqlx::query!(
        "INSERT INTO commands(guild_id, name, content)
            VALUES($1, $2, $3)
            ON CONFLICT (guild_id, name)
            DO UPDATE
            SET content = EXCLUDED.content",
        guild_id,
        command_name,
        args.rest()
    )
    .execute(&*pool)
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
    let pool = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap().clone();
        pool
    };
    let guild_id = msg.guild_id.unwrap().0 as i64;

    sqlx::query!(
        "DELETE FROM commands WHERE guild_id = $1 AND name = $2",
        guild_id,
        command_name
    )
    .execute(&*pool)
    .await?;

    msg.channel_id
        .say(ctx, format!("Command {} successfully deleted!", command_name))
        .await?;

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap().clone();
        pool
    };
    let guild_id = msg.guild_id.unwrap().0 as i64;
    let mut command_map: Vec<String> = Vec::new();

    let command_data = sqlx::query!("SELECT name FROM commands WHERE guild_id = $1", guild_id)
        .fetch_all(&*pool)
        .await?;

    for i in command_data {
        command_map.push(i.name);
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Custom commands");
                e.description(format!("```{} \n```", command_map.join(" \n")))
            });

            m
        })
        .await?;

    Ok(())
}

pub async fn command_help(ctx: &Context, channel_id: ChannelId) {
    let mut content = String::new();
    content
        .push_str("set <name> <content>: Sets a new custom command, {user} is replaced with a mention \n\n");
    content.push_str("remove <name>: Removes an existing custom command \n\n");
    content.push_str("list: Lists all custom commands in the server");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(DEFAULT_HELP_EMBED_COLOUR);
                e.title("Custom Command Help");
                e.description("Description: Custom command configuration (For administrators only!)");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
