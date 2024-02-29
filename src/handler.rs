use std::collections::HashMap;

use anyhow::Result;
use serenity::{
    all::{
        Command, CommandInteraction, ComponentInteraction, GuildId, Interaction, Member,
        ModalInteraction, Reaction, Ready, User,
    },
    builder::CreateInteractionResponseFollowup,
    client::{Context, EventHandler},
    gateway::ActivityData,
};
use sqlx::SqlitePool;
use strum::IntoEnumIterator;

use crate::{listener, GUILD_ID};

pub struct Handler {
    pub pool: SqlitePool,
    pub listeners: HashMap<String, listener::ListenerName>,
}

impl Handler {
    async fn command(&self, ctx: &Context, command: &CommandInteraction) -> Result<()> {
        if let Some(listener) = self.listeners.get(command.data.name.as_str()) {
            return listener.command(ctx, command, &self.pool).await;
        }

        Ok(())
    }

    async fn component(&self, ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        if let Some(listener) = self.listeners.get(interaction.data.custom_id.as_str()) {
            return listener.component(ctx, interaction, &self.pool).await;
        }

        Ok(())
    }

    async fn modal(&self, ctx: &Context, interaction: &ModalInteraction) -> Result<()> {
        if let Some(listener) = self.listeners.get(interaction.data.custom_id.as_str()) {
            return listener.modal(ctx, interaction, &self.pool).await;
        }

        Ok(())
    }

    async fn autocomplete(&self, ctx: &Context, autocomplete: &CommandInteraction) -> Result<()> {
        if let Some(listener) = self.listeners.get(autocomplete.data.name.as_str()) {
            return listener.autocomplete(ctx, autocomplete, &self.pool).await;
        }

        Ok(())
    }

    async fn reaction_add(&self, ctx: &Context, reaction: &Reaction) -> Result<()> {
        for listener in self.listeners.values() {
            listener.reaction_add(ctx, reaction, &self.pool).await?;
        }

        Ok(())
    }

    async fn ban_add(&self, ctx: &Context, guild: &GuildId, user: &User) -> Result<()> {
        for listener in self.listeners.values() {
            listener.ban_add(ctx, guild, user, &self.pool).await?;
        }

        Ok(())
    }

    async fn ban_remove(&self, ctx: &Context, guild: &GuildId, user: &User) -> Result<()> {
        for listener in self.listeners.values() {
            listener.ban_remove(ctx, guild, user, &self.pool).await?;
        }

        Ok(())
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let commands = listener::ListenerName::iter()
            .map(|c| c.register())
            .collect();

        Command::set_global_commands(&ctx, commands).await.unwrap();

        ctx.set_activity(Some(ActivityData::watching("Chive Hunters")));

        let members = GUILD_ID.members(&ctx, None, None).await.unwrap();

        for member in members {
            if member.user.bot {
                continue;
            }

            loop {
                if member.add_role(&ctx, 1210489410467143741).await.is_ok() {
                    break;
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                if let Err(e) = self.command(&ctx, &command).await {
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
                if let Err(e) = self.component(&ctx, &interaction).await {
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
                if let Err(e) = self.modal(&ctx, &interaction).await {
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
        self.reaction_add(&ctx, &reaction).await.unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        if member.user.bot {
            return;
        }

        loop {
            if member.add_role(&ctx, 1210489410467143741).await.is_ok() {
                break;
            }
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild: GuildId, user: User) {
        self.ban_add(&ctx, &guild, &user).await.unwrap();
    }

    async fn guild_ban_removal(&self, ctx: Context, guild: GuildId, user: User) {
        self.ban_remove(&ctx, &guild, &user).await.unwrap();
    }
}
