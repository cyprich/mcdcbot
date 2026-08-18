use crate::Error;
use crate::PoiseContext;
use crate::db;
use crate::models::DimensionEnum;

// https://docs.rs/poise/0.6.2/src/poise/builtins/paginate.rs.html
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("list"),
    aliases("waypoint", "w")
)]
pub async fn waypoints(ctx: PoiseContext<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: PoiseContext<'_>, dimension: Option<DimensionEnum>) -> Result<(), Error> {
    // get waypoints or say error
    let waypoints = match db::select_waypoints(&ctx.data().pool, &dimension).await {
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
            text.push_str(&format!("\nDimension: {}", dim));
        }

        // page
        text.push_str(&format!("\nPage {}/{}", n, max_pages));

        // each waypoint
        for w in page {
            // title kinda
            let mut name = format!("**{}** \\#{} ", w.name, w.id);
            // completed
            if let Some(val) = w.completed
                && val
            {
                name.push_str("*(Completed)*");
            };
            // name, coords
            text.push_str(&format!("\n\n{}\n{} / {} / {}", name, w.x, w.y, w.z));

            // dimension, if not filtered
            if dimension.is_none() {
                text.push_str(&format!("\n> {}", w.dimension));
            }
        }

        pages.push(text);
    }

    let pages = pages.iter().map(String::as_str).collect::<Vec<_>>();
    poise::builtins::paginate(ctx, &pages).await?;

    Ok(())
}
