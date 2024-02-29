use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serenity::{
    all::CommandInteraction,
    builder::{
        CreateCommand, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::{database, stardb, updater};

pub struct Update;

impl super::Listener for Update {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name).description("Update connected uids")
    }

    async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
        command
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        let user_id = command.user.id.get() as i64;

        let connections = database::get_connections_by_user(user_id, pool).await?;

        if connections.is_empty() {
            return Err(anyhow!("You have no connected uids"));
        }

        for connection in &connections {
            stardb::put(connection.uid).await?;

            command
                .create_followup(
                    &ctx,
                    CreateInteractionResponseFollowup::new()
                        .content(format!("Updated {}", connection.uid))
                        .ephemeral(true),
                )
                .await?;
        }

        updater::update_user_roles(
            command.user.id.get() as i64,
            &mut HashSet::new(),
            &ctx.http,
            pool,
        )
        .await?;

        Ok(())
    }
}
