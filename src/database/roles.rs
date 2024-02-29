use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbRole {
    pub role: i64,
    pub chives: i64,
    pub permanent: bool,
}

pub async fn set_role(data: &DbRole, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO roles(chives, role, permanent) VALUES(?, ?, ?)",
        data.chives,
        data.role,
        data.permanent,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_role_by_role(role: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM roles WHERE role == ?1", role)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_roles(pool: &SqlitePool) -> Result<Vec<DbRole>> {
    Ok(sqlx::query_as!(DbRole, "SELECT * FROM roles")
        .fetch_all(pool)
        .await?)
}

pub async fn get_roles_order_by_chives_desc(pool: &SqlitePool) -> Result<Vec<DbRole>> {
    Ok(
        sqlx::query_as!(DbRole, "SELECT * FROM roles ORDER BY chives DESC")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_exclusive_roles(pool: &SqlitePool) -> Result<Vec<DbRole>> {
    Ok(
        sqlx::query_as!(DbRole, "SELECT * FROM roles WHERE NOT permanent")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_permanent_roles_order_by_chives(pool: &SqlitePool) -> Result<Vec<DbRole>> {
    Ok(
        sqlx::query_as!(DbRole, "SELECT * FROM roles WHERE permanent")
            .fetch_all(pool)
            .await?,
    )
}
