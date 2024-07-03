use anyhow::Result;
use reqwest::header;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScoreAchievement {
    pub achievement_count: i64,
    pub signature: String,
}

pub async fn get(uid: i64) -> Result<ScoreAchievement> {
    if let Ok(response) = reqwest::get(&format!(
        "http://localhost:8000/api/scores/achievements/{uid}"
    ))
    .await
    {
        if let Ok(sa) = response.json::<ScoreAchievement>().await {
            return Ok(sa);
        }
    }

    put(uid).await
}

pub async fn put(uid: i64) -> Result<ScoreAchievement> {
    if let Ok(response) = reqwest::Client::new()
        .put(&format!(
            "http://localhost:8000/api/scores/achievements/{uid}"
        ))
        .send()
        .await
    {
        if let Ok(sa) = response.json::<ScoreAchievement>().await {
            return Ok(sa);
        }
    }

    let value: serde_json::Value = reqwest::Client::new()
        .get("https://enka.network/api/hsr/uid/{uid}")
        .header(header::USER_AGENT, "stardb")
        .send()
        .await?
        .json()
        .await?;

    let achievement_count = value["detailInfo"]["recordInfo"]["achievementCount"]
        .as_i64()
        .unwrap();

    let signature = value["detailInfo"]["signature"]
        .as_str()
        .unwrap()
        .to_string();

    Ok(ScoreAchievement {
        achievement_count,
        signature,
    })
}
