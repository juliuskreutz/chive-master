use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbWarn {
    pub user: i64,
    pub moderator: i64,
    pub reason: String,
    pub dm: bool,
    pub message: Option<String>,
}

pub async fn set_warn(warn: DbWarn, pool: &SqlitePool) -> Result<i64> {
    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "INSERT OR REPLACE INTO warns(user, moderator, reason, dm, message) VALUES(?, ?, ?, ?, ?)",
        warn.user,
        warn.moderator,
        warn.reason,
        warn.dm,
        warn.message,
    )
    .execute(&mut *transaction)
    .await?;

    let id = sqlx::query!("SELECT LAST_INSERT_ROWID() as id")
        .fetch_one(&mut *transaction)
        .await?
        .id;

    transaction.commit().await?;

    Ok(id as i64)
}
