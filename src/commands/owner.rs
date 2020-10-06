use crate::ShardManagerContainer;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
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
#[command]
async fn _test(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut builder = MessageBuilder::new();
    builder.push_line(format!("You passed following {} arguments:", args.len()));
    for (pos, arg) in args.iter::<String>().enumerate() {
        builder.push(format!("{}: #{}#\n", pos, arg.unwrap()));
    }
    builder.push_line("All args:").push(args.message());
    msg.channel_id.say(&ctx.http, builder.build()).await?;
    Ok(())
}
