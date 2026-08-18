use crate::Error;
use crate::PoiseContext;

#[poise::command(slash_command, prefix_command)]
pub async fn hello(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.say("Hello, World!").await?;
    Ok(())
}
