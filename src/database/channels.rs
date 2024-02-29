use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbChannel {
    pub channel: i64,
}

pub async fn get_channels(pool: &SqlitePool) -> Result<Vec<DbChannel>> {
    Ok(sqlx::query_as!(DbChannel, "SELECT * FROM channels")
        .fetch_all(pool)
        .await?)
}

pub async fn set_channel(channel: DbChannel, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO channels(channel) VALUES(?)",
        channel.channel
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_channel_by_channel(channel: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM channels WHERE channel == ?1", channel)
        .execute(pool)
        .await?;

    Ok(())
}
