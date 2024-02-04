use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        Permissions,
    },
    prelude::{Context, Mentionable},
};
use sqlx::SqlitePool;

use crate::database::{self, RoleData};

pub const NAME: &str = "role";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "set" => set(ctx, command, pool).await,
        "delete" => delete(ctx, command, pool).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Role management")
        .create_option(|o| {
            o.name("set")
                .description("Set a role breakpoint")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("role")
                        .description("Role")
                        .kind(CommandOptionType::Role)
                        .required(true)
                })
                .create_sub_option(|so| {
                    so.name("chives")
                        .description("Chives")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_sub_option(|so| {
                    so.name("permanent")
                        .description("Permanent")
                        .kind(CommandOptionType::Boolean)
                        .required(true)
                })
        })
        .create_option(|o| {
            o.name("delete")
                .description("Delete a role breakpoint")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("role")
                        .description("Role")
                        .kind(CommandOptionType::Role)
                        .required(true)
                })
        })
        .default_member_permissions(Permissions::MANAGE_ROLES)
}

async fn set(
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

    let role_option = command.data.options[0].options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let chives_option = command.data.options[0].options[1]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let permanent_option = command.data.options[0].options[2]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Role(role) = role_option else {
        return Err(anyhow!("Not a role"));
    };

    let CommandDataOptionValue::Integer(chives) = *chives_option else {
        return Err(anyhow!("Not an integer"));
    };

    let CommandDataOptionValue::Boolean(permanent) = *permanent_option else {
        return Err(anyhow!("Not a boolean"));
    };

    let role_id = role.id.0 as i64;

    let data = RoleData::new(role_id, chives, permanent);

    database::set_role(&data, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!(
                "Added {} with breakpoint {}",
                role.mention(),
                chives
            ))
            .ephemeral(true)
        })
        .await?;

    Ok(())
}

async fn delete(
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

    let option = command.data.options[0].options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Role(role) = option else {
        return Err(anyhow!("Not a role"));
    };

    database::delete_role_by_role(role.id.0 as i64, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Deleted {}", role.mention()))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}
