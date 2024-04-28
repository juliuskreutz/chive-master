mod apply;
mod bans;
mod blacklist;
mod card;
mod disband;
mod message;
mod purge;
mod register;
mod role;
mod roles;
mod rolestats;
mod sniff;
mod sql;
mod status;
mod uids;
mod unapply;
mod unregister;
mod update;
mod verify;
mod warn;

use anyhow::Result;
use serenity::{
    all::{CommandInteraction, ComponentInteraction, GuildId, ModalInteraction, Reaction, User},
    client::Context,
};
use sqlx::SqlitePool;

#[derive(strum_macros::Display, strum_macros::EnumIter)]
#[strum(serialize_all = "camelCase")]
pub enum ListenerName {
    Apply,
    Bans,
    Blacklist,
    Card,
    Disband,
    Message,
    Register,
    Role,
    Roles,
    Rolestats,
    Sniff,
    Sql,
    Status,
    Uids,
    Unapply,
    Unregister,
    Update,
    Verify,
    Warn,
    Purge,
}

impl ListenerName {
    pub fn register(&self) -> serenity::builder::CreateCommand {
        match self {
            ListenerName::Apply => apply::register(&self.to_string()),
            ListenerName::Bans => bans::register(&self.to_string()),
            ListenerName::Blacklist => blacklist::register(&self.to_string()),
            ListenerName::Card => card::register(&self.to_string()),
            ListenerName::Disband => disband::register(&self.to_string()),
            ListenerName::Message => message::register(&self.to_string()),
            ListenerName::Register => register::register(&self.to_string()),
            ListenerName::Role => role::register(&self.to_string()),
            ListenerName::Roles => roles::register(&self.to_string()),
            ListenerName::Rolestats => rolestats::register(&self.to_string()),
            ListenerName::Sniff => sniff::register(&self.to_string()),
            ListenerName::Sql => sql::register(&self.to_string()),
            ListenerName::Status => status::register(&self.to_string()),
            ListenerName::Uids => uids::register(&self.to_string()),
            ListenerName::Unapply => unapply::register(&self.to_string()),
            ListenerName::Unregister => unregister::register(&self.to_string()),
            ListenerName::Update => update::register(&self.to_string()),
            ListenerName::Verify => verify::register(&self.to_string()),
            ListenerName::Warn => warn::register(&self.to_string()),
            ListenerName::Purge => purge::register(&self.to_string()),
        }
    }

    pub async fn command(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::command(ctx, command, pool).await,
            ListenerName::Bans => bans::command(ctx, command, pool).await,
            ListenerName::Blacklist => blacklist::command(ctx, command, pool).await,
            ListenerName::Card => card::command(ctx, command, pool).await,
            ListenerName::Disband => disband::command(ctx, command, pool).await,
            ListenerName::Message => message::command(ctx, command, pool).await,
            ListenerName::Register => register::command(ctx, command, pool).await,
            ListenerName::Role => role::command(ctx, command, pool).await,
            ListenerName::Roles => roles::command(ctx, command, pool).await,
            ListenerName::Rolestats => rolestats::command(ctx, command, pool).await,
            ListenerName::Sniff => sniff::command(ctx, command, pool).await,
            ListenerName::Sql => sql::command(ctx, command, pool).await,
            ListenerName::Status => status::command(ctx, command, pool).await,
            ListenerName::Uids => uids::command(ctx, command, pool).await,
            ListenerName::Unapply => unapply::command(ctx, command, pool).await,
            ListenerName::Unregister => unregister::command(ctx, command, pool).await,
            ListenerName::Update => update::command(ctx, command, pool).await,
            ListenerName::Verify => verify::command(ctx, command, pool).await,
            ListenerName::Warn => warn::command(ctx, command, pool).await,
            ListenerName::Purge => purge::command(ctx, command, pool).await,
        }
    }

    pub async fn component(
        &self,
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::component(ctx, interaction, pool).await,
            ListenerName::Register => register::component(ctx, interaction, pool).await,
            ListenerName::Unapply => unapply::component(ctx, interaction, pool).await,
            _ => Ok(()),
        }
    }

    pub async fn modal(
        &self,
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Register => register::modal(ctx, interaction, pool).await,
            _ => Ok(()),
        }
    }

    pub async fn autocomplete(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Verify => verify::autocomplete(ctx, command, pool).await,
            _ => Ok(()),
        }
    }

    pub async fn reaction_add(
        &self,
        ctx: &Context,
        reaction: &Reaction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Blacklist => blacklist::reaction_add(ctx, reaction, pool).await,
            _ => Ok(()),
        }
    }

    pub async fn ban_add(
        &self,
        ctx: &Context,
        guild_id: &GuildId,
        user: &User,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Bans => bans::ban_add(ctx, guild_id, user, pool).await,
            _ => Ok(()),
        }
    }

    pub async fn ban_remove(
        &self,
        ctx: &Context,
        guild_id: &GuildId,
        user: &User,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Bans => bans::ban_remove(ctx, guild_id, user, pool).await,
            _ => Ok(()),
        }
    }
}
