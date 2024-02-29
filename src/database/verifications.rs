use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub struct DbVerification {
    pub uid: i64,
    pub user: i64,
    pub name: String,
    pub otp: String,
    pub timestamp: NaiveDateTime,
}

pub async fn get_verifications(pool: &SqlitePool) -> Result<Vec<DbVerification>> {
    Ok(
        sqlx::query_as!(DbVerification, "SELECT * FROM verifications")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_verification_by_uid(uid: i64, pool: &SqlitePool) -> Result<DbVerification> {
    Ok(sqlx::query_as!(
        DbVerification,
        "SELECT * FROM verifications WHERE uid = ?1",
        uid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_verifications_by_user(
    user: i64,
    pool: &SqlitePool,
) -> Result<Vec<DbVerification>> {
    Ok(sqlx::query_as!(
        DbVerification,
        "SELECT * FROM verifications WHERE user == ?1",
        user
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_verifications_where_like(
    input: &str,
    pool: &SqlitePool,
) -> Result<Vec<DbVerification>> {
    Ok(sqlx::query_as!(
        DbVerification,
        "SELECT * FROM verifications WHERE uid LIKE ?1 OR name LIKE ?1 LIMIT 25",
        input
    )
    .fetch_all(pool)
    .await?)
}

pub async fn set_verification(data: &DbVerification, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO verifications(uid, user, name, otp, timestamp) VALUES(?, ?, ?, ?, ?)",
        data.uid,
        data.user,
        data.name,
        data.otp,
        data.timestamp
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_verification_by_uid(uid: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM verifications WHERE uid == ?1", uid)
        .execute(pool)
        .await?;

    Ok(())
}
