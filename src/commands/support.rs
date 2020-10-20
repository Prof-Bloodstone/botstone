use crate::{structures::context::VersionDataContainer, utils::defaults::*};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
    utils::MessageBuilder,
};

#[command]
#[description = "Points where to get support and report issues."]
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
#[description = "Returns build information about the bot."]
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
