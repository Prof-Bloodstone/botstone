use serenity::framework::standard::{CommandOptions, Reason};
use serenity::{model::prelude::*, prelude::*};
use tracing::debug;
use anyhow::Context as AnyContext;

use crate::database::queries::{Permissions as PermissionsDB, RolePermission};
use crate::parsers::permissions::Permissible;
use crate::structures::errors::BotstoneError;
use crate::utils::guild::get_message_author_member;



#[check]
async fn has_perms(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    opts: &CommandOptions,
    ) -> Result<(), Reason> {
    message_author_has_permission(ctx, msg, )

}


/// Checks whether user has permission to run command
pub async fn message_author_has_permission<F>(ctx: &Context, msg: &Message, permission_node: &String) -> Result<bool, BotstoneError> {
    let member = get_message_author_member(ctx, msg).await?;
    let roles = member.roles;
    let mut permissions = ctx.data.read().await.get::<PermissionsDB>().context("Can't get Permissions")?.get_permissions(msg.guild_id.expect("Not in guild")).await?;
    Ok(has_permission(roles, &mut permissions, permission_node))
}


pub fn has_permission(roles: Vec<RoleId>, role_permissions: &mut Vec<RolePermission>, permission: &String) -> bool {
    role_permissions.sort_unstable_by_key(|rp| {rp.permission().node().len()});
    role_permissions.iter()
        .filter(|rp| roles.contains(rp.role_id()))
        .map(|rp| rp.permission())
        .any(|perm| perm.matches(permission))
}


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
