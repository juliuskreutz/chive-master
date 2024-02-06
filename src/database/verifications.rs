use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub struct VerificationData {
    pub uid: i64,
    pub user: i64,
    pub name: String,
    pub otp: String,
    pub timestamp: NaiveDateTime,
}

impl VerificationData {
    pub fn new(uid: i64, user: i64, name: String, otp: String, timestamp: NaiveDateTime) -> Self {
        Self {
            uid,
            user,
            name,
            otp,
            timestamp,
        }
    }
}

pub async fn get_verifications(pool: &SqlitePool) -> Result<Vec<VerificationData>> {
    Ok(
        sqlx::query_as!(VerificationData, "SELECT * FROM verifications")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_verification_by_uid(uid: i64, pool: &SqlitePool) -> Result<VerificationData> {
    Ok(sqlx::query_as!(
        VerificationData,
        "SELECT * FROM verifications WHERE uid = ?1",
        uid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_verifications_by_user(
    user: i64,
    pool: &SqlitePool,
) -> Result<Vec<VerificationData>> {
    Ok(sqlx::query_as!(
        VerificationData,
        "SELECT * FROM verifications WHERE user == ?1",
        user
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_verifications_where_like(
    input: &str,
    pool: &SqlitePool,
) -> Result<Vec<VerificationData>> {
    Ok(sqlx::query_as!(
        VerificationData,
        "SELECT * FROM verifications WHERE uid LIKE ?1 OR name LIKE ?1 LIMIT 25",
        input
    )
    .fetch_all(pool)
    .await?)
}

pub async fn set_verification(data: &VerificationData, pool: &SqlitePool) -> Result<()> {
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
