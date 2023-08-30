use anyhow::Result;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        Permissions,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "shipstats";

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

    let ship_stats = database::get_ship_stats(pool).await?;

    let total: usize = ship_stats.iter().map(|ss| ss.votes as usize).sum();

    let message = ship_stats
        .into_iter()
        .map(|ss| format!("{}|{}", ss.ship, ss.votes))
        .collect::<Vec<_>>()
        .join("\n");

    command
        .create_followup_message(ctx, |m| {
            m.content(format!(
                "```Total: {total}\n------------------\n{message}```"
            ))
            .ephemeral(true)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("See the current ship standings")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
