use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScoreAchivement {
    pub global_rank: usize,
    pub achievement_count: i64,
    pub name: String,
    pub signature: String,
}

pub async fn get(uid: i64) -> Result<ScoreAchivement> {
    Ok(
        if let Ok(score) = reqwest::get(&format!(
            "http://localhost:8000/api/scores/achievements/{uid}"
        ))
        .await?
        .json::<ScoreAchivement>()
        .await
        {
            score
        } else {
            reqwest::Client::new()
                .put(&format!(
                    "http://localhost:8000/api/scores/achievements/{uid}"
                ))
                .send()
                .await?;

            reqwest::get(&format!(
                "http://localhost:8000/api/scores/achievements/{uid}"
            ))
            .await?
            .json::<ScoreAchivement>()
            .await?
        },
    )
}
