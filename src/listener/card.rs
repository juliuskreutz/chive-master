use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{
        CreateAttachment, CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::database;

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(CreateCommand::new(name)
            .description("Generate a player card. All relic slots must be filled. Optional color personalization in hex code.")
            .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "showuid", "Should the card show your uid").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "character", "Which support character should it generate").required(true)
                .add_int_choice("0", 0)
                .add_int_choice("1", 1)
                .add_int_choice("2", 2)
                .add_int_choice("3", 3)
                .add_int_choice("4", 4)
                .add_int_choice("5", 5)
                .add_int_choice("6", 6)
                .add_int_choice("7", 7))
            .add_option(CreateCommandOption::new(CommandOptionType::String, "primarycolor", "Primary Color in hexcode notation").required(false))
            .add_option(CreateCommandOption::new(CommandOptionType::String, "secondarycolor", "Secondary Color in hexcode notation").required(false))
            .dm_permission(false));
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    let user_id = command.user.id;

    let scores = database::get_connections_by_user(user_id.get() as i64, pool).await?;

    if scores.is_empty() {
        command
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        return Err(anyhow!("This user doesn't have any connected character"));
    }

    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
        )
        .await?;

    let mut url = "https://hsr-profile-generator.vercel.app/api/generate?".to_string();

    let showuid = command
        .data
        .options
        .iter()
        .find(|o| o.name == "showuid")
        .and_then(|o| o.value.as_bool())
        .unwrap();

    url.push_str(&format!("showuid={showuid}"));

    let character = command
        .data
        .options
        .iter()
        .find(|o| o.name == "character")
        .and_then(|o| o.value.as_i64())
        .unwrap();

    url.push_str(&format!("&characterselection={character}"));

    if let Some(primarycolor) = command
        .data
        .options
        .iter()
        .find(|o| o.name == "primarycolor")
        .and_then(|o| o.value.as_str())
    {
        url.push_str(&format!("&primarycolor={primarycolor}"));
    }

    if let Some(secondarycolor) = command
        .data
        .options
        .iter()
        .find(|o| o.name == "secondarycolor")
        .and_then(|o| o.value.as_str())
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

    for (i, bytes) in images.into_iter().enumerate() {
        if let Ok(s) = String::from_utf8(bytes.clone()) {
            command
                .create_followup(&ctx, CreateInteractionResponseFollowup::new().content(s))
                .await?;
        } else {
            command
                .create_followup(
                    &ctx,
                    CreateInteractionResponseFollowup::new()
                        .add_file(CreateAttachment::bytes(bytes, format!("profile{i}.png"))),
                )
                .await?;
        }
    }

    Ok(())
}
