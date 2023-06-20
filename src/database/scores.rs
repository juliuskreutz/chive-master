use anyhow::Result;
use chrono::NaiveDateTime;
use derive_getters::Getters;
use sqlx::SqlitePool;

#[derive(Getters)]
pub struct ScoreData {
    uid: i64,
    name: String,
    chives: i64,
    user: i64,
    timestamp: NaiveDateTime,
}

impl ScoreData {
    pub fn new(uid: i64, name: String, chives: i64, user: i64, timestamp: NaiveDateTime) -> Self {
        Self {
            uid,
            name,
            chives,
            user,
            timestamp,
        }
    }
}

pub async fn get_scores(pool: &SqlitePool) -> Result<Vec<ScoreData>> {
    Ok(sqlx::query_as!(ScoreData, "SELECT * FROM scores")
        .fetch_all(pool)
        .await?)
}

pub async fn get_score_by_uid(uid: i64, pool: &SqlitePool) -> Result<ScoreData> {
    Ok(
        sqlx::query_as!(ScoreData, "SELECT * FROM scores WHERE uid == ?1", uid)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn set_score(data: &ScoreData, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO scores(uid, name, chives, user, timestamp) VALUES(?, ?, ?, ?, ?)",
        data.uid,
        data.name,
        data.chives,
        data.user,
        data.timestamp
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_score_by_uid(uid: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM scores WHERE uid = ?1", uid)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_scores_by_user(user: i64, pool: &SqlitePool) -> Result<Vec<ScoreData>> {
    Ok(
        sqlx::query_as!(ScoreData, "SELECT * FROM scores WHERE user = ?1", user)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_scores_order_by_chives_desc_timestamp(
    pool: &SqlitePool,
) -> Result<Vec<ScoreData>> {
    Ok(sqlx::query_as!(
        ScoreData,
        "SELECT * FROM scores ORDER BY chives DESC, timestamp"
    )
    .fetch_all(pool)
    .await?)
}
