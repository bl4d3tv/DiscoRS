mod admin;
mod handlers;
mod music;
mod utils;

use handlers::Handler;

use dotenvy::dotenv;
use poise::serenity_prelude;
use songbird::SerenityInit;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Lee variables de entorno del archivo .env
    dotenv().ok();

    setup_logger();

    info!("Iniciando bot...");

    let token = std::env::var("DISCORD_TOKEN").expect("No token provided");

    let intents = serenity_prelude::GatewayIntents::non_privileged();

    // Construye el framework principal
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                admin::register::register(),
                music::play::play(),
                music::queue::queue(),
                music::skip::skip(),
                music::info::info(),
                music::stop::stop(),
            ],
            ..Default::default()
        })
        .token(&token)
        .intents(intents)
        .client_settings(|c| c.register_songbird().event_handler(Handler))
        .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .build()
        .await
        .unwrap();

    let shard_manager = framework.shard_manager().clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        info!("Recibido Ctrl+C, terminado programa...");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = framework.start().await {
        error!("Client error: {:?}", why);
    }

    //framework.run().await.unwrap();

    Ok(())
}

fn setup_logger() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
