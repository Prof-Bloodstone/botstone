use serenity::{model::prelude::*, prelude::*};
use tracing::debug;

pub async fn check_permission(ctx: &Context, msg: &Message, permission: Permissions) -> bool {
    let guild_id = match msg.guild_id {
        Some(guild_id) => guild_id,
        None => return false,
    };
    let member = match guild_id.member(&ctx, msg.author.id).await {
        Ok(member) => member,
        Err(_) => return false,
    };

    let permissions = match member.permissions(&ctx).await {
        Ok(perms) => perms,
        Err(e) => {
            debug!("Error getting user permission: {}", e);
            return false;
        }
    };

    if permissions.contains(permission) {
        return true;
    }

    let _ = match permission {
        Permissions::ADMINISTRATOR => {
            msg.channel_id
                .say(
                    ctx,
                    "You can't execute this command because you aren't an administrator!",
                )
                .await
        }
        Permissions::MANAGE_MESSAGES => msg
            .channel_id
            .say(
                ctx,
                "You can't execute this command because you aren't a moderator (Manage Messages permission)!",
            )
            .await,
        _ => msg.channel_id.say(ctx, "You can't execute this command!").await,
    };

    return false;
}
