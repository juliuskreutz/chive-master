use anyhow::Result;
use serenity::{
    async_trait,
    model::{
        application::interaction::Interaction,
        gateway::Ready,
        prelude::{
            command::Command,
            interaction::{
                application_command::ApplicationCommandInteraction,
                autocomplete::AutocompleteInteraction,
                message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
            },
            Activity,
        },
    },
    prelude::*,
};
use sqlx::SqlitePool;

use crate::commands;

pub struct Handler {
    pub pool: SqlitePool,
}

impl Handler {
    async fn application_command(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<()> {
        match command.data.name.as_str() {
            commands::register::NAME => commands::register::command(ctx, command, &self.pool).await,
            commands::unregister::NAME => {
                commands::unregister::command(ctx, command, &self.pool).await
            }
            commands::verify::NAME => commands::verify::command(ctx, command, &self.pool).await,
            commands::cancel::NAME => commands::cancel::command(ctx, command, &self.pool).await,
            commands::status::NAME => commands::status::command(ctx, command, &self.pool).await,
            commands::card::NAME => commands::card::command(ctx, command, &self.pool).await,
            commands::message::NAME => commands::message::command(ctx, command, &self.pool).await,
            commands::roles::NAME => commands::roles::command(ctx, command, &self.pool).await,
            commands::role::NAME => commands::role::command(ctx, command, &self.pool).await,
            commands::channel::NAME => commands::channel::command(ctx, command, &self.pool).await,
            _ => Ok(()),
        }
    }

    async fn message_component(
        &self,
        ctx: &Context,
        interaction: &MessageComponentInteraction,
    ) -> Result<()> {
        match interaction.data.custom_id.as_str() {
            commands::register::NAME => commands::register::component(ctx, interaction).await,
            _ => Ok(()),
        }
    }

    async fn modal_submit(
        &self,
        ctx: &Context,
        interaction: &ModalSubmitInteraction,
    ) -> Result<()> {
        match interaction.data.custom_id.as_str() {
            commands::register::NAME => {
                commands::register::modal(ctx, interaction, &self.pool).await
            }
            _ => Ok(()),
        }
    }

    async fn autocomplete(
        &self,
        ctx: &Context,
        autocomplete: &AutocompleteInteraction,
    ) -> Result<()> {
        match autocomplete.data.name.as_str() {
            commands::verify::NAME => {
                commands::verify::autocomplete(ctx, autocomplete, &self.pool).await
            }
            commands::cancel::NAME => {
                commands::cancel::autocomplete(ctx, autocomplete, &self.pool).await
            }
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        Command::set_global_application_commands(&ctx.http, |command| {
            command
                .create_application_command(|command| commands::register::register(command))
                .create_application_command(|command| commands::unregister::register(command))
                .create_application_command(|command| commands::verify::register(command))
                .create_application_command(|command| commands::cancel::register(command))
                .create_application_command(|command| commands::status::register(command))
                .create_application_command(|command| commands::card::register(command))
                .create_application_command(|command| commands::message::register(command))
                .create_application_command(|command| commands::roles::register(command))
                .create_application_command(|command| commands::role::register(command))
                .create_application_command(|command| commands::channel::register(command))
        })
        .await
        .unwrap();

        ctx.set_activity(Activity::watching("Chive Hunters")).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(e) = self.application_command(&ctx, &command).await {
                    command
                        .create_followup_message(&ctx, |m| m.content(e).ephemeral(true))
                        .await
                        .unwrap();
                }
            }
            Interaction::MessageComponent(interaction) => {
                let _ = self.message_component(&ctx, &interaction).await;
            }
            Interaction::ModalSubmit(interaction) => {
                if let Err(e) = self.modal_submit(&ctx, &interaction).await {
                    interaction
                        .create_followup_message(&ctx, |m| m.content(e).ephemeral(true))
                        .await
                        .unwrap();
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                self.autocomplete(&ctx, &autocomplete).await.unwrap();
            }
            _ => unimplemented!(),
        };
    }
}
