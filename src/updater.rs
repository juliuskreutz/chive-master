use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use regex::{Captures, Regex};
use serenity::{
    model::prelude::{ChannelId, GuildId, RoleId, UserId},
    CacheAndHttp,
};
use sqlx::SqlitePool;

use crate::{
    api,
    database::{self, RoleData, ScoreData},
    timestamp,
};

pub fn init(cache: Arc<CacheAndHttp>, pool: SqlitePool) {
    let cache_clone = cache.clone();
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        loop {
            let now = Instant::now();
            if let Err(e) = update_leaderboard(&cache_clone, &pool_clone).await {
                log(
                    &format!("Error: Leaderboard {} <@246684413075652612>", e),
                    &cache_clone,
                )
                .await;
            }
            log(
                &format!("Updated leaderboard in {} seconds", now.elapsed().as_secs()),
                &cache_clone,
            )
            .await;
        }
    });

    tokio::spawn(async move {
        let minutes = 5;

        let mut timer = tokio::time::interval(Duration::from_secs(60 * minutes));

        loop {
            timer.tick().await;

            let now = Instant::now();
            if let Err(e) = update_verifications(&cache, &pool).await {
                log(
                    &format!("Error: Verifications {} <@246684413075652612>", e),
                    &cache,
                )
                .await;
            }
            log(
                &format!(
                    "Updated verifications in {} seconds",
                    now.elapsed().as_secs()
                ),
                &cache,
            )
            .await;

            let now = Instant::now();
            if let Err(e) = update_roles(&cache, &pool).await {
                log(&format!("Error: Roles {} <@246684413075652612>", e), &cache).await;
            }
            log(
                &format!("Updated roles in {} seconds", now.elapsed().as_secs()),
                &cache,
            )
            .await;

            log(
                &format!(
                    "Completed update of verifications and roles. Next update in {}min",
                    minutes
                ),
                &cache,
            )
            .await;
        }
    });
}

async fn update_verifications(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
    let verifications = database::get_verifications(pool).await?;
    for verification_data in verifications {
        let uid = *verification_data.uid();

        let api_data = api::get(uid).await?;

        if !api_data
            .player()
            .signature()
            .ends_with(verification_data.otp())
        {
            continue;
        }

        database::delete_verification_by_uid(uid, pool).await?;

        let name = api_data.player().nickname().clone();
        let chives = *api_data.player().space_info().achievement_count();
        let user_id = *verification_data.user();
        let date = timestamp::by_uid(uid)?;

        let score_data = ScoreData::new(uid, name, chives, user_id, date);
        database::set_score(&score_data, pool).await?;

        if let Ok(channel) = UserId(user_id as u64).create_dm_channel(&cache).await {
            let _ = channel
                .send_message(&cache.http, |m| {
                    m.content("You are now verified! You can change your HSR bio back :D")
                })
                .await;
        }
    }

    Ok(())
}

async fn update_leaderboard(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
    let now = Instant::now();

    let scores = database::get_scores(pool).await?;
    for score_data in scores {
        let uid = *score_data.uid();

        let Ok(api_data) = api::get(uid).await else {
            continue;
        };

        let name = api_data.player().nickname().clone();
        let chives = *api_data.player().space_info().achievement_count();
        let user = *score_data.user();
        let date = if chives != *score_data.chives() {
            timestamp::by_uid(uid)?
        } else {
            *score_data.timestamp()
        };

        let score_data = ScoreData::new(uid, name, chives, user, date);

        database::set_score(&score_data, pool).await?;
    }

    let scores = database::get_scores_order_by_chives_desc_timestamp(pool).await?;

    let mut message1 = Vec::new();
    let mut message2 = Vec::new();

    let re = Regex::new(r"<.*>(.*)</?.*>").unwrap();
    for (i, data) in scores.iter().take(100).enumerate() {
        let place = i + 1;
        let chives = *data.chives();
        let name = re
            .replace_all(data.name(), |c: &Captures| {
                c.get(1).unwrap().as_str().to_string()
            })
            .to_string();

        if i < 50 {
            message1.push(format!(
                "**{place}** - {chives} <:chive:1112854178302267452> - {name}",
            ));
        } else {
            message2.push(format!(
                "**{place}** - {chives} <:chive:1112854178302267452> - {name}",
            ));
        }
    }

    let channels = database::get_channels(pool).await?;
    for channel in channels {
        if let Ok(messages) = ChannelId(*channel.channel() as u64)
            .messages(&cache.http, |b| b.limit(2))
            .await
        {
            if let Some(message) = messages.get(0) {
                let _ = message.delete(&cache).await;
            }

            if let Some(message) = messages.get(1) {
                let _ = message.delete(&cache).await;
            }
        }

        if let Err(serenity::Error::Http(_)) = ChannelId(*channel.channel() as u64)
            .send_message(&cache.http, |m| {
                m.embed(|e| {
                    e.color(0xFFD700)
                        .title("Leaderboard")
                        .description(message1.join("\n"))
                })
            })
            .await
        {
            log(
                &format!(
                    "Error: Channel <#{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                    channel.channel()
                ),
                cache,
            )
            .await;

            database::delete_channel_by_channel(*channel.channel(), pool).await?;
            continue;
        }

        ChannelId(*channel.channel() as u64)
            .send_message(&cache.http, |m| {
                m.embed(|e| {
                    e.color(0xFFD700)
                        .description(message2.join("\n"))
                        .footer(|f| {
                            f.text(format!(
                                "Refreshes every {} minutes",
                                (now.elapsed().as_secs_f64() / 60.0).ceil()
                            ))
                        })
                })
            })
            .await?;
    }

    Ok(())
}

async fn update_roles(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
    let roles = database::get_roles_order_by_chives_desc(pool).await?;

    let mut guild_roles: HashMap<_, Vec<RoleData>> = HashMap::new();

    for role in roles {
        guild_roles.entry(*role.guild()).or_default().push(role);
    }

    let mut c = HashSet::new();
    let mut d = HashSet::new();

    let scores = database::get_scores(pool).await?;
    for score in scores {
        for (&guild, roles) in &guild_roles {
            let user = *score.user();

            let key = (guild, user);
            if c.contains(&key) {
                continue;
            }
            c.insert(key);

            let Ok(mut member) = GuildId(guild as u64).member(&cache, user as u64).await else {
                continue;
            };

            let Some(role_add) = roles.iter().find(|r| score.chives() >= r.chives()) else {
                continue;
            };

            if let Err(serenity::Error::Http(_)) = member
                .add_role(&cache.http, RoleId(*role_add.role() as u64))
                .await
            {
                if d.insert(role_add.role()) {
                    log(
                        &format!(
                            "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                            role_add.role()
                        ),
                        cache,
                    )
                    .await;

                    database::delete_role_by_role(*role_add.role(), pool).await?;
                }
            }

            for role in &guild_roles[&guild] {
                if role.role() == role_add.role() {
                    continue;
                }

                if *role.chives() < 0 {
                    if let Err(serenity::Error::Http(_)) = member
                        .add_role(&cache.http, RoleId(*role.role() as u64))
                        .await
                    {
                        if d.insert(role.role()) {
                            log(
                            &format!(
                                "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                                role.role()
                            ),
                            cache,
                        )
                        .await;

                            database::delete_role_by_role(*role.role(), pool).await?;
                        }
                    }

                    continue;
                }

                if let Err(serenity::Error::Http(_)) = member
                    .remove_role(&cache.http, RoleId(*role.role() as u64))
                    .await
                {
                    if d.insert(role.role()) {
                        log(
                        &format!(
                            "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                            role.role()
                        ),
                        cache,
                    )
                    .await;

                        database::delete_role_by_role(*role.role(), pool).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn log(content: &str, cache: &Arc<CacheAndHttp>) {
    ChannelId(1119634729377992774)
        .send_message(&cache.http, |m| m.content(content))
        .await
        .unwrap();
}
