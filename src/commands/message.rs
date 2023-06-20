use anyhow::Result;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
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

use super::register;

pub const NAME: &str = "message";

pub async fn command(
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
        e.title("Join the Chive Hunters Leaderboard")
            .thumbnail("https://cdn.discordapp.com/emojis/1112854178302267452.png")
            .description("Enter your UID to join the Chive Army for Honkai Star Rail.\n\nYour name and achievements will be automatically verified and added to the leaderboard.\n\nClick the \"Register\" button below and follow the on-screen instructions.")
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

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Message")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
