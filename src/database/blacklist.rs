use anyhow::Result;
use sqlx::SqlitePool;

pub struct BlacklistData {
    pub emoji: String,
}

impl BlacklistData {
    pub fn new(emoji: String) -> Self {
        Self { emoji }
    }
}

pub async fn get_blacklist(pool: &SqlitePool) -> Result<Vec<BlacklistData>> {
    Ok(sqlx::query_as!(BlacklistData, "SELECT * FROM blacklist")
        .fetch_all(pool)
        .await?)
}

pub async fn set_emoji(blacklist: BlacklistData, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO blacklist(emoji) VALUES(?)",
        blacklist.emoji
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_emoji_by_emoji(emoji: &str, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM blacklist WHERE emoji == ?1", emoji)
        .execute(pool)
        .await?;

    Ok(())
}
