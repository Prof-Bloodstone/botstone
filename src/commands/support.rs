use crate::commands::config::{command_help, prefix_help};
use crate::structures::context::VersionDataContainer;
use crate::utils::defaults::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::id::ChannelId;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

#[command]
async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() < 1 {
        default_help_message(ctx, msg.channel_id).await;
        return Ok(());
    }

    let subcommand = args.single::<String>()?;

    match subcommand.as_str() {
        "prefix" => prefix_help(ctx, msg.channel_id).await,
        "command" => command_help(ctx, msg.channel_id).await,
        _ => {}
    }

    Ok(())
}

async fn default_help_message(ctx: &Context, channel_id: ChannelId) {
    let categories = vec!["prefix", "command"];

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(DEFAULT_HELP_EMBED_COLOUR);
                e.title("BotStone Help");
                e.description("Help for the BotStone Discord bot");
                e.field(
                    "Subcategories",
                    format!("```\n{}\n```", categories.join("\n")),
                    false,
                );
                e.footer(|f| {
                    f.text("Use the support command for any further help!");
                    f
                });
                e
            })
        })
        .await;
}

#[command]
async fn support(ctx: &Context, msg: &Message) -> CommandResult {
    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(DEFAULT_HELP_EMBED_COLOUR);
                e.title("BotStone Support");
                e.description("Need more help?");
                e.field("Support Server", "UNAVAILABLE", false);
                e
            })
        })
        .await;

    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let version_data = ctx
        .data
        .read()
        .await
        .get::<VersionDataContainer>()
        .unwrap()
        .clone();
    let fields = vec![
        ("Version", version_data.version.clone()),
        ("Build Time", version_data.timestamp.clone()),
        ("Build", version_data.build.clone()),
        (
            "Source",
            format!(
                "{}:{}{}",
                version_data.branch,
                version_data.commit,
                if version_data.clean_worktree { "" } else { "*" }
            ),
        ),
        // TODO: Add more information :)
    ];
    let mut content = MessageBuilder::new();
    for (key, val) in fields {
        content.push_bold(format!("{}: ", key)).push_mono_line(val);
    }
    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.colour(DEFAULT_HELP_EMBED_COLOUR);
                e.title("BotStone");
                e.description(content.build());
                e
            })
        })
        .await;

    Ok(())
}
