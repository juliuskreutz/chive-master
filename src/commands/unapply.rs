use anyhow::{anyhow, Result};
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

use crate::database;

pub const NAME: &str = "unapply";

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

    if database::get_match_by_user(user, pool).await.is_ok() {
        return Err(anyhow!("You are already in a match!"));
    }

    if database::get_candidate_by_user(user, pool).await.is_err() {
        return Err(anyhow!("You are not matching!"));
    }

    database::delete_candidate_by_user(user, pool).await?;

    command
        .create_followup_message(ctx, |m| m.content("No longer matching").ephemeral(true))
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

    if database::get_match_by_user(user, pool).await.is_ok() {
        return Err(anyhow!("You are already in a match!"));
    }

    if database::get_candidate_by_user(user, pool).await.is_err() {
        return Err(anyhow!("You are not matching!"));
    }

    database::delete_candidate_by_user(user, pool).await?;

    interaction
        .create_followup_message(ctx, |m| m.content("No longer matching").ephemeral(true))
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(NAME).description("Stop matching :(")
}
