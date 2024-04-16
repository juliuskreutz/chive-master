use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbPost {
    pub id: i64,
}

pub async fn get_post_by_id(id: i64, pool: &SqlitePool) -> Result<DbPost> {
    Ok(
        sqlx::query_as!(DbPost, "SELECT * FROM posts WHERE id = ?", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn set_post(post: DbPost, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("INSERT OR REPLACE INTO posts(id) VALUES(?)", post.id)
        .execute(pool)
        .await?;

    Ok(())
}
