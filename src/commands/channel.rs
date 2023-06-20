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

use crate::database::{self, ChannelData};

pub const NAME: &str = "channel";

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
        .description("Channel management")
        .create_option(|o| {
            o.name("set")
                .description("Set an update channel")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("channel")
                        .description("Channel")
                        .kind(CommandOptionType::Channel)
                        .required(true)
                })
        })
        .create_option(|o| {
            o.name("delete")
                .description("Delete an update channel")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("channel")
                        .description("Channel")
                        .kind(CommandOptionType::Channel)
                        .required(true)
                })
        })
        .default_member_permissions(Permissions::ADMINISTRATOR)
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

    let option = command.data.options[0].options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Channel(channel) = option else {
        return Err(anyhow!("Not a channel"));
    };

    database::set_channel(ChannelData::new(channel.id.0 as i64), pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!(
                "Added {}. Make sure, that this channel has the right permissions.",
                channel.id.mention()
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

    let CommandDataOptionValue::Channel(channel) = option else {
        return Err(anyhow!("Not a channel"));
    };

    database::delete_channel_by_channel(channel.id.0 as i64, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Deleted {}", channel.id.mention()))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}
