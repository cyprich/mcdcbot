use log::{error, warn};
use poise::serenity_prelude as serenity;

use crate::PoiseContext;

/// True, if any of these are true:
/// - User has the Admin role (role ID is defined in `DISCORD_ADMIN_ROLE_ID` environment variable)
/// - User is the server admin
pub async fn has_permission(ctx: &PoiseContext<'_>) -> bool {
    let member = match ctx.author_member().await {
        Some(val) => val,
        None => {
            error!("This command can only be used in a server");
            return false;
        }
    };

    // is system admin?
    if member
        .permissions
        .map(|p| p.administrator())
        .unwrap_or(false)
    {
        return true;
    }

    // has Admin role?
    let admin_role_id = match std::env::var("DISCORD_ADMIN_ROLE_ID") {
        // if env var exists
        Ok(id) => {
            // convert string -> u64
            let parsed: u64 = match id.parse() {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse Role ID: {}", e);
                    return false;
                }
            };

            // return serrenity::RoleId
            serenity::RoleId::new(parsed)
        }
        // if env var does not exist
        Err(e) => {
            warn!(
                "Environment variable 'DISCORD_ADMIN_ROLE_ID' not set or unavailable: {}",
                e
            );

            return false;
        }
    };

    if member.roles.contains(&admin_role_id) {
        return true;
    }

    return false;
}
