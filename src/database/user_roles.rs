use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbUserRole {
    pub user: i64,
    pub role: i64,
}

pub async fn set_user_role(data: &DbUserRole, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO user_roles(user, role) VALUES(?, ?)",
        data.user,
        data.role,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_roles_by_user(user: i64, pool: &SqlitePool) -> Result<Vec<DbUserRole>> {
    Ok(
        sqlx::query_as!(DbUserRole, "SELECT * FROM user_roles WHERE user == ?", user)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn delete_user_roles_by_user(user: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM user_roles WHERE user == ?", user)
        .execute(pool)
        .await?;

    Ok(())
}
