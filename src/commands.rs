use poise::serenity_prelude as serenity;

use crate::Error;
use crate::PoiseContext;

#[poise::command(slash_command, prefix_command)]
pub async fn hello(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.say("Hello, World!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.say("Pong").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: PoiseContext<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let resp = format!(
        "{}'s user account was created at {}",
        u.name,
        u.created_at()
    );
    ctx.say(resp).await?;

    Ok(())
}
