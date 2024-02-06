mod commands;
mod database;
mod handler;
mod stardb;
mod updater;

use std::{env, str::FromStr};

use anyhow::Result;
use dotenv::dotenv;
use serenity::{prelude::GatewayIntents, Client};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

use crate::handler::Handler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let discord_token = env::var("DISCORD_TOKEN").unwrap();
    let database_url = env::var("DATABASE_URL").unwrap();

    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;

    let mut client = loop {
        if let Ok(client) = Client::builder(
            &discord_token,
            GatewayIntents::non_privileged() | GatewayIntents::GUILD_MEMBERS,
        )
        .event_handler(Handler { pool: pool.clone() })
        .await
        {
            break client;
        }
    };

    updater::init(client.http.clone(), pool);

    client.start().await?;

    Ok(())
}
