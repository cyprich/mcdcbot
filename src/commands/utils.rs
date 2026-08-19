use log::{error, warn};
use poise::serenity_prelude as serenity;

use crate::{Error, PoiseContext, db};

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

    false
}

pub enum ApproveOrRejectEnum {
    Approve,
    Reject,
}

impl ApproveOrRejectEnum {
    /// "Approved" or "Rejected"
    pub fn to_verb(&self) -> String {
        match self {
            ApproveOrRejectEnum::Approve => "Approved".to_string(),
            ApproveOrRejectEnum::Reject => "Rejected".to_string(),
        }
    }

    /// "Approve" or "Reject"
    pub fn to_noun(&self) -> String {
        match self {
            ApproveOrRejectEnum::Approve => "Approve".to_string(),
            ApproveOrRejectEnum::Reject => "Reject".to_string(),
        }
    }
}

pub async fn approve_or_reject_pending_waypoints(
    ctx: &PoiseContext<'_>,
    id: Option<i32>,
    action: ApproveOrRejectEnum,
) -> Result<(), Error> {
    // check permissions
    if !has_permission(ctx).await {
        ctx.say("You don't have permissions to perform this operation!")
            .await?;

        return Ok(());
    }

    // no waypoints to approve/reject
    if !db::has_pending_waypoints(&ctx.data().pool)
        .await
        .unwrap_or(true)
    {
        ctx.say(format!("No Waypoints to {}!", action.to_noun()))
            .await?;
        return Ok(());
    }

    // warn if id was not specified (all records will be approved/rejected)
    let ready_to_proceed = match id {
        Some(_) => true,
        None => {
            // TODO: buttons: continue/cancel
            ctx.say(format!(
                "**Warning!** All Waypoints will be {}.\nProceed?",
                action.to_verb()
            ))
            .await?;
            false
        }
    };
    if !ready_to_proceed {
        return Ok(());
    }

    // execute the database operation
    let ids = match action {
        ApproveOrRejectEnum::Approve => db::approve(&ctx.data().pool, id).await?,
        ApproveOrRejectEnum::Reject => db::reject(&ctx.data().pool, id).await?,
    };

    // feedback
    // checks the number of returned IDs agains how many actually should have been returned
    match (ids.len(), id) {
        // nothing was approved/rejected => error happened (most likely)
        (0, _) => {
            ctx.say(format!("No Waypoints were ${}!", action.to_verb()))
                .await?;
        }
        // 1 was approved/rejected, and one should have been => wanted situation
        (1, Some(_)) => {
            ctx.say(format!(
                "Waypoint succesfully {}!\n{}: {}",
                action.to_verb(),
                match action {
                    ApproveOrRejectEnum::Approve => "New Waypoint ID",
                    ApproveOrRejectEnum::Reject => "Rejected Pending Waypoint ID",
                },
                ids[0]
            ))
            .await?;
        }
        // multiple were approved/rejected, and multiple should have been => wanted situation
        (_, None) => {
            ctx.say(format!(
                "Waypoints succesfully {}!\nIDs: {}",
                action.to_verb(),
                ids.iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .await?;
        }
        // multiple were approved/rejected, but only one should have been => partially unwanted situation, should not happen
        (_, Some(_)) => {
            ctx.say(format!(
                "Multiple Waypoints were {}, which is weird :/\nAnyways, here are the IDs: {}",
                action.to_verb(),
                ids.iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .await?;
        }
    }

    Ok(())
}
