use anyhow::Result;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

const URL: &str = "https://api.mihomo.me/sr_info_parsed/%uid%?lang=en&version=v2";

#[derive(Serialize, Deserialize, Getters)]
pub struct ApiData {
    player: Player,
}

#[derive(Serialize, Deserialize, Getters)]
pub struct Player {
    nickname: String,
    signature: String,
    space_info: SpaceInfo,
}

#[derive(Serialize, Deserialize, Getters)]
pub struct SpaceInfo {
    achievement_count: i64,
}

pub async fn get(uid: i64) -> Result<ApiData> {
    Ok(reqwest::get(&URL.replace("%uid%", &uid.to_string()))
        .await?
        .json::<ApiData>()
        .await?)
}
