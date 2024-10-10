use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use linked_hash_map::LinkedHashMap;
use serenity::{
    all::{
        Channel, ChannelType, Command, CommandInteraction, ComponentInteraction, GuildChannel,
        GuildId, Interaction, Member, Message, ModalInteraction, Reaction, Ready, RoleId, User,
        UserId,
    },
    builder::CreateInteractionResponseFollowup,
    client::{Context, EventHandler},
    futures::StreamExt,
    gateway::ActivityData,
    prelude::TypeMapKey,
};
use sqlx::SqlitePool;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;

use crate::{database, listener, GUILD_ID};

pub struct MessageCache;

impl TypeMapKey for MessageCache {
    type Value = Arc<Mutex<LinkedHashMap<(u64, u64), Message>>>;
}

pub struct Handler {
    pub user: Arc<Mutex<UserId>>,
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
    async fn ready(&self, ctx: Context, ready: Ready) {
        *self.user.lock().await = ready.user.id;

        {
            let mut data = ctx.data.write().await;

            data.insert::<MessageCache>(Default::default());
        }

        let mut commands = Vec::new();
        for l in listener::ListenerName::iter() {
            l.register(&mut commands);
        }
        Command::set_global_commands(&ctx, commands).await.unwrap();

        ctx.set_activity(Some(ActivityData::watching("Chive Hunters")));

        tokio::spawn(async move {
            let members = GUILD_ID
                .members_iter(&ctx)
                .filter_map(|m| async move {
                    m.ok().and_then(|m| {
                        (!(m.roles.contains(&RoleId::new(1210489410467143741)) || m.user.bot))
                            .then_some(m)
                    })
                })
                .collect::<Vec<_>>()
                .await;

            for member in members {
                loop {
                    if member.add_role(&ctx, 1210489410467143741).await.is_ok() {
                        break;
                    }
                }
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

    async fn message(&self, ctx: Context, message: Message) {
        if !message.author.bot {
            if let Ok(Channel::Guild(channel)) = message.channel(&ctx).await {
                let re = regex::Regex::new(r"<@&\d+>").unwrap();
                if channel.kind == ChannelType::News && re.find(&message.content).is_some() {
                    let _ = message.crosspost(&ctx).await;
                }
            }
        }

        let message_cache_lock = {
            let data = ctx.data.read().await;

            data.get::<MessageCache>().unwrap().clone()
        };

        let mut message_cache = message_cache_lock.lock().await;
        message_cache.insert((message.channel_id.get(), message.id.get()), message);
        if message_cache.len() > 1000 {
            message_cache.pop_front();
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        self.reaction_add(&ctx, &reaction).await.unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        if member.user.bot {
            return;
        }

        let guild_roles = member.guild_id.roles(&ctx).await.unwrap();

        let position = {
            let user = *self.user.lock().await;
            let member = member.guild_id.member(&ctx, user).await.unwrap();

            member
                .roles
                .iter()
                .map(|r| guild_roles[r].position)
                .max()
                .unwrap()
        };

        let mut roles = vec![1210489410467143741];

        for user_role in database::get_user_roles_by_user(member.user.id.get() as i64, &self.pool)
            .await
            .unwrap()
        {
            if guild_roles
                .get(&RoleId::new(user_role.role as u64))
                .map(|r| r.position < position)
                .unwrap_or_default()
            {
                roles.push(user_role.role as u64);
            }
        }

        for role in roles {
            loop {
                if member.add_role(&ctx, role).await.is_ok() {
                    break;
                }
            }
        }

        database::delete_user_roles_by_user(member.user.id.get() as i64, &self.pool)
            .await
            .unwrap();
    }

    async fn guild_member_removal(
        &self,
        _: Context,
        _: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        let Some(member) = member_data_if_available else {
            return;
        };

        let mut user_role = database::DbUserRole {
            user: user.id.get() as i64,
            role: 0,
        };

        for role in member.roles {
            user_role.role = role.get() as i64;

            database::set_user_role(&user_role, &self.pool)
                .await
                .unwrap();
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild: GuildId, user: User) {
        self.ban_add(&ctx, &guild, &user).await.unwrap();
    }

    async fn guild_ban_removal(&self, ctx: Context, guild: GuildId, user: User) {
        self.ban_remove(&ctx, &guild, &user).await.unwrap();
    }
}
