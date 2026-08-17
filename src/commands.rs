use crate::Error;
use crate::PoiseContext;
use crate::db;

#[poise::command(slash_command, prefix_command)]
pub async fn hello(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.say("Hello, World!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn waypoints(ctx: PoiseContext<'_>) -> Result<(), Error> {
    // get waypoints or say error
    let waypoints = match db::select_waypoints(&ctx.data().pool).await {
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

    // construct message text
    let mut text = format!("Waypoints ({}):", waypoints.len());
    for w in waypoints {
        text.push_str(&format!("\n  {}", w));
    }

    // say text in chunks/pages
    for chunk in text.chars().collect::<Vec<_>>().chunks(1900) {
        let message: String = chunk.iter().collect();
        ctx.say(message).await?;
    }

    Ok(())
}
