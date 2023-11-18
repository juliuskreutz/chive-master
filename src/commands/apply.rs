use anyhow::{anyhow, Result};
use chrono::Utc;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        message_component::MessageComponentInteraction,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database::{self, DbCandidate};

pub const NAME: &str = "apply";

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

    let user = command.user.id.0 as i64;

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

    let candidate = DbCandidate { user, timestamp };
    database::set_candidate(candidate, pool).await?;

    command.create_followup_message(ctx, |m| {
        m.content("Successfully applied for support matching. You will be notified, once we have found a good partner for you :D").ephemeral(true)
    })
    .await?;

    Ok(())
}

pub async fn component(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    let user = interaction.user.id.0 as i64;

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

    let candidate = DbCandidate { user, timestamp };
    database::set_candidate(candidate, pool).await?;

    interaction
        .create_followup_message(ctx, |m| {
            m.content("Successfully applied for support matching. You will be notified, once we have found a good partner for you :D").ephemeral(true)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(NAME).description("Start matching :D")
}
