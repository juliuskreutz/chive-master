use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        RoleId,
    },
    prelude::{Context, Mentionable},
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "roles";

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

    let mut message = Vec::new();

    let roles = database::get_roles_order_by_chives_desc(pool).await?;

    for data in roles {
        message.push(format!(
            "{} - {}",
            RoleId(data.role as u64).mention(),
            data.chives
        ));
    }

    if message.is_empty() {
        return Err(anyhow!("No roles"));
    }

    command
        .create_followup_message(ctx, |m| {
            m.embed(|e| e.description(message.join("\n")))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(NAME).description("Role breakpoints")
}
