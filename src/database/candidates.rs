use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub struct DbCandidate {
    pub user: i64,
    pub timestamp: NaiveDateTime,
}

pub async fn get_candidates(pool: &SqlitePool) -> Result<Vec<DbCandidate>> {
    Ok(
        sqlx::query_as!(DbCandidate, "SELECT * FROM candidates ORDER BY timestamp")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn set_candidate(candidate: DbCandidate, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO candidates(user, timestamp) VALUES(?, ?)",
        candidate.user,
        candidate.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_candidate_by_user(user: i64, pool: &SqlitePool) -> Result<DbCandidate> {
    Ok(
        sqlx::query_as!(DbCandidate, "SELECT * FROM candidates WHERE user = ?", user,)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn delete_candidate_by_user(user: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM candidates WHERE user = ?", user)
        .execute(pool)
        .await?;

    Ok(())
}
