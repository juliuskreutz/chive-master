use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use serenity::{
    model::prelude::{ChannelId, GuildId, RoleId, UserId},
    CacheAndHttp,
};
use sqlx::SqlitePool;

use crate::{
    database::{self, DbConnection, RoleData},
    stardb,
};

pub fn init(cache: Arc<CacheAndHttp>, pool: SqlitePool) {
    tokio::spawn(async move {
        let minutes = 10;

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
            if let Err(e) = update_leaderboard(&cache, &pool).await {
                log(
                    &format!("Error: Leaderboard {} <@246684413075652612>", e),
                    &cache,
                )
                .await;
            }
            log(
                &format!("Updated leaderboard in {} seconds", now.elapsed().as_secs()),
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
                &format!("Completed update. Next update in {}min", minutes),
                &cache,
            )
            .await;
        }
    });
}

async fn update_verifications(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
    let verifications = database::get_verifications(pool).await?;
    for verification_data in verifications {
        let uid = verification_data.uid;

        let api_data = stardb::get(uid).await?;

        if !api_data.signature.ends_with(&verification_data.otp) {
            continue;
        }

        database::delete_verification_by_uid(uid, pool).await?;

        let user = verification_data.user;

        let score_data = DbConnection { uid, user };
        database::set_score(&score_data, pool).await?;

        if let Ok(channel) = UserId(user as u64).create_dm_channel(&cache).await {
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
        if let Ok(messages) = ChannelId(channel.channel as u64)
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

        if let Err(serenity::Error::Http(_)) = ChannelId(channel.channel as u64)
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
                    channel.channel
                ),
                cache,
            )
            .await;

            database::delete_channel_by_channel(channel.channel, pool).await?;
            continue;
        }

        ChannelId(channel.channel as u64)
            .send_message(&cache.http, |m| {
                m.embed(|e| {
                    e.color(0xFFD700)
                        .description(message2.join("\n"))
                        .footer(|f| f.text("Refreshes every 10 minutes"))
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
        guild_roles.entry(role.guild).or_default().push(role);
    }

    let mut c = HashSet::new();
    let mut d = HashSet::new();

    let scores = database::get_connections(pool).await?;
    for score in scores {
        for (&guild, roles) in &guild_roles {
            let uid = score.uid;
            let user = score.user;

            let key = (guild, user);
            if c.contains(&key) {
                continue;
            }
            c.insert(key);

            let Ok(mut member) = GuildId(guild as u64).member(&cache, user as u64).await else {
                continue;
            };

            let score = stardb::get(uid).await?;

            let Some(role_add) = roles.iter().find(|r| score.achievement_count >= r.chives) else {
                continue;
            };

            if let Err(serenity::Error::Http(_)) = member
                .add_role(&cache.http, RoleId(role_add.role as u64))
                .await
            {
                if d.insert(role_add.role) {
                    log(
                        &format!(
                            "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                            role_add.role
                        ),
                        cache,
                    )
                    .await;

                    database::delete_role_by_role(role_add.role, pool).await?;
                }
            }

            for role in &guild_roles[&guild] {
                if role.role == role_add.role {
                    continue;
                }

                if role.chives < 0 {
                    if let Err(serenity::Error::Http(_)) =
                        member.add_role(&cache.http, RoleId(role.role as u64)).await
                    {
                        if d.insert(role.role) {
                            log(
                            &format!(
                                "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                                role.role
                            ),
                            cache,
                        )
                        .await;

                            database::delete_role_by_role(role.role, pool).await?;
                        }
                    }

                    continue;
                }

                if let Err(serenity::Error::Http(_)) = member
                    .remove_role(&cache.http, RoleId(role.role as u64))
                    .await
                {
                    if d.insert(role.role) {
                        log(
                        &format!(
                            "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>",
                            role.role
                        ),
                        cache,
                    )
                    .await;
                        database::delete_role_by_role(role.role, pool).await?;
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
