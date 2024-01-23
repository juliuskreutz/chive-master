use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::{database, stardb, updater};

pub const NAME: &str = "update";

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

    let connections = database::get_connections_by_user(user_id, pool).await?;

    if connections.is_empty() {
        return Err(anyhow!("You have no connected uids"));
    }

    for connection in &connections {
        stardb::put(connection.uid).await?;

        command
            .create_followup_message(ctx, |m| {
                m.content(format!("Updated {}", connection.uid))
                    .ephemeral(true)
            })
            .await?;
    }

    updater::update_user_roles(
        command.user.id.0 as i64,
        &mut HashSet::new(),
        &ctx.http,
        pool,
    )
    .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(NAME).description("Update connected uids")
}
