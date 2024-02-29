use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbBan {
    pub user: i64,
    pub count: i64,
}

pub async fn get_ban_by_user(user: i64, pool: &SqlitePool) -> Result<Option<DbBan>> {
    Ok(
        sqlx::query_as!(DbBan, "SELECT * FROM bans WHERE user = ?", user)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get_bans(pool: &SqlitePool) -> Result<Vec<DbBan>> {
    Ok(
        sqlx::query_as!(DbBan, "SELECT * FROM bans ORDER BY count DESC")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn set_ban(ban: DbBan, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO bans(user, count) VALUES(?, ?)",
        ban.user,
        ban.count
    )
    .execute(pool)
    .await?;

    Ok(())
}
