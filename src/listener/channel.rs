use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType, Mentionable},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub struct Channel;

impl super::Listener for Channel {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Channel management")
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "enable",
                "Enable current channel as update channel",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "disable",
                "Disable current channel as update channel",
            ))
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }

    async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
        match command.data.options[0].name.as_str() {
            "enable" => enable(ctx, command, pool).await,
            "disable" => disable(ctx, command, pool).await,
            _ => Err(anyhow!("Not a subcommand")),
        }
    }
}

async fn enable(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    if command.guild_id.is_none() {
        return Err(anyhow!("This command has to be in a guild"));
    }

    let channel = database::DbChannel {
        channel: command.channel_id.get() as i64,
    };
    database::set_channel(channel, pool).await?;

    // rewritten using new api
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "Enabled {}. Make sure, that this channel has the right permissions.",
                        command.channel_id.mention()
                    ))
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

async fn disable(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    database::delete_channel_by_channel(command.channel_id.get() as i64, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("Disabled {}", command.channel_id.mention()))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
