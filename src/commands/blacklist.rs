use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, Reaction, ReactionType},
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub const NAME: &str = "blacklist";

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "add" => add(ctx, command, pool).await,
        "remove" => remove(ctx, command, pool).await,
        "list" => list(ctx, command, pool).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Role management")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add an emoji to the blacklist",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "emoji", "Emoji")
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Remove an emoji from the blacklist",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "emoji", "Emoji")
                    .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List blacklist",
        ))
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

async fn add(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let CommandDataOptionValue::SubCommand(options) = &command.data.options[0].value else {
        return Err(anyhow!("Not a subcommand"));
    };

    let emoji = options[0].value.as_str().unwrap().trim();

    if emoji.is_empty() {
        return Err(anyhow!("Emoji cannot be empty"));
    }

    let blacklist = database::BlacklistData::new(emoji.to_string());
    database::set_emoji(blacklist, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("Added {emoji} to blacklist"))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn remove(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let CommandDataOptionValue::SubCommand(options) = &command.data.options[0].value else {
        return Err(anyhow!("Not a subcommand"));
    };

    let emoji = options[0].value.as_str().unwrap();

    database::delete_emoji_by_emoji(emoji, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("Removed {emoji} to blacklist"))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn list(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let blacklist = database::get_blacklist(pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .embed(
                    CreateEmbed::new().title("Blacklist").description(
                        blacklist
                            .iter()
                            .map(|b| b.emoji.clone())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    ),
                )
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub async fn reaction(ctx: &Context, reaction: &Reaction, pool: &SqlitePool) {
    if let ReactionType::Unicode(emoji) = &reaction.emoji {
        if database::get_blacklist(pool)
            .await
            .unwrap()
            .iter()
            .any(|b| &b.emoji == emoji)
        {
            reaction.delete(&ctx).await.unwrap();
        }
    }
}
