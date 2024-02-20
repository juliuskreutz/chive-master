use anyhow::Result;
use serenity::{
    all::{
        Command, CommandInteraction, ComponentInteraction, Interaction, ModalInteraction, Reaction,
        Ready,
    },
    builder::CreateInteractionResponseFollowup,
    client::{Context, EventHandler},
    gateway::ActivityData,
};
use sqlx::SqlitePool;

use crate::commands;

pub struct Handler {
    pub pool: SqlitePool,
}

impl Handler {
    async fn application_command(&self, ctx: &Context, command: &CommandInteraction) -> Result<()> {
        match command.data.name.as_str() {
            commands::register::NAME => commands::register::command(ctx, command, &self.pool).await,
            commands::unregister::NAME => {
                commands::unregister::command(ctx, command, &self.pool).await
            }
            commands::verify::NAME => commands::verify::command(ctx, command, &self.pool).await,
            commands::status::NAME => commands::status::command(ctx, command, &self.pool).await,
            commands::card::NAME => commands::card::command(ctx, command, &self.pool).await,
            commands::message::NAME => commands::message::command(ctx, command, &self.pool).await,
            commands::roles::NAME => commands::roles::command(ctx, command, &self.pool).await,
            commands::role::NAME => commands::role::command(ctx, command, &self.pool).await,
            commands::channel::NAME => commands::channel::command(ctx, command, &self.pool).await,
            commands::rolestats::NAME => {
                commands::rolestats::command(ctx, command, &self.pool).await
            }
            commands::apply::NAME => commands::apply::command(ctx, command, &self.pool).await,
            commands::unapply::NAME => commands::unapply::command(ctx, command, &self.pool).await,
            commands::disband::NAME => commands::disband::command(ctx, command, &self.pool).await,
            commands::uids::NAME => commands::uids::command(ctx, command, &self.pool).await,
            commands::sniff::NAME => commands::sniff::command(ctx, command, &self.pool).await,
            commands::update::NAME => commands::update::command(ctx, command, &self.pool).await,
            commands::blacklist::NAME => {
                commands::blacklist::command(ctx, command, &self.pool).await
            }
            commands::warn::NAME => commands::warn::command(ctx, command, &self.pool).await,
            _ => Ok(()),
        }
    }

    async fn message_component(
        &self,
        ctx: &Context,
        interaction: &ComponentInteraction,
    ) -> Result<()> {
        match interaction.data.custom_id.as_str() {
            commands::register::NAME => {
                commands::register::component(ctx, interaction, &self.pool).await
            }
            commands::apply::NAME => commands::apply::component(ctx, interaction, &self.pool).await,
            commands::unapply::NAME => {
                commands::unapply::component(ctx, interaction, &self.pool).await
            }
            _ => Ok(()),
        }
    }

    async fn modal_submit(&self, ctx: &Context, interaction: &ModalInteraction) -> Result<()> {
        match interaction.data.custom_id.as_str() {
            commands::register::NAME => {
                commands::register::modal(ctx, interaction, &self.pool).await
            }
            _ => Ok(()),
        }
    }

    async fn autocomplete(&self, ctx: &Context, autocomplete: &CommandInteraction) -> Result<()> {
        match autocomplete.data.name.as_str() {
            commands::verify::NAME => {
                commands::verify::autocomplete(ctx, autocomplete, &self.pool).await
            }
            _ => Ok(()),
        }
    }

    async fn reaction(&self, ctx: &Context, reaction: &Reaction) {
        commands::blacklist::reaction(ctx, reaction, &self.pool).await;
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        Command::set_global_commands(
            &ctx,
            vec![
                commands::register::register(),
                commands::unregister::register(),
                commands::verify::register(),
                commands::status::register(),
                commands::card::register(),
                commands::message::register(),
                commands::roles::register(),
                commands::role::register(),
                commands::channel::register(),
                commands::rolestats::register(),
                commands::apply::register(),
                commands::unapply::register(),
                commands::disband::register(),
                commands::uids::register(),
                commands::sniff::register(),
                commands::update::register(),
                commands::blacklist::register(),
                commands::warn::register(),
            ],
        )
        .await
        .unwrap();

        ctx.set_activity(Some(ActivityData::watching("Chive Hunters")));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                if let Err(e) = self.application_command(&ctx, &command).await {
                    command
                        .create_followup(
                            &ctx,
                            CreateInteractionResponseFollowup::new()
                                .content(e.to_string())
                                .ephemeral(true),
                        )
                        .await
                        .unwrap();
                }
            }
            Interaction::Component(interaction) => {
                if let Err(e) = self.message_component(&ctx, &interaction).await {
                    interaction
                        .create_followup(
                            &ctx,
                            CreateInteractionResponseFollowup::new()
                                .content(e.to_string())
                                .ephemeral(true),
                        )
                        .await
                        .unwrap();
                }
            }
            Interaction::Modal(interaction) => {
                if let Err(e) = self.modal_submit(&ctx, &interaction).await {
                    interaction
                        .create_followup(
                            &ctx,
                            CreateInteractionResponseFollowup::new()
                                .content(e.to_string())
                                .ephemeral(true),
                        )
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

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        self.reaction(&ctx, &reaction).await;
    }
}
