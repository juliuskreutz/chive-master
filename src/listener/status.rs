use anyhow::{anyhow, Result};
use serenity::{
    all::CommandInteraction,
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub struct Status;

impl super::Listener for Status {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name).description("Verification status")
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

        let verifications = database::get_verifications_by_user(user_id, pool).await?;

        if verifications.is_empty() {
            return Err(anyhow!("You have no pending verifications"));
        }

        let embed = CreateEmbed::new()
            .title("Pending Verifications")
            .description("Listed below are your uids and their respective code, which you'll have to append in your comment section of the game.")
            .fields(
                verifications
                    .iter()
                    .map(|verification| {
                        (
                            verification.uid.to_string(),
                            format!("Code: **{}**", verification.otp),
                            false,
                        )
                    })
            );

        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new()
                    .embed(embed)
                    .ephemeral(true),
            )
            .await?;

        Ok(())
    }
}
