use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
        RoleId,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "unregister";

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

    let option = command.data.options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Integer(uid) = *option else {
        return Err(anyhow!("Not an integer"));
    };

    let user = command.user.id.0 as i64;
    let scores = database::get_connections_by_user(user, pool).await?;

    if !scores.iter().any(|s| s.uid == uid) {
        return Err(anyhow!("This uid is not connected to your account"));
    }

    database::delete_connection_by_uid(uid, pool).await?;

    if let Some(mut member) = command.member.clone() {
        let roles = database::get_roles(pool).await?;

        for role in roles {
            let _ = member.remove_role(&ctx, RoleId(role.role as u64)).await;
        }
    }

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Successfully unregistered {uid}"))
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Unregister your uid")
        .create_option(|o| {
            o.name("uid")
                .description("Your uid")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
}
