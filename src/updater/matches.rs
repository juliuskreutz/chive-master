use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use serenity::all::{
    ChannelType, CreateChannel, CreateMessage, Http, PermissionOverwrite, PermissionOverwriteType,
    Permissions, RoleId, UserId,
};
use sqlx::SqlitePool;

use crate::database;

pub async fn update(http: &Arc<Http>, pool: &SqlitePool) -> Result<()> {
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
                kind: PermissionOverwriteType::Role(RoleId::new(1008493665116758167)),
            },
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(UserId::new(user1 as u64)),
            },
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(UserId::new(user2 as u64)),
            },
        ];

        let name1 = UserId::new(user1 as u64).to_user(http).await?.name;
        let name2 = UserId::new(user2 as u64).to_user(http).await?.name;

        let channels = super::GUILD_ID.channels(http).await?;

        let mut matches_categories = Vec::new();

        for channel in channels.values() {
            if channel.kind == ChannelType::Category && channel.name == "💕 [matches] 💕" {
                matches_categories.push(channel.id.get());
            }
        }

        let mut channel = None;

        for category in matches_categories {
            if let Ok(ch) = super::GUILD_ID
                .create_channel(
                    http,
                    CreateChannel::new(format!("{name1} x {name2}"))
                        .category(category)
                        .permissions(permissions.clone()),
                )
                .await
            {
                channel = Some(ch);
                break;
            }
        }

        if channel.is_none() {
            let category = super::GUILD_ID
                .create_channel(
                    http,
                    CreateChannel::new("💕 [matches] 💕").kind(ChannelType::Category),
                )
                .await?;

            channel = Some(
                super::GUILD_ID
                    .create_channel(
                        http,
                        CreateChannel::new(format!("{name1} x {name2}"))
                            .category(category)
                            .permissions(permissions.clone()),
                    )
                    .await?,
            );
        }

        let channel = channel.unwrap();

        let db_match = database::DbMatch {
            channel: channel.id.get() as i64,
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
            .send_message(http, CreateMessage::new().content(text))
            .await?;

        database::delete_candidate_by_user(user1, pool).await?;
        database::delete_candidate_by_user(user2, pool).await?;
    }

    Ok(())
}
