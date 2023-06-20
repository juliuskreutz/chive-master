use anyhow::Result;
use derive_getters::Getters;
use sqlx::SqlitePool;

#[derive(Getters)]
pub struct ChannelData {
    channel: i64,
}

impl ChannelData {
    pub fn new(channel: i64) -> Self {
        Self { channel }
    }
}

pub async fn get_channels(pool: &SqlitePool) -> Result<Vec<ChannelData>> {
    Ok(sqlx::query_as!(ChannelData, "SELECT * FROM channels")
        .fetch_all(pool)
        .await?)
}

pub async fn set_channel(channel: ChannelData, pool: &SqlitePool) -> Result<()> {
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
