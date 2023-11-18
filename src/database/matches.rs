use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbMatch {
    pub channel: i64,
    pub user1: i64,
    pub user2: i64,
}

pub async fn set_match(data: &DbMatch, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO matches(channel, user1, user2) VALUES(?, ?, ?)",
        data.channel,
        data.user1,
        data.user2,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_match_by_channel(channel: i64, pool: &SqlitePool) -> Result<DbMatch> {
    Ok(
        sqlx::query_as!(DbMatch, "SELECT * FROM matches WHERE channel = ?", channel)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn get_match_by_user(user: i64, pool: &SqlitePool) -> Result<DbMatch> {
    Ok(sqlx::query_as!(
        DbMatch,
        "SELECT * FROM matches WHERE user1 = ? OR user2 = ?",
        user,
        user,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn delete_match_by_channel(channel: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM matches WHERE channel = ?", channel)
        .execute(pool)
        .await?;

    Ok(())
}
