use std::sync::Arc;

use anyhow::Result;
use serenity::all::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, GetMessages, Http};
use sqlx::SqlitePool;

use crate::{database, stardb};

pub async fn update(http: &Arc<Http>, pool: &SqlitePool) -> Result<()> {
    let mut scores = Vec::new();

    for uid in database::get_uids(pool).await? {
        let score = stardb::get(uid).await?;

        scores.push(score);
    }

    scores.sort_unstable_by_key(|s| s.global_rank);

    let mut message1 = Vec::new();
    let mut message2 = Vec::new();

    for (i, data) in scores.iter().take(100).enumerate() {
        let place = i + 1;
        let achievement_count = data.achievement_count;
        let name = data.name.clone();

        if i < 50 {
            message1.push(format!(
                "**{place}** - {achievement_count} <:chive:1112854178302267452> - {name}",
            ));
        } else {
            message2.push(format!(
                "**{place}** - {achievement_count} <:chive:1112854178302267452> - {name}",
            ));
        }
    }

    let channels = database::get_channels(pool).await?;
    for channel in channels {
        if let Ok(messages) = ChannelId::new(channel.channel as u64)
            .messages(http, GetMessages::new().limit(2))
            .await
        {
            if let Some(message) = messages.first() {
                let _ = message.delete(http).await;
            }

            if let Some(message) = messages.get(1) {
                let _ = message.delete(http).await;
            }
        }

        if let Err(serenity::Error::Http(_)) = ChannelId::new(channel.channel as u64)
            .send_message(
                http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .color(0xFFD700)
                        .title("Leaderboard")
                        .description(message1.join("\n")),
                ),
            )
            .await
        {
            super::log(
                &format!(
                    "Error: Channel <#{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                    channel.channel
                ),
                http,
            )
            .await;

            database::delete_channel_by_channel(channel.channel, pool).await?;
            continue;
        }

        ChannelId::new(channel.channel as u64)
            .send_message(
                http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .color(0xFFD700)
                        .description(message2.join("\n"))
                        .footer(CreateEmbedFooter::new("Refreshes every 10 minutes")),
                ),
            )
            .await?;
    }

    Ok(())
}
