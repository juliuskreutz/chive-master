use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            InteractionResponseType,
        },
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "card";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    let private = command
        .data
        .options
        .iter()
        .find(|o| o.name == "private")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::Boolean(s) = o {
                Some(*s)
            } else {
                None
            }
        })
        .unwrap();

    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(private))
        })
        .await?;

    let user_id = command.user.id;

    let primary_color = command
        .data
        .options
        .iter()
        .find(|o| o.name == "primarycolor")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::String(s) = o {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or("292525".to_string())
        .replace('#', "");

    let secondary_color = command
        .data
        .options
        .iter()
        .find(|o| o.name == "secondarycolor")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::String(s) = o {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or("E8D9C9".to_string())
        .replace('#', "");

    let scores = database::get_scores_by_user(user_id.0 as i64, pool).await?;

    if scores.is_empty() {
        return Err(anyhow!("This user doesn't have any connected character"));
    }

    let mut images = Vec::new();

    for score in scores {
        let uid = score.uid();

        let bytes = reqwest::get(&format!("https://hsr-profile-generator.vercel.app/api/generate?uid={uid}&primarycolor={primary_color}&secondarycolor={secondary_color}")).await?.bytes().await?.to_vec();

        images.push(bytes);
    }

    command
        .create_followup_message(ctx, |m| {
            for (i, image) in images.iter().enumerate() {
                m.add_file((image.as_slice(), format!("profile{i}.png").as_str()));
            }

            m.ephemeral(private)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Generate a player card. All relic slots must be filled. Optional color personalization in hex code.")
        .create_option(|o| {
            o.name("private")
                .description("Should the card be displayed privately.")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
        .create_option(|o| {
            o.name("primarycolor")
                .description("Primary Color")
                .kind(CommandOptionType::String)
                .required(false)
        })
        .create_option(|o| {
            o.name("secondarycolor")
                .description("Secondary Color")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
