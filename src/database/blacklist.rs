use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbBlacklist {
    pub emoji: String,
}

pub async fn get_blacklist(pool: &SqlitePool) -> Result<Vec<DbBlacklist>> {
    Ok(sqlx::query_as!(DbBlacklist, "SELECT * FROM blacklist")
        .fetch_all(pool)
        .await?)
}

pub async fn set_emoji(blacklist: DbBlacklist, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO blacklist(emoji) VALUES(?)",
        blacklist.emoji
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_emoji_by_emoji(emoji: &str, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM blacklist WHERE emoji == ?", emoji)
        .execute(pool)
        .await?;

    Ok(())
}
