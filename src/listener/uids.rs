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

use crate::database;

pub struct Uids;

impl super::Listener for Uids {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name).description("Get connected uids")
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

        for connection in connections {
            command
                .create_followup(
                    &ctx,
                    CreateInteractionResponseFollowup::new()
                        .content(connection.uid.to_string())
                        .ephemeral(true),
                )
                .await?;
        }

        Ok(())
    }
}
