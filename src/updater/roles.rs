use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use serenity::all::Http;
use sqlx::SqlitePool;

use crate::database;

pub async fn update(http: &Arc<Http>, pool: &SqlitePool) -> Result<()> {
    let mut d = HashSet::new();

    let users = database::get_users(pool).await?;
    for user in users {
        super::update_user_roles(user, &mut d, http, pool).await?;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
