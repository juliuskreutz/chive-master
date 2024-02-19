use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, Mentionable, RoleId},
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "roles";

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let mut message = Vec::new();

    let roles = database::get_roles_order_by_chives_desc(pool).await?;

    for data in roles {
        message.push(format!(
            "{} - {} - {}",
            RoleId::new(data.role as u64).mention(),
            data.chives,
            if data.permanent {
                "Permanent"
            } else {
                "Exclusive"
            }
        ));
    }

    if message.is_empty() {
        return Err(anyhow!("No roles"));
    }

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .embed(CreateEmbed::new().description(message.join("\n")))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME).description("Role breakpoints")
}
