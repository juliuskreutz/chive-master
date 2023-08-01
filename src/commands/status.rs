use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "status";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    let user_id = command.user.id.0 as i64;

    let verifications = database::get_verifications_by_user(user_id, pool).await?;

    if verifications.is_empty() {
        return Err(anyhow!("You have no pending verifications"));
    }

    command
        .create_followup_message(ctx, |m| {
            m.embed(|e| {
                let mut e = e.title("Pending Verifications").description("Listed below are your uids and their respective code, which you'll have to append in your comment section of the game.");

                for verification in verifications {
                    e = e.field(
                        verification.uid,
                        format!("Code: **{}**", verification.otp),
                        false,
                    );
                }

                e
            }).ephemeral(true)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(NAME).description("Verification status")
}
