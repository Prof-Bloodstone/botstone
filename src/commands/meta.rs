use crate::utils::channel::AsEmoji;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{debug, error, info};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
#[num_args(1)]
#[usage = ".react <EMOJI>"]
async fn react(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 1 {
        let emoji_arg = args.single_quoted::<String>().unwrap();
        debug!("Trying to parse {:?} as emoji", emoji_arg);
        match emoji_arg.as_emoji() {
            Ok(reaction) => {
                info!("Found reaction: {:?}", reaction);
                msg.react(ctx, reaction).await?;
            }
            Err(e) => {
                let error_msg = format!("Unable to parse {:?} as emoji", emoji_arg);
                error!("{:?} - the full error was {:?}", error_msg, &e);
                msg.reply(&ctx.http, error_msg).await?;
            }
        };
    }

    Ok(())
}
