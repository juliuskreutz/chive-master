mod hsr_posts;
mod matches;
mod roles;
mod verifications;
mod zzz_posts;

use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use serenity::{
    all::{ChannelId, Member, RoleId},
    builder::CreateMessage,
    http::Http,
};
use sqlx::SqlitePool;
use tokio::time;

use crate::{database, stardb, GUILD_ID};

pub fn init(http: Arc<Http>, pool: SqlitePool) {
    {
        let http = http.clone();

        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                let http = http.clone();
                let pool = pool.clone();

                let task = tokio::spawn(async move {
                    let mut timer = time::interval(Duration::from_secs(5 * 60));

                    loop {
                        timer.tick().await;

                        let now = Instant::now();
                        if let Err(e) = hsr_posts::update(&http, &pool).await {
                            log(
                                &format!("Error: Hsr posts {} <@246684413075652612>", e),
                                &http,
                            )
                            .await;
                        }
                        log(
                            &format!("Updated hsr posts in {} seconds", now.elapsed().as_secs()),
                            &http,
                        )
                        .await;

                        let now = Instant::now();
                        if let Err(e) = zzz_posts::update(&http, &pool).await {
                            log(
                                &format!("Error: Zzz posts {} <@246684413075652612>", e),
                                &http,
                            )
                            .await;
                        }
                        log(
                            &format!("Updated zzz posts in {} seconds", now.elapsed().as_secs()),
                            &http,
                        )
                        .await;

                        let now = Instant::now();
                        if let Err(e) = verifications::update(&http, &pool).await {
                            log(
                                &format!("Error: Verifications {} <@246684413075652612>", e),
                                &http,
                            )
                            .await;
                        }
                        log(
                            &format!(
                                "Updated verifications in {} seconds",
                                now.elapsed().as_secs()
                            ),
                            &http,
                        )
                        .await;

                        let now = Instant::now();
                        if let Err(e) = matches::update(&http, &pool).await {
                            log(
                                &format!("Error: Matches {} <@246684413075652612>", e),
                                &http,
                            )
                            .await;
                        }
                        log(
                            &format!("Updated matches in {} seconds", now.elapsed().as_secs()),
                            &http,
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
            let http = http.clone();
            let pool = pool.clone();

            let task = tokio::spawn(async move {
                let now = Instant::now();
                if let Err(e) = roles::update(&http, &pool).await {
                    log(&format!("Error: Roles {} <@246684413075652612>", e), &http).await;
                }
                log(
                    &format!("Updated roles in {} seconds", now.elapsed().as_secs()),
                    &http,
                )
                .await;
            });

            let _ = task.await;
        }
    });
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
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
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
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    for role in database::get_exclusive_roles(pool).await? {
        if role.role == role_add.role {
            continue;
        }

        remove_member_role(&mut member, role.role, d, http, pool).await?;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
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
    if d.contains(&role) {
        return Ok(());
    }

    let mut i = 0;

    while i < 5 {
        if member
            .add_role(http, RoleId::new(role as u64))
            .await
            .is_err()
        {
            if GUILD_ID
                .roles(http)
                .await?
                .get(&RoleId::new(role as u64))
                .is_none()
            {
                i += 1;
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            return Ok(());
        }
    }

    d.insert(role);
    log(&format!( "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>", role), http) .await;

    database::delete_role_by_role(role, pool).await?;

    Ok(())
}

async fn remove_member_role(
    member: &mut Member,
    role: i64,
    d: &mut HashSet<i64>,
    http: &Arc<Http>,
    pool: &SqlitePool,
) -> Result<()> {
    if member
        .remove_role(http, RoleId::new(role as u64))
        .await
        .is_err()
        && d.insert(role)
    {
        log( &format!( "Error: Role <@&{}>. Wrong permissions or doesn't exists. Deleting! <@246684413075652612>", role), http) .await;

        database::delete_role_by_role(role, pool).await?;
    }

    Ok(())
}

pub async fn log(content: &str, http: &Arc<Http>) {
    ChannelId::new(1119634729377992774)
        .send_message(http, CreateMessage::new().content(content))
        .await
        .unwrap();
}
