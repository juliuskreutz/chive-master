use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub fn register(name: &str) -> CreateCommand {
    CreateCommand::new(name)
        .description("Get connected uids of a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "User").required(true),
        )
        .default_member_permissions(Permissions::MANAGE_NICKNAMES)
        .dm_permission(false)
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let user_id = command.data.options[0].value.as_user_id().unwrap().get() as i64;

    let connections = database::get_connections_by_user(user_id, pool).await?;

    if connections.is_empty() {
        return Err(anyhow!("This user has no connected uids"));
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
