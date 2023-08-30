use anyhow::Result;
use sqlx::SqlitePool;

pub struct DbShip {
    pub user: i64,
    pub ship: String,
}

pub struct DbShipStat {
    pub ship: String,
    pub votes: i64,
}

pub async fn set_ship(data: &DbShip, pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO ships(user, ship) VALUES(?, ?)",
        data.user,
        data.ship,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_ship_stats(pool: &SqlitePool) -> Result<Vec<DbShipStat>> {
    Ok(sqlx::query_as!(
        DbShipStat,
        "SELECT ship, COUNT(*) votes FROM ships GROUP BY ship ORDER BY votes DESC"
    )
    .fetch_all(pool)
    .await?)
}
