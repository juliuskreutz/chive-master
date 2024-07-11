use std::{collections::HashSet, sync::Arc, time::Duration};

use anyhow::Result;
use serenity::all::{CreateMessage, Http, UserId};
use sqlx::SqlitePool;

use crate::{database, stardb};

pub async fn update(http: &Arc<Http>, pool: &SqlitePool) -> Result<()> {
    let verifications = database::get_verifications(pool).await?;
    for verification_data in verifications {
        if verification_data.timestamp + chrono::Duration::days(1) < chrono::Utc::now().naive_utc()
        {
            database::delete_verification_by_uid(verification_data.uid, pool).await?;
            continue;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;

        let uid = verification_data.uid;

        let score = stardb::put(uid).await?;

        if !score.signature.ends_with(&verification_data.otp) {
            continue;
        }

        database::delete_verification_by_uid(uid, pool).await?;

        let user = verification_data.user;

        let score_data = database::DbConnection { uid, user };
        database::set_connection(&score_data, pool).await?;

        super::update_user_roles(user, &mut HashSet::new(), http, pool).await?;

        if let Ok(channel) = UserId::new(user as u64).create_dm_channel(http).await {
            let _ = channel
                .send_message(http, CreateMessage::new().content("Congratulations Completionist! You are now @Chive Verified and your profile will appear on the Chive Leaderboards: https://stardb.gg/leaderboard. You can change your HSR bio back to what it was originally.")
                )
                .await;
        }
    }

    Ok(())
}
