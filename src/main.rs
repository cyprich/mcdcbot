use log::{error, trace};
use poise::serenity_prelude as serenity;
use simple_logger::SimpleLogger;

pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type PoiseContext<'a> = poise::Context<'a, Data, Error>;

mod commands;

use commands::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // create simple logger
    let ignored = [
        "tokio_tungstenite",
        "tracing",
        "serenity",
        "hyper_util",
        "reqwest",
        "rustls",
        "tungstenite",
    ];
    let mut logger = SimpleLogger::new();
    for i in ignored {
        logger = logger.with_module_level(i, log::LevelFilter::Off)
    }
    match logger.init() {
        Ok(_) => {
            trace!("SimpleLogger initiated")
        }
        Err(e) => {
            eprintln!("Failed to initiate SimpleLogger: {}", e);
            panic!()
        }
    }

    // load env variables
    dotenvy::dotenv()?;
    trace!("Dotenv loaded");

    // get token from env variables
    let token = match std::env::var("DISCORD_TOKEN") {
        Ok(val) => {
            trace!("Got Discord token");
            val
        }
        Err(e) => {
            error!("Failed to get token: {}", e);
            panic!()
        }
    };

    // get guild id from from env variables
    let guild_id = std::env::var("DISCORD_GUILD_ID");

    // intents
    let intents = serenity::GatewayIntents::non_privileged();
    trace!("Got intents: {:?}", intents);

    // commands
    let commands = vec![ping(), age(), hello()];

    // framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                trace!("Logged in as '{}'", ready.user.name);

                match guild_id {
                    Ok(val) => {
                        trace!("Running in Developer (GUILD) mode");
                        let guild_id = serenity::GuildId::new(val.parse()?);
                        let commands = &framework
                            .options()
                            .commands
                            .iter()
                            .filter_map(|c| c.create_as_slash_command())
                            .collect::<Vec<_>>();

                        guild_id.set_commands(ctx, commands.clone()).await?;

                        trace!("OK");
                    }
                    Err(_) => {
                        trace!("Running in Production (GLOBAL) mode");
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                        trace!("OK");
                    }
                }

                trace!("Commands registered:");

                for c in &framework.options().commands {
                    trace!("  - {}", c.name)
                }

                Ok(Data {})
            })
        })
        .build();
    trace!("Got framework");

    // init client
    let mut client = match serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
    {
        Ok(val) => {
            trace!("Client initiated");
            val
        }
        Err(e) => {
            error!("Failed to initiate client: {}", e);
            panic!()
        }
    };

    // start client
    client.start().await.unwrap();

    Ok(())
}
