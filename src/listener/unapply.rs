use anyhow::{anyhow, Result};
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

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(
        CreateCommand::new(name)
            .description("Stop matching :(")
            .dm_permission(false),
    );
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

    let user = command.user.id.get() as i64;

    if database::get_match_by_user(user, pool).await.is_ok() {
        return Err(anyhow!("You are already in a match!"));
    }

    if database::get_candidate_by_user(user, pool).await.is_err() {
        return Err(anyhow!("You are not matching!"));
    }

    database::delete_candidate_by_user(user, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("No longer matching")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub async fn component(
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

    if database::get_match_by_user(user, pool).await.is_ok() {
        return Err(anyhow!("You are already in a match!"));
    }

    if database::get_candidate_by_user(user, pool).await.is_err() {
        return Err(anyhow!("You are not matching!"));
    }

    database::delete_candidate_by_user(user, pool).await?;

    interaction
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("No longer matching")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
