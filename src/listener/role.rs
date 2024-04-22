use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, Mentionable},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub fn register(name: &str) -> CreateCommand {
    CreateCommand::new(name)
        .description("Role management")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "set",
                "Set a role breakpoint",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Role, "role", "Role").required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "chives", "Chives")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Boolean, "permanent", "Permanent")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "delete",
                "Delete a role breakpoint",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Role, "role", "Role").required(true),
            ),
        )
        .default_member_permissions(Permissions::MANAGE_ROLES)
        .dm_permission(false)
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "set" => set(ctx, command, pool).await,
        "delete" => delete(ctx, command, pool).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

async fn set(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let CommandDataOptionValue::SubCommand(options) = &command.data.options[0].value else {
        return Err(anyhow!("Not a subcommand"));
    };

    let role_id = options[0].value.as_role_id().unwrap();
    let chives = options[1].value.as_i64().unwrap();
    let permanent = options[2].value.as_bool().unwrap();

    let role = database::DbRole {
        role: role_id.get() as i64,
        chives,
        permanent,
    };
    database::set_role(&role, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!(
                    "Added {} with breakpoint {}",
                    role_id.mention(),
                    chives
                ))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn delete(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let CommandDataOptionValue::SubCommand(options) = &command.data.options[0].value else {
        return Err(anyhow!("Not a subcommand"));
    };

    let role_id = options[0].value.as_role_id().unwrap();

    database::delete_role_by_role(role_id.get() as i64, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("Deleted {}", role_id.mention()))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
