use anyhow::{anyhow, Result};
use chrono::Utc;
use serenity::{
    all::{CommandInteraction, ComponentInteraction},
    builder::{
        CreateCommand, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub struct Apply;

impl super::Listener for Apply {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name).description("Start matching :D")
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

        let user = command.user.id.get() as i64;

        if database::get_connections_by_user(user, pool)
            .await?
            .is_empty()
        {
            return Err(anyhow!("You are not verified. Please head to https://discord.com/channels/1008493665116758167/1138771945517764608"));
        }

        if database::get_match_by_user(user, pool).await.is_ok() {
            return Err(anyhow!("You are already in a match!"));
        }

        if database::get_candidate_by_user(user, pool).await.is_ok() {
            return Err(anyhow!("You are already matching!"));
        }

        let timestamp = Utc::now().naive_utc();

        let candidate = database::DbCandidate { user, timestamp };
        database::set_candidate(candidate, pool).await?;

        command.create_followup(&ctx, CreateInteractionResponseFollowup::new().content("Successfully applied for support matching. You will be notified, once we have found a good partner for you :D").ephemeral(true)).await?;

        Ok(())
    }

    async fn component(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        let user = interaction.user.id.get() as i64;

        if database::get_connections_by_user(user, pool)
            .await?
            .is_empty()
        {
            return Err(anyhow!("You are not verified. Please head to https://discord.com/channels/1008493665116758167/1138771945517764608"));
        }

        if database::get_match_by_user(user, pool).await.is_ok() {
            return Err(anyhow!("You are already in a match!"));
        }

        if database::get_candidate_by_user(user, pool).await.is_ok() {
            return Err(anyhow!("You are already matching!"));
        }

        let timestamp = Utc::now().naive_utc();

        let candidate = database::DbCandidate { user, timestamp };
        database::set_candidate(candidate, pool).await?;

        interaction.create_followup(&ctx, CreateInteractionResponseFollowup::new().content("Successfully applied for support matching. You will be notified, once we have found a good partner for you :D").ephemeral(true)).await?;

        Ok(())
    }
}
