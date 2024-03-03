mod apply;
mod bans;
mod blacklist;
mod card;
mod channel;
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

pub trait Listener {
    fn register(name: &str) -> serenity::builder::CreateCommand;

    async fn command(
        _ctx: &Context,
        _command: &CommandInteraction,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }

    async fn component(
        _ctx: &Context,
        _interaction: &ComponentInteraction,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }

    async fn modal(
        _ctx: &Context,
        _interaction: &ModalInteraction,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }

    async fn autocomplete(
        _ctx: &Context,
        _command: &CommandInteraction,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }

    async fn reaction_add(_ctx: &Context, _reaction: &Reaction, _pool: &SqlitePool) -> Result<()> {
        Ok(())
    }

    async fn ban_add(
        _ctx: &Context,
        _guild_id: &GuildId,
        _user_id: &User,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }

    async fn ban_remove(
        _ctx: &Context,
        _guild_id: &GuildId,
        _user_id: &User,
        _pool: &SqlitePool,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(strum_macros::Display, strum_macros::EnumIter)]
#[strum(serialize_all = "camelCase")]
pub enum ListenerName {
    Apply,
    Bans,
    Blacklist,
    Card,
    Channel,
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
            ListenerName::Apply => apply::Apply::register(&self.to_string()),
            ListenerName::Bans => bans::Bans::register(&self.to_string()),
            ListenerName::Blacklist => blacklist::Blacklist::register(&self.to_string()),
            ListenerName::Card => card::Card::register(&self.to_string()),
            ListenerName::Channel => channel::Channel::register(&self.to_string()),
            ListenerName::Disband => disband::Disband::register(&self.to_string()),
            ListenerName::Message => message::Message::register(&self.to_string()),
            ListenerName::Register => register::Register::register(&self.to_string()),
            ListenerName::Role => role::Role::register(&self.to_string()),
            ListenerName::Roles => roles::Roles::register(&self.to_string()),
            ListenerName::Rolestats => rolestats::Rolestats::register(&self.to_string()),
            ListenerName::Sniff => sniff::Sniff::register(&self.to_string()),
            ListenerName::Sql => sql::Sql::register(&self.to_string()),
            ListenerName::Status => status::Status::register(&self.to_string()),
            ListenerName::Uids => uids::Uids::register(&self.to_string()),
            ListenerName::Unapply => unapply::Unapply::register(&self.to_string()),
            ListenerName::Unregister => unregister::Unregister::register(&self.to_string()),
            ListenerName::Update => update::Update::register(&self.to_string()),
            ListenerName::Verify => verify::Verify::register(&self.to_string()),
            ListenerName::Warn => warn::Warn::register(&self.to_string()),
            ListenerName::Purge => purge::Purge::register(&self.to_string()),
        }
    }

    pub async fn command(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::Apply::command(ctx, command, pool).await,
            ListenerName::Bans => bans::Bans::command(ctx, command, pool).await,
            ListenerName::Blacklist => blacklist::Blacklist::command(ctx, command, pool).await,
            ListenerName::Card => card::Card::command(ctx, command, pool).await,
            ListenerName::Channel => channel::Channel::command(ctx, command, pool).await,
            ListenerName::Disband => disband::Disband::command(ctx, command, pool).await,
            ListenerName::Message => message::Message::command(ctx, command, pool).await,
            ListenerName::Register => register::Register::command(ctx, command, pool).await,
            ListenerName::Role => role::Role::command(ctx, command, pool).await,
            ListenerName::Roles => roles::Roles::command(ctx, command, pool).await,
            ListenerName::Rolestats => rolestats::Rolestats::command(ctx, command, pool).await,
            ListenerName::Sniff => sniff::Sniff::command(ctx, command, pool).await,
            ListenerName::Sql => sql::Sql::command(ctx, command, pool).await,
            ListenerName::Status => status::Status::command(ctx, command, pool).await,
            ListenerName::Uids => uids::Uids::command(ctx, command, pool).await,
            ListenerName::Unapply => unapply::Unapply::command(ctx, command, pool).await,
            ListenerName::Unregister => unregister::Unregister::command(ctx, command, pool).await,
            ListenerName::Update => update::Update::command(ctx, command, pool).await,
            ListenerName::Verify => verify::Verify::command(ctx, command, pool).await,
            ListenerName::Warn => warn::Warn::command(ctx, command, pool).await,
            ListenerName::Purge => purge::Purge::command(ctx, command, pool).await,
        }
    }

    pub async fn component(
        &self,
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::Apply::component(ctx, interaction, pool).await,
            ListenerName::Bans => bans::Bans::component(ctx, interaction, pool).await,
            ListenerName::Blacklist => {
                blacklist::Blacklist::component(ctx, interaction, pool).await
            }
            ListenerName::Card => card::Card::component(ctx, interaction, pool).await,
            ListenerName::Channel => channel::Channel::component(ctx, interaction, pool).await,
            ListenerName::Disband => disband::Disband::component(ctx, interaction, pool).await,
            ListenerName::Message => message::Message::component(ctx, interaction, pool).await,
            ListenerName::Register => register::Register::component(ctx, interaction, pool).await,
            ListenerName::Role => role::Role::component(ctx, interaction, pool).await,
            ListenerName::Roles => roles::Roles::component(ctx, interaction, pool).await,
            ListenerName::Rolestats => {
                rolestats::Rolestats::component(ctx, interaction, pool).await
            }
            ListenerName::Sniff => sniff::Sniff::component(ctx, interaction, pool).await,
            ListenerName::Sql => sql::Sql::component(ctx, interaction, pool).await,
            ListenerName::Status => status::Status::component(ctx, interaction, pool).await,
            ListenerName::Uids => uids::Uids::component(ctx, interaction, pool).await,
            ListenerName::Unapply => unapply::Unapply::component(ctx, interaction, pool).await,
            ListenerName::Unregister => {
                unregister::Unregister::component(ctx, interaction, pool).await
            }
            ListenerName::Update => update::Update::component(ctx, interaction, pool).await,
            ListenerName::Verify => verify::Verify::component(ctx, interaction, pool).await,
            ListenerName::Warn => warn::Warn::component(ctx, interaction, pool).await,
            ListenerName::Purge => purge::Purge::component(ctx, interaction, pool).await,
        }
    }

    pub async fn modal(
        &self,
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::Apply::modal(ctx, interaction, pool).await,
            ListenerName::Bans => bans::Bans::modal(ctx, interaction, pool).await,
            ListenerName::Blacklist => blacklist::Blacklist::modal(ctx, interaction, pool).await,
            ListenerName::Card => card::Card::modal(ctx, interaction, pool).await,
            ListenerName::Channel => channel::Channel::modal(ctx, interaction, pool).await,
            ListenerName::Disband => disband::Disband::modal(ctx, interaction, pool).await,
            ListenerName::Message => message::Message::modal(ctx, interaction, pool).await,
            ListenerName::Register => register::Register::modal(ctx, interaction, pool).await,
            ListenerName::Role => role::Role::modal(ctx, interaction, pool).await,
            ListenerName::Roles => roles::Roles::modal(ctx, interaction, pool).await,
            ListenerName::Rolestats => rolestats::Rolestats::modal(ctx, interaction, pool).await,
            ListenerName::Sniff => sniff::Sniff::modal(ctx, interaction, pool).await,
            ListenerName::Sql => sql::Sql::modal(ctx, interaction, pool).await,
            ListenerName::Status => status::Status::modal(ctx, interaction, pool).await,
            ListenerName::Uids => uids::Uids::modal(ctx, interaction, pool).await,
            ListenerName::Unapply => unapply::Unapply::modal(ctx, interaction, pool).await,
            ListenerName::Unregister => unregister::Unregister::modal(ctx, interaction, pool).await,
            ListenerName::Update => update::Update::modal(ctx, interaction, pool).await,
            ListenerName::Verify => verify::Verify::modal(ctx, interaction, pool).await,
            ListenerName::Warn => warn::Warn::modal(ctx, interaction, pool).await,
            ListenerName::Purge => purge::Purge::modal(ctx, interaction, pool).await,
        }
    }

    pub async fn autocomplete(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::Apply::autocomplete(ctx, command, pool).await,
            ListenerName::Bans => bans::Bans::autocomplete(ctx, command, pool).await,
            ListenerName::Blacklist => blacklist::Blacklist::autocomplete(ctx, command, pool).await,
            ListenerName::Card => card::Card::autocomplete(ctx, command, pool).await,
            ListenerName::Channel => channel::Channel::autocomplete(ctx, command, pool).await,
            ListenerName::Disband => disband::Disband::autocomplete(ctx, command, pool).await,
            ListenerName::Message => message::Message::autocomplete(ctx, command, pool).await,
            ListenerName::Register => register::Register::autocomplete(ctx, command, pool).await,
            ListenerName::Role => role::Role::autocomplete(ctx, command, pool).await,
            ListenerName::Roles => roles::Roles::autocomplete(ctx, command, pool).await,
            ListenerName::Rolestats => rolestats::Rolestats::autocomplete(ctx, command, pool).await,
            ListenerName::Sniff => sniff::Sniff::autocomplete(ctx, command, pool).await,
            ListenerName::Sql => sql::Sql::autocomplete(ctx, command, pool).await,
            ListenerName::Status => status::Status::autocomplete(ctx, command, pool).await,
            ListenerName::Uids => uids::Uids::autocomplete(ctx, command, pool).await,
            ListenerName::Unapply => unapply::Unapply::autocomplete(ctx, command, pool).await,
            ListenerName::Unregister => {
                unregister::Unregister::autocomplete(ctx, command, pool).await
            }
            ListenerName::Update => update::Update::autocomplete(ctx, command, pool).await,
            ListenerName::Verify => verify::Verify::autocomplete(ctx, command, pool).await,
            ListenerName::Warn => warn::Warn::autocomplete(ctx, command, pool).await,
            ListenerName::Purge => purge::Purge::autocomplete(ctx, command, pool).await,
        }
    }

    pub async fn reaction_add(
        &self,
        ctx: &Context,
        reaction: &Reaction,
        pool: &SqlitePool,
    ) -> Result<()> {
        match self {
            ListenerName::Apply => apply::Apply::reaction_add(ctx, reaction, pool).await,
            ListenerName::Bans => bans::Bans::reaction_add(ctx, reaction, pool).await,
            ListenerName::Blacklist => {
                blacklist::Blacklist::reaction_add(ctx, reaction, pool).await
            }
            ListenerName::Card => card::Card::reaction_add(ctx, reaction, pool).await,
            ListenerName::Channel => channel::Channel::reaction_add(ctx, reaction, pool).await,
            ListenerName::Disband => disband::Disband::reaction_add(ctx, reaction, pool).await,
            ListenerName::Message => message::Message::reaction_add(ctx, reaction, pool).await,
            ListenerName::Register => register::Register::reaction_add(ctx, reaction, pool).await,
            ListenerName::Role => role::Role::reaction_add(ctx, reaction, pool).await,
            ListenerName::Roles => roles::Roles::reaction_add(ctx, reaction, pool).await,
            ListenerName::Rolestats => {
                rolestats::Rolestats::reaction_add(ctx, reaction, pool).await
            }
            ListenerName::Sniff => sniff::Sniff::reaction_add(ctx, reaction, pool).await,
            ListenerName::Sql => sql::Sql::reaction_add(ctx, reaction, pool).await,
            ListenerName::Status => status::Status::reaction_add(ctx, reaction, pool).await,
            ListenerName::Uids => uids::Uids::reaction_add(ctx, reaction, pool).await,
            ListenerName::Unapply => unapply::Unapply::reaction_add(ctx, reaction, pool).await,
            ListenerName::Unregister => {
                unregister::Unregister::reaction_add(ctx, reaction, pool).await
            }
            ListenerName::Update => update::Update::reaction_add(ctx, reaction, pool).await,
            ListenerName::Verify => verify::Verify::reaction_add(ctx, reaction, pool).await,
            ListenerName::Warn => warn::Warn::reaction_add(ctx, reaction, pool).await,
            ListenerName::Purge => purge::Purge::reaction_add(ctx, reaction, pool).await,
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
            ListenerName::Apply => apply::Apply::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Bans => bans::Bans::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Blacklist => {
                blacklist::Blacklist::ban_add(ctx, guild_id, user, pool).await
            }
            ListenerName::Card => card::Card::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Channel => channel::Channel::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Disband => disband::Disband::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Message => message::Message::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Register => register::Register::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Role => role::Role::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Roles => roles::Roles::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Rolestats => {
                rolestats::Rolestats::ban_add(ctx, guild_id, user, pool).await
            }
            ListenerName::Sniff => sniff::Sniff::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Sql => sql::Sql::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Status => status::Status::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Uids => uids::Uids::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Unapply => unapply::Unapply::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Unregister => {
                unregister::Unregister::ban_add(ctx, guild_id, user, pool).await
            }
            ListenerName::Update => update::Update::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Verify => verify::Verify::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Warn => warn::Warn::ban_add(ctx, guild_id, user, pool).await,
            ListenerName::Purge => purge::Purge::ban_add(ctx, guild_id, user, pool).await,
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
            ListenerName::Apply => apply::Apply::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Bans => bans::Bans::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Blacklist => {
                blacklist::Blacklist::ban_remove(ctx, guild_id, user, pool).await
            }
            ListenerName::Card => card::Card::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Channel => channel::Channel::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Disband => disband::Disband::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Message => message::Message::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Register => {
                register::Register::ban_remove(ctx, guild_id, user, pool).await
            }
            ListenerName::Role => role::Role::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Roles => roles::Roles::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Rolestats => {
                rolestats::Rolestats::ban_remove(ctx, guild_id, user, pool).await
            }
            ListenerName::Sniff => sniff::Sniff::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Sql => sql::Sql::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Status => status::Status::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Uids => uids::Uids::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Unapply => unapply::Unapply::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Unregister => {
                unregister::Unregister::ban_remove(ctx, guild_id, user, pool).await
            }
            ListenerName::Update => update::Update::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Verify => verify::Verify::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Warn => warn::Warn::ban_remove(ctx, guild_id, user, pool).await,
            ListenerName::Purge => purge::Purge::ban_remove(ctx, guild_id, user, pool).await,
        }
    }
}
