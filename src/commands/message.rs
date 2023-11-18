use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            component::ButtonStyle,
            interaction::{
                application_command::ApplicationCommandInteraction, InteractionResponseType,
            },
        },
        Permissions,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use super::{apply, register, unapply};

pub const NAME: &str = "message";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "verify" => verify(ctx, command, pool).await,
        "match" => r#match(ctx, command, pool).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Message")
        .create_option(|o| {
            o.name("verify")
                .description("Verify message")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("match")
                .description("Match message")
                .kind(CommandOptionType::SubCommand)
        })
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

async fn verify(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _: &SqlitePool,
) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    command.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.title("Join StarDB Completionist Community")
            .thumbnail("https://cdn.discordapp.com/emojis/1112854178302267452.png")
            .description("Enter your UID to become a verified member and gain access to leaderboards, giveaways, and other server tools.\n\nYour UID and achievements will be automatically verified and added to the leaderboard.\n\nClick the \"Register\" button below and follow the on-screen instructions.")
            })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id(register::NAME)
                        .label("Register")
                        .style(ButtonStyle::Primary)
                })
            })
        })).await?;

    command
        .create_followup_message(ctx, |m| m.content("Sent message").ephemeral(true))
        .await?;

    Ok(())
}

async fn r#match(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _: &SqlitePool,
) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    command.channel_id.send_message(ctx, |m| m.embed(|e| {
        e.title("Support Contract")
            .thumbnail("https://cdn.discordapp.com/emojis/1112854178302267452.png")
            .description("This tool randomly matches you with a player in your region to mutually earn 20k support credits daily, and for as long as both parties agree to the support contract.\n\nDisclaimer: You can't choose which player, nor know the level of their units.\n\nRequirement: You must have the @Chive Verified role via https://discord.com/channels/1008493665116758167/1138771945517764608 in order to hit the apply button.")
            })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id(apply::NAME)
                        .label("Apply")
                        .style(ButtonStyle::Primary)
                }).create_button(|b| {
                    b.custom_id(unapply::NAME)
                        .label("Unapply")
                        .style(ButtonStyle::Danger)
                })
            })
        })).await?;

    command
        .create_followup_message(ctx, |m| m.content("Sent message").ephemeral(true))
        .await?;

    Ok(())
}
