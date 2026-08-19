use poise::serenity_prelude as serenity;

use crate::Error;
use crate::PoiseContext;
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
    subcommands("list", "add", "help"),
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
- `/waypoints help` - Show this help screen

**Command `/waypoints list`:** 
- Example usage: 
  - `/waypoint list`
  - `/waypoint list dimension:overworld`
  - `/waypoint list dimension:end completed:false`
- Optional Arguments: 
  - `dimension` 
    - Shows only waypoints in this dimension
    - Possible values: `overworld`, `nether`, `end`
  - `completed` 
    - Shows only waypoints which are (or aren't) marked as *Completed*
    - Possible values: `true`, `false`";

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
            text.push_str(&w.to_embed_text(completed.is_none(), dimension.is_none()));
        }

        pages.push(text);
    }

    let pages = pages.iter().map(String::as_str).collect::<Vec<_>>();
    poise::builtins::paginate(ctx, &pages).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("a", "create", "c"))]
pub async fn add(
    ctx: PoiseContext<'_>,
    #[description = "Name of the waypoint"] name: String,
    x: i32,
    y: i32,
    z: i32,
    dimension: DimensionEnum,
    completed: Option<bool>,
) -> Result<(), Error> {
    let pending = PendingWaypoint {
        id: 0,
        action: PendingWaypointAction::Add,
        author: ctx.author().name.clone(),
        waypoint_id: None,
        x: Some(x),
        y: Some(y),
        z: Some(z),
        name: Some(name),
        dimension: Some(dimension.to_string()),
        completed: Some(completed),
    };

    let id = db::insert_pending_waypoint(&ctx.data().pool, &pending).await?;

    ctx.say(format!(
"Created new Waypoint!
This Waypoint is **pending** with ID **#{id}** and will appear in Waypoints as soon as Admin approves it
> See `/waypoints pending`",
    ))
    .await?;

    Ok(())
}
