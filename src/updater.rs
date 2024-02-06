use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use serenity::{
    http::Http,
    model::{
        prelude::{
            ChannelId, ChannelType, GuildId, Member, PermissionOverwrite, PermissionOverwriteType,
            RoleId, UserId,
        },
        Permissions,
    },
    CacheAndHttp,
};
use sqlx::SqlitePool;
use tokio::time;

use crate::{
    database::{self, DbConnection, DbMatch},
    stardb,
};

const GUILD_ID: GuildId = GuildId(1008493665116758167);

pub fn init(cache: Arc<CacheAndHttp>, pool: SqlitePool) {
    {
        let cache = cache.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                let cache = cache.clone();
                let pool = pool.clone();

                let task = tokio::spawn(async move {
                    let mut timer = time::interval(Duration::from_secs(5 * 60));

                    loop {
                        timer.tick().await;

                        let now = Instant::now();
                        if let Err(e) = update_verifications(&cache, &pool).await {
                            log(
                                &format!("Error: Verifications {} <@246684413075652612>", e),
                                &cache.http,
                            )
                            .await;
                        }
                        log(
                            &format!(
                                "Updated verifications in {} seconds",
                                now.elapsed().as_secs()
                            ),
                            &cache.http,
                        )
                        .await;

                        let now = Instant::now();
                        if let Err(e) = update_leaderboard(&cache, &pool).await {
                            log(
                                &format!("Error: Leaderboard {} <@246684413075652612>", e),
                                &cache.http,
                            )
                            .await;
                        }
                        log(
                            &format!("Updated leaderboard in {} seconds", now.elapsed().as_secs()),
                            &cache.http,
                        )
                        .await;

                        let now = Instant::now();
                        if let Err(e) = update_matches(&cache, &pool).await {
                            log(
                                &format!("Error: Matches {} <@246684413075652612>", e),
                                &cache.http,
                            )
                            .await;
                        }
                        log(
                            &format!("Updated matches in {} seconds", now.elapsed().as_secs()),
                            &cache.http,
                        )
                        .await;
                    }
                });

                let _ = task.await;
            }
        });
    }

    tokio::spawn(async move {
        loop {
            let cache = cache.clone();
            let pool = pool.clone();

            let task = tokio::spawn(async move {
                let now = Instant::now();
                if let Err(e) = update_roles(&cache, &pool).await {
                    log(
                        &format!("Error: Roles {} <@246684413075652612>", e),
                        &cache.http,
                    )
                    .await;
                }
                log(
                    &format!("Updated roles in {} seconds", now.elapsed().as_secs()),
                    &cache.http,
                )
                .await;
            });

            let _ = task.await;
        }
    });
}

async fn update_verifications(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
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

        let score_data = DbConnection { uid, user };
        database::set_connection(&score_data, pool).await?;

        update_user_roles(user, &mut HashSet::new(), &cache.http, pool).await?;

        if let Ok(channel) = UserId(user as u64).create_dm_channel(&cache).await {
            let _ = channel
                .send_message(&cache.http, |m| {
                    m.content("Congratulations Completionist! You are now @Chive Verified and your profile will appear on the Chive Leaderboards: https://stardb.gg/leaderboard. You can change your HSR bio back to what it was originally. Additionally, you've gained access to the https://discord.com/channels/1008493665116758167/1108110331043123200 channel.")
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
            if let Some(message) = messages.first() {
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
                &cache.http,
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
    let mut d = HashSet::new();

    let users = database::get_users(pool).await?;
    for user in users {
        update_user_roles(user, &mut d, &cache.http, pool).await?;
    }

    Ok(())
}

pub async fn update_user_roles(
    user: i64,
    d: &mut HashSet<i64>,
    http: &Arc<Http>,
    pool: &SqlitePool,
) -> Result<()> {
    let Ok(mut member) = GUILD_ID.member(http, user as u64).await else {
        return Ok(());
    };

    let connections = database::get_connections_by_user(user, pool).await?;

    let mut score = stardb::get(connections[0].uid).await?;
    for connection in connections.iter().skip(1) {
        let s = stardb::get(connection.uid).await?;

        if s.achievement_count > score.achievement_count {
            score = s;
        }
    }

    let permanent_roles = database::get_permanent_roles_order_by_chives(pool).await?;

    for role in &permanent_roles {
        if score.achievement_count >= role.chives {
            add_member_role(&mut member, role.role, d, http, pool).await?;
        }
    }

    let Some(role_add) = database::get_roles_order_by_chives_desc(pool)
        .await?
        .into_iter()
        .find(|r| score.achievement_count >= r.chives)
    else {
        return Ok(());
    };

    add_member_role(&mut member, role_add.role, d, http, pool).await?;

    for role in database::get_exclusive_roles(pool).await? {
        if role.role == role_add.role {
            continue;
        }

        remove_member_role(&mut member, role.role, d, http, pool).await?;
    }

    Ok(())
}

async fn add_member_role(
    member: &mut Member,
    role: i64,
    d: &mut HashSet<i64>,
    http: &Arc<Http>,
    pool: &SqlitePool,
) -> Result<()> {
    if let Err(serenity::Error::Http(_)) = member.add_role(&http, RoleId(role as u64)).await {
        if d.insert(role) {
            log(&format!( "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>", role), http) .await;

            database::delete_role_by_role(role, pool).await?;
        }
    }

    Ok(())
}

async fn remove_member_role(
    member: &mut Member,
    role: i64,
    d: &mut HashSet<i64>,
    http: &Arc<Http>,
    pool: &SqlitePool,
) -> Result<()> {
    if let Err(serenity::Error::Http(_)) = member.remove_role(&http, RoleId(role as u64)).await {
        if d.insert(role) {
            log( &format!( "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>", role), http) .await;

            database::delete_role_by_role(role, pool).await?;
        }
    }

    Ok(())
}

async fn update_matches(cache: &Arc<CacheAndHttp>, pool: &SqlitePool) -> Result<()> {
    let candidates = database::get_candidates(pool).await?;

    let mut c = HashSet::new();

    for candidate in &candidates {
        let user1 = candidate.user;

        if c.contains(&user1) {
            continue;
        }

        c.insert(user1);

        let connections1 = database::get_connections_by_user(user1, pool).await?;

        let user2 = {
            let mut o = None;

            for candidate in &candidates {
                let user2 = candidate.user;

                if c.contains(&user2) {
                    continue;
                }

                let connections2 = database::get_connections_by_user(user2, pool).await?;

                if connections1.iter().any(|c1| {
                    connections2
                        .iter()
                        .any(|c2| c1.uid / 100000000 == c2.uid / 100000000)
                }) {
                    o = Some(user2);
                    break;
                }
            }

            if let Some(user2) = o {
                user2
            } else {
                continue;
            }
        };

        c.insert(user2);

        let permissions = vec![
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                kind: PermissionOverwriteType::Role(RoleId(1008493665116758167)),
            },
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(UserId(user1 as u64)),
            },
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(UserId(user2 as u64)),
            },
        ];

        let name1 = UserId(user1 as u64).to_user(&cache.http).await?.name;
        let name2 = UserId(user2 as u64).to_user(&cache.http).await?.name;

        let channels = GUILD_ID.channels(&cache.http).await?;

        let mut matches_categories = Vec::new();

        for channel in channels.values() {
            if channel.kind == ChannelType::Category && channel.name == "💕 [matches] 💕" {
                matches_categories.push(channel.id.0);
            }
        }

        let mut channel = None;

        for category in matches_categories {
            if let Ok(ch) = GUILD_ID
                .create_channel(&cache.http, |c| {
                    c.name(format!("{name1} x {name2}"))
                        .category(category)
                        .permissions(permissions.clone())
                })
                .await
            {
                channel = Some(ch);
                break;
            }
        }

        if channel.is_none() {
            let category = GUILD_ID
                .create_channel(&cache.http, |c| {
                    c.name("💕 [matches] 💕").kind(ChannelType::Category)
                })
                .await?;

            channel = Some(
                GUILD_ID
                    .create_channel(&cache.http, |c| {
                        c.name(format!("{name1} x {name2}"))
                            .category(category)
                            .permissions(permissions)
                    })
                    .await?,
            );
        }

        let channel = channel.unwrap();

        let db_match = DbMatch {
            channel: channel.id.0 as i64,
            user1,
            user2,
        };

        database::set_match(&db_match, pool).await?;

        let text = format!("
<@{user1}> <@{user2}>
Amazing! Your support contractor has been found. Please use this channel to
1. Add each other to friend list
2. Agree on which support unit to provide to the other
3. Ensure you know the two correct ways to give credits to each other by reading the guide https://stardb.gg/articles/how-to-get-credits-from-supports/
4. Agree on assisting each other 10 times per day or unless otherwise agreed upon
5. If one party is unresponsive for more than 24hrs, then ping a staff member and we can unmatch you
");

        channel
            .send_message(&cache.http, |m| m.content(text))
            .await?;

        database::delete_candidate_by_user(user1, pool).await?;
        database::delete_candidate_by_user(user2, pool).await?;
    }

    Ok(())
}

pub async fn log(content: &str, http: &Arc<Http>) {
    ChannelId(1119634729377992774)
        .send_message(&http, |m| m.content(content))
        .await
        .unwrap();
}
