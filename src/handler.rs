use std::collections::HashMap;

use anyhow::Result;
use serenity::{
    all::{
        Command, CommandInteraction, ComponentInteraction, GuildId, Interaction, Member,
        ModalInteraction, Reaction, Ready, User,
    },
    builder::CreateInteractionResponseFollowup,
    client::{Context, EventHandler},
    futures::StreamExt,
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

        tokio::spawn(async move {
            let members = GUILD_ID.members_iter(&ctx).collect::<Vec<_>>().await;

            crate::updater::log(&format!("Total users {}", members.len()), &ctx.http).await;

            for (i, members) in members.chunks(10).enumerate() {
                let mut handles = vec![];

                for member in members {
                    let member = match member {
                        Err(e) => {
                            crate::updater::log(&format!("{i} {e}"), &ctx.http).await;
                            continue;
                        }
                        Ok(m) => m,
                    }
                    .clone();

                    if member.user.bot {
                        continue;
                    }

                    let ctx = ctx.clone();
                    handles.push(tokio::spawn(async move {
                        loop {
                            if let Err(e) = member.add_role(&ctx, 1210489410467143741).await {
                                crate::updater::log(&format!("{e}"), &ctx.http).await;
                            }

                            if member.add_role(&ctx, 1210489410467143741).await.is_ok() {
                                break;
                            }

                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }));
                }

                for handle in handles {
                    handle.await.unwrap();
                }

                crate::updater::log(&format!("Updated chunk {i}"), &ctx.http).await;
            }

            crate::updater::log("<@246684413075652612> Done", &ctx.http).await;
        });
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
