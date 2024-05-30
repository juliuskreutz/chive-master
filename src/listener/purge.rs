use anyhow::{anyhow, Result};
use serenity::{
    all::{
        CommandDataOptionValue, CommandInteraction, CommandOptionType, GetMessages, Mentionable,
    },
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(
        CreateCommand::new(name)
            .description("Purge")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "all",
                    "Purge all messages",
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "amount", "Amount")
                        .required(true)
                        .min_int_value(1)
                        .max_int_value(100),
                ),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "user",
                    "Purge messages from a user",
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "User")
                        .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "amount", "Amount")
                        .min_int_value(1)
                        .max_int_value(100),
                ),
            )
            .default_member_permissions(Permissions::MANAGE_MESSAGES)
            .dm_permission(false),
    );
}

pub async fn command(ctx: &Context, command: &CommandInteraction, _: &SqlitePool) -> Result<()> {
    match command.data.options[0].name.as_str() {
        "all" => all(ctx, command).await,
        "user" => user(ctx, command).await,
        _ => Err(anyhow!("Not a subcommand")),
    }
}

async fn all(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

    let amount = options
        .first()
        .map(|o| o.value.as_i64().unwrap())
        .unwrap_or(100);

    let messages = command
        .channel_id
        .messages(&ctx, GetMessages::new().limit(amount as u8))
        .await?;

    command.channel_id.delete_messages(&ctx, &messages).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("Purged {} messages", messages.len()))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn user(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

    let user_id = options[0].value.as_user_id().unwrap();
    let amount = options
        .get(1)
        .map(|o| o.value.as_i64().unwrap())
        .unwrap_or(100);

    let mut messages = command
        .channel_id
        .messages(&ctx, GetMessages::new())
        .await?;

    messages.retain(|m| m.author.id == user_id);
    messages.truncate(amount as usize);

    command.channel_id.delete_messages(&ctx, &messages).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!(
                    "Purged {} messages from {}",
                    messages.len(),
                    user_id.mention()
                ))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
