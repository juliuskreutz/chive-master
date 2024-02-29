use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType, RoleId},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub struct Unregister;

impl super::Listener for Unregister {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Unregister your uid")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "uid", "Your uid")
                    .required(true),
            )
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

        let uid = command.data.options[0].value.as_i64().unwrap();

        let user = command.user.id.get() as i64;
        let scores = database::get_connections_by_user(user, pool).await?;

        if !scores.iter().any(|s| s.uid == uid) {
            return Err(anyhow!("This uid is not connected to your account"));
        }

        database::delete_connection_by_uid(uid, pool).await?;

        if let Some(member) = command.member.clone() {
            let roles = database::get_roles(pool).await?;

            for role in roles {
                let _ = member
                    .remove_role(&ctx, RoleId::new(role.role as u64))
                    .await;
            }
        }

        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new().content("Successfully unregistered uid"),
            )
            .await?;

        Ok(())
    }
}
