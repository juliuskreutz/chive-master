use anyhow::Result;
use derive_getters::Getters;
use sqlx::SqlitePool;

#[derive(Getters)]
pub struct RoleData {
    role: i64,
    chives: i64,
    guild: i64,
}

impl RoleData {
    pub fn new(role: i64, chives: i64, guild: i64) -> Self {
        Self {
            role,
            chives,
            guild,
        }
    }
}

pub async fn set_role(data: &RoleData, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO roles(chives, role, guild) VALUES(?, ?, ?)",
        data.chives,
        data.role,
        data.guild
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

pub async fn get_roles_order_by_chives_desc(pool: &SqlitePool) -> Result<Vec<RoleData>> {
    Ok(
        sqlx::query_as!(RoleData, "SELECT * FROM roles ORDER BY chives DESC")
            .fetch_all(pool)
            .await?,
    )
}
