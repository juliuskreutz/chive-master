use anyhow::Result;
use serenity::{
    all::{CommandInteraction, MemberAction},
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub fn register(name: &str) -> serenity::builder::CreateCommand {
    CreateCommand::new(name)
        .description("Bans")
        .default_member_permissions(Permissions::BAN_MEMBERS)
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
        )
        .await?;

    let message = database::get_bans(pool)
        .await?
        .into_iter()
        .map(|b| format!("<@{}> - {}", b.user, b.count))
        .collect::<Vec<_>>();

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .embed(
                    CreateEmbed::new()
                        .title("Bans")
                        .description(message.join("\n")),
                )
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub async fn ban_add(
    ctx: &Context,
    guild_id: &serenity::model::prelude::GuildId,
    user_id: &serenity::model::prelude::User,
    pool: &SqlitePool,
) -> Result<()> {
    use serenity::model::guild::audit_log::Action;

    let perpetrator = loop {
        let a = guild_id
            .audit_logs(
                &ctx,
                Some(Action::Member(MemberAction::BanAdd)),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        if !a.entries.is_empty()
            && a.entries[0].target_id.map(|id| id.get()) == Some(user_id.id.get())
        {
            break a.entries[0].user_id;
        }
    };

    let mut ban = database::get_ban_by_user(perpetrator.get() as i64, pool)
        .await
        .unwrap()
        .unwrap_or_else(|| database::DbBan {
            user: perpetrator.get() as i64,
            count: 0,
        });

    ban.count += 1;

    database::set_ban(ban, pool).await
}

pub async fn ban_remove(
    ctx: &Context,
    guild_id: &serenity::model::prelude::GuildId,
    user_id: &serenity::model::prelude::User,
    pool: &SqlitePool,
) -> Result<()> {
    use serenity::model::guild::audit_log::Action;

    let perpetrator = loop {
        let a = guild_id
            .audit_logs(
                &ctx,
                Some(Action::Member(MemberAction::BanRemove)),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        if !a.entries.is_empty()
            && a.entries[0].target_id.map(|id| id.get()) == Some(user_id.id.get())
        {
            break a.entries[0].user_id;
        }
    };

    let mut ban = database::get_ban_by_user(perpetrator.get() as i64, pool)
        .await
        .unwrap()
        .unwrap_or_else(|| database::DbBan {
            user: perpetrator.get() as i64,
            count: 0,
        });

    ban.count -= 1;

    database::set_ban(ban, pool).await
}
