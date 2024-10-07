mod commands;
mod context;

use anyhow::Result;
use commands::{calc, register, reset_context};
use context::restore_contexts;
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

use poise::{
    serenity_prelude::{self as serenity, futures::lock::Mutex, UserId},
    PrefixFrameworkOptions,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub context: Arc<Mutex<HashMap<u64, fend_core::Context>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = env::var("BOT_TOKEN")?;
    let owner = u64::from_str_radix(&env::var("BOT_OWNER")?, 10)?;
    let prefix = env::var("BOT_PREFIX")?;

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let contexts = restore_contexts();

    let framework = poise::Framework::builder()
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    context: Arc::new(Mutex::new(contexts)),
                })
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![register(), calc(), reset_context()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(prefix),
                ..Default::default()
            },
            owners: HashSet::from([UserId::new(owner)]),
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();

    Ok(())
}
