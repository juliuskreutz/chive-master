use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction, InteractionResponseType,
            },
        },
        Permissions,
    },
    prelude::{Context, Mentionable},
};
use sqlx::SqlitePool;

use crate::database::{self, ChannelData};

pub const NAME: &str = "channel";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "enable" => enable(ctx, command, pool).await,
        "disable" => disable(ctx, command, pool).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Channel management")
        .create_option(|o| {
            o.name("enable")
                .description("Enable current channel as update channel")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("disable")
                .description("Disable current channel as update channel")
                .kind(CommandOptionType::SubCommand)
        })
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

async fn enable(
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

    if command.guild_id.is_none() {
        return Err(anyhow!("This command has to be in a guild"));
    }

    database::set_channel(ChannelData::new(command.channel_id.0 as i64), pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!(
                "Enabled {}. Make sure, that this channel has the right permissions.",
                command.channel_id.mention()
            ))
            .ephemeral(true)
        })
        .await?;

    Ok(())
}

async fn disable(
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

    database::delete_channel_by_channel(command.channel_id.0 as i64, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Disabled {}", command.channel_id.mention()))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}
