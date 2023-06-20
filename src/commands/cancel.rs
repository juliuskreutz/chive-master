use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                autocomplete::AutocompleteInteraction,
                InteractionResponseType,
            },
        },
        Permissions,
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "cancel";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    if command
        .guild_id
        .ok_or_else(|| anyhow!("Has to be used in guild"))?
        .0
        != 1008493665116758167
    {
        return Err(anyhow!("Has to be used in Meow Completionist Guild"));
    }

    let option = command.data.options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Integer(uid) = *option else {
        return Err(anyhow!("Not an integer"));
    };

    database::delete_verification_by_uid(uid, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Successfully cancelled {uid}"))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    let input = autocomplete
        .data
        .options
        .get(0)
        .and_then(|o| o.value.as_ref())
        .and_then(|o| o.as_str().map(|s| format!("{s}%")))
        .unwrap_or("%".to_string());

    let vds = database::get_verifications_where_like(&input, pool).await?;

    let mut choices = Vec::new();

    for vd in vds {
        let uid = vd.uid();
        let user = vd.name();

        choices.push((format!("{uid} - {user}"), *uid));
    }

    autocomplete
        .create_autocomplete_response(ctx, |r| {
            for choice in choices {
                r.add_int_choice(choice.0, choice.1);
            }

            r
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Cancel a verification")
        .create_option(|o| {
            o.name("uid")
                .description("Uid")
                .kind(CommandOptionType::Integer)
                .required(true)
                .set_autocomplete(true)
        })
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
