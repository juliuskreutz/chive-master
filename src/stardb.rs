use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScoreAchievement {
    pub global_rank: usize,
    pub achievement_count: i64,
    pub name: String,
    pub signature: String,
}

pub async fn get(uid: i64) -> Result<ScoreAchievement> {
    Ok(
        if let Ok(score) = reqwest::get(&format!("https://stardb.gg/api/scores/achievements/{uid}"))
            .await?
            .json::<ScoreAchievement>()
            .await
        {
            score
        } else {
            reqwest::Client::new()
                .put(&format!("https://stardb.gg/api/scores/achievements/{uid}"))
                .send()
                .await?
                .json::<ScoreAchievement>()
                .await
                .map_err(|e| anyhow::anyhow!("{e}: {uid}"))?
        },
    )
}

pub async fn put(uid: i64) -> Result<ScoreAchievement> {
    reqwest::Client::new()
        .put(&format!("https://stardb.gg/api/scores/achievements/{uid}"))
        .send()
        .await?
        .json::<ScoreAchievement>()
        .await
        .map_err(|e| anyhow::anyhow!("{e}: {uid}"))
}
