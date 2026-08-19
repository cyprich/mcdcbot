use log::warn;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbed;

use crate::Error;
use crate::PoiseContext;
use crate::commands::utils;
use crate::db;
use crate::models::DimensionEnum;
use crate::models::PendingWaypoint;
use crate::models::PendingWaypointAction;

/// Root waypoints command
///
/// Contains subcommands
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("list", "add", "pending", "approve", "reject", "help"),
    aliases("waypoint", "w")
)]
pub async fn waypoints(_: PoiseContext<'_>) -> Result<(), Error> {
    Ok(())
}

/// Waypoints help command
///
/// TODO: doesn't poise have better solution for this?
#[poise::command(slash_command, prefix_command, aliases("?", "h"))]
pub async fn help(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let text = "## Waypoints help
**Available commands:**
- `/waypoints list [dimension] [completed]` - List (show) waypoints 
- `/waypoint add <name> <x> <y> <z> <dimension> [completed]`
- `/waypoints help` - Show this help screen

**Command `/waypoints list`:** 
- Example usage: 
  - `/waypoints list`
  - `/waypoints list dimension:overworld`
  - `/waypoints list dimension:end completed:false`
- Optional Arguments: 
  - `dimension` 
    - Shows only waypoints in this dimension
    - Possible values: `overworld`, `nether`, `end`
  - `completed` 
    - Shows only waypoints which are (or aren't) marked as *Completed*
    - Possible values: `true`, `false`

**Command `/waypoints add`:** 
- Example usage: 
  - `/waypoints add name:Home x:0 y:63 z:0 dimension:overworld`
  - `/waypoints add name:Gold farm x:-500 y:127 z:-500 dimension:nether`
  - `/waypoints add name:End City x:5000 y:100 z:5000 dimension:end completed:true`
- Arguments: 
  - `name` - Name of the Waypoint 
  - `x`, `y`, `z`, - Coordinates of the Waypoint
  - `dimension` - Possible values: `overworld`, `nether`, `end`
- Optional Arguments: 
  - `completed` - Possible values: `true`, `false`
";

    let embed = serenity::CreateEmbed::default().description(text);
    let reply = poise::CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}

/// Waypoints list command
///
/// Has optional parameters:
/// - dimension
/// - completed
// https://docs.rs/poise/0.6.2/src/poise/builtins/paginate.rs.html
#[poise::command(slash_command, prefix_command, aliases("show", "display"))]
pub async fn list(
    ctx: PoiseContext<'_>,
    dimension: Option<DimensionEnum>,
    completed: Option<bool>,
) -> Result<(), Error> {
    // get waypoints or say error
    let waypoints = match db::select_waypoints(&ctx.data().pool, &dimension, &completed).await {
        Ok(val) => val,
        Err(e) => {
            ctx.say(format!("Failed getting waypoints: {}", e)).await?;
            return Ok(());
        }
    };

    // check if there are any waypoints
    if waypoints.is_empty() {
        ctx.say("No waypoints found!").await?;
        return Ok(());
    }

    // create pages
    let mut pages = vec![];
    const PAGE_SIZE: usize = 8;
    let max_pages = waypoints.len().div_ceil(PAGE_SIZE);

    // each page
    for (n, page) in (1..).zip(waypoints.chunks(PAGE_SIZE)) {
        let mut text = "## Waypoints".to_string();

        // dimension, if filetered
        if let Some(dim) = &dimension {
            text.push_str(&format!("\nDimension: *{}*", dim));
        }

        // completed, if filetered
        if let Some(val) = completed {
            text.push_str(&format!(
                "\nOnly {} entries",
                match val {
                    true => "*Completed*",
                    false => "*Not Completed*",
                }
            ));
        }

        // page
        text.push_str(&format!("\nPage {}/{}", n, max_pages));

        // each waypoint
        for w in page {
            text.push('\n');
            text.push_str(&w.to_embed_text(completed.is_none(), dimension.is_none()));
        }

        pages.push(text);
    }

    let pages = pages.iter().map(String::as_str).collect::<Vec<_>>();
    poise::builtins::paginate(ctx, &pages).await?;

    Ok(())
}

/// Add Waypoint
///
/// This waypoint will go to pending waypoints
#[poise::command(slash_command, prefix_command, aliases("a", "create", "c"))]
pub async fn add(
    ctx: PoiseContext<'_>,
    name: String,
    x: i32,
    y: i32,
    z: i32,
    dimension: DimensionEnum,
    #[description = "(Optional) Whether the subject is completed (if applicable)"]
    completed: Option<bool>,
) -> Result<(), Error> {
    let pending = PendingWaypoint {
        id: 0,
        action: PendingWaypointAction::Add,
        author_name: ctx.author().name.clone(),
        author_id: ctx.author().to_string(),
        waypoint_id: None,
        x: Some(x),
        y: Some(y),
        z: Some(z),
        name: Some(name),
        dimension_id: Some(dimension.id()),
        dimension_name: Some(dimension.to_string()),
        completed: Some(completed),
    };

    let _ = db::insert_pending_waypoint(&ctx.data().pool, &pending).await?;

    ctx.say(
        "Created new Waypoint!
This Waypoint is **pending** and will appear in Waypoints as soon as Admin approves it
> See `/waypoints pending`",
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("p"))]
pub async fn pending(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let pending = db::select_pending_waypoints(&ctx.data().pool).await?;

    if pending.is_empty() {
        ctx.say("The are no Pending Waypoints!").await?;
        return Ok(());
    }

    let mut text = format!("## Pending Waypoints\n Results: {}", pending.len());
    for p in pending {
        text.push_str(&format!("\n{}", p.to_embed_text()));
    }

    let embed = CreateEmbed::default().description(text);
    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn approve(ctx: PoiseContext<'_>, id: Option<i32>) -> Result<(), Error> {
    // check permissions
    if !utils::has_permission(&ctx).await {
        ctx.say("You don't have permissions to perform this operation!")
            .await?;

        return Ok(());
    }

    // no waypoints to approve
    if !db::has_pending_waypoints(&ctx.data().pool)
        .await
        .unwrap_or(true)
    {
        ctx.say("No Waypoints to approve!").await?;
        return Ok(());
    }

    // approve one or more?
    match id {
        Some(id) => {
            let new_id = db::approve(&ctx.data().pool, Some(id)).await?;
            if new_id.is_empty() {
                // result is empty
                ctx.say("Failed to approve Waypoint, or Waypoint ID is incorrect")
                    .await?;
            } else if new_id.len() == 1 {
                // result is one number (expected situation)
                ctx.say(format!(
                    "Wapoint succesfully approved!\nNew Waypoint ID is #{}",
                    new_id[0]
                ))
                .await?;
            } else {
                // result is multiple numbers
                ctx.say(format!(
                    "It seems like multiple Waypoints were approved, which is weird :/
                    Anyways, there are the new ID's: {}",
                    new_id
                        .iter()
                        .map(i32::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .await?;
            }
        }
        None => {
            ctx.say("**Warning!** All Waypoints will be approved.\nProceed?")
                .await?;
            // TODO: buttons: continue/cancel
            let ids = db::approve(&ctx.data().pool, None).await?;
            if ids.is_empty() {
                ctx.say("Failed to approve Waypoints").await?;
            } else {
                ctx.say(format!(
                    "Waypoints approved!\nHere are the new ID's: {}",
                    ids.iter()
                        .map(i32::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("deny"))]
pub async fn reject(ctx: PoiseContext<'_>, id: Option<i32>) -> Result<(), Error> {
    // check permissions
    if !utils::has_permission(&ctx).await {
        ctx.say("You don't have permissions to perform this operation!")
            .await?;

        return Ok(());
    }

    // no waypoints to approve
    if !db::has_pending_waypoints(&ctx.data().pool)
        .await
        .unwrap_or(true)
    {
        ctx.say("No Waypoints to approve!").await?;
        return Ok(());
    }

    ctx.say("*Sorry, this feature is not implemented (yet!)*")
        .await?;

    Ok(())
}
