mod database;
mod handler;
mod listener;
mod stardb;
mod updater;

use std::{collections::HashMap, env, str::FromStr, sync::Arc};

use anyhow::Result;
use dotenv::dotenv;
use listener::ListenerName;
use serenity::{all::GuildId, prelude::GatewayIntents, Client};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use strum::IntoEnumIterator;

use crate::handler::Handler;

const GUILD_ID: GuildId = GuildId::new(1008493665116758167);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let discord_token = env::var("DISCORD_TOKEN").unwrap();
    let database_url = env::var("DATABASE_URL").unwrap();

    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;

    let listeners = ListenerName::iter()
        .map(|l| (l.to_string(), l))
        .collect::<HashMap<_, _>>();

    let mut client = Client::builder(&discord_token, GatewayIntents::all())
        .event_handler(Handler {
            user: Arc::default(),
            listeners,
            pool: pool.clone(),
        })
        .await?;

    updater::init(client.http.clone(), pool);

    client.start().await?;

    Ok(())
}
