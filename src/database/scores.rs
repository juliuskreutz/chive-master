use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbConnection {
    pub uid: i64,
    pub user: i64,
}

pub async fn get_connections(pool: &SqlitePool) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_as!(DbConnection, "SELECT * FROM connections")
        .fetch_all(pool)
        .await?)
}

pub async fn get_score_by_uid(uid: i64, pool: &SqlitePool) -> Result<DbConnection> {
    Ok(sqlx::query_as!(
        DbConnection,
        "SELECT * FROM connections WHERE uid == ?1",
        uid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_uids(pool: &SqlitePool) -> Result<Vec<i64>> {
    Ok(sqlx::query!("SELECT uid FROM connections")
        .fetch_all(pool)
        .await
        .map(|r| r.into_iter().map(|r| r.uid))?
        .collect())
}

pub async fn set_score(data: &DbConnection, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO connections(uid, user) VALUES(?, ?)",
        data.uid,
        data.user,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_score_by_uid(uid: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM connections WHERE uid = ?1", uid)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_connections_by_user(user: i64, pool: &SqlitePool) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_as!(
        DbConnection,
        "SELECT * FROM connections WHERE user = ?1",
        user
    )
    .fetch_all(pool)
    .await?)
}
