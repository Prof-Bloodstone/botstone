use crate::ShardManagerContainer;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(&ctx.http, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(&ctx.http, "There was a problem getting the shard manager")
            .await?;
        return Ok(());
    }

    Ok(())
}
