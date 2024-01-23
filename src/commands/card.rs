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
    let user_id = command.user.id;

    let scores = database::get_connections_by_user(user_id.0 as i64, pool).await?;

    if scores.is_empty() {
        command
            .create_interaction_response(ctx, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(
                        |d: &mut serenity::builder::CreateInteractionResponseData<'_>| {
                            d.ephemeral(true)
                        },
                    )
            })
            .await?;

        return Err(anyhow!("This user doesn't have any connected character"));
    }

    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d)
        })
        .await?;

    let mut url = "https://hsr-profile-generator.vercel.app/api/generate?".to_string();

    let showuid = command
        .data
        .options
        .iter()
        .find(|o| o.name == "showuid")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::Boolean(s) = o {
                Some(*s)
            } else {
                None
            }
        })
        .unwrap();

    url.push_str(&format!("showuid={showuid}"));

    let character = command
        .data
        .options
        .iter()
        .find(|o| o.name == "character")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::Integer(i) = o {
                Some(*i)
            } else {
                None
            }
        })
        .unwrap();

    url.push_str(&format!("&characterselection={character}"));

    if let Some(primarycolor) = command
        .data
        .options
        .iter()
        .find(|o| o.name == "primarycolor")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::String(s) = o {
                Some(s.replace('#', ""))
            } else {
                None
            }
        })
    {
        url.push_str(&format!("&primarycolor={primarycolor}"));
    }

    if let Some(secondarycolor) = command
        .data
        .options
        .iter()
        .find(|o| o.name == "secondarycolor")
        .and_then(|o| o.resolved.as_ref())
        .and_then(|o| {
            if let CommandDataOptionValue::String(s) = o {
                Some(s.replace('#', ""))
            } else {
                None
            }
        })
    {
        url.push_str(&format!("&secondarycolor={secondarycolor}"));
    }

    let mut images = Vec::new();

    for score in scores {
        let uid = score.uid;

        let bytes = reqwest::get(&format!("{url}&uid={uid}"))
            .await?
            .bytes()
            .await?
            .to_vec();

        images.push(bytes);
    }

    for (i, bytes) in images.iter().enumerate() {
        if let Ok(s) = String::from_utf8(bytes.clone()) {
            command
                .create_followup_message(ctx, |m| m.content(s))
                .await?;
        } else {
            command
                .create_followup_message(ctx, |m| {
                    m.add_file((bytes.as_slice(), format!("profile{i}.png").as_str()))
                })
                .await?;
        }
    }

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Generate a player card. All relic slots must be filled. Optional color personalization in hex code.")
        .create_option(|o| {
            o.name("showuid")
                .description("Should the card show your uid")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
        .create_option(|o| {
            o.name("character")
                .description("Which support character should it generate")
                .kind(CommandOptionType::Integer)
                .add_int_choice("0", 0)
                .add_int_choice("1", 1)
                .add_int_choice("2", 2)
                .add_int_choice("3", 3)
                .required(true)
        })
        .create_option(|o| {
            o.name("primarycolor")
                .description("Primary Color in hexcode notation")
                .kind(CommandOptionType::String)
                .required(false)
        })
        .create_option(|o| {
            o.name("secondarycolor")
                .description("Secondary Color in hexcode notation")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
