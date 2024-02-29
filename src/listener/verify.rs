use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType, UserId},
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::{database, updater, GUILD_ID};

pub struct Verify;

impl super::Listener for Verify {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Verify a verification")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "uid", "Uid")
                    .required(true)
                    .set_autocomplete(true),
            )
            .default_member_permissions(Permissions::MANAGE_ROLES)
    }

    async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
        command
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        if command.guild_id != Some(GUILD_ID) {
            return Err(anyhow!("Has to be used in Meow Completionist Guild"));
        }

        let uid = command.data.options[0].value.as_i64().unwrap();

        let vd = database::get_verification_by_uid(uid, pool).await?;
        database::delete_verification_by_uid(uid, pool).await?;

        let user = vd.user;

        let score_data = database::DbConnection { uid, user };
        database::set_connection(&score_data, pool).await?;

        updater::update_user_roles(user, &mut HashSet::new(), &ctx.http, pool).await?;

        if let Ok(channel) = UserId::new(user as u64).create_dm_channel(&ctx).await {
            let _ = channel
                .send_message(&ctx, CreateMessage::new().content("Congratulations Completionist! You are now @Chive Verified and your profile will appear on the Chive Leaderboards: https://stardb.gg/leaderboard. You can change your HSR bio back to what it was originally. Additionally, you've gained access to the https://discord.com/channels/1008493665116758167/1108110331043123200 channel."))
                .await;
        }

        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new()
                    .content(format!("Successfully verified {uid}"))
                    .ephemeral(true),
            )
            .await?;

        Ok(())
    }

    async fn autocomplete(
        ctx: &Context,
        autocomplete: &CommandInteraction,
        pool: &SqlitePool,
    ) -> Result<()> {
        let input = autocomplete
            .data
            .options
            .first()
            .and_then(|o| o.value.as_str())
            .map(|s| format!("{s}%"))
            .unwrap_or("%".to_string());

        let vds = database::get_verifications_where_like(&input, pool).await?;

        let mut choices = Vec::new();

        for vd in vds {
            let uid = vd.uid;
            let user = vd.name;

            choices.push((format!("{uid} - {user}"), uid));
        }

        let mut response = CreateAutocompleteResponse::new();

        for choice in choices {
            response = response.add_int_choice(choice.0, choice.1);
        }

        autocomplete
            .create_response(&ctx, CreateInteractionResponse::Autocomplete(response))
            .await?;

        Ok(())
    }
}
