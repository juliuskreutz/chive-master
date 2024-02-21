use anyhow::Result;
use serenity::{
    all::{ChannelId, CommandInteraction, CommandOptionType, Mentionable},
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

pub const NAME: &str = "warn";

pub async fn command(ctx: &Context, command: &CommandInteraction, _: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let user = command.data.options[0].value.as_user_id().unwrap();

    if user == 246684413075652612 {
        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new()
                    .content("Nice try nerd")
                    .ephemeral(true),
            )
            .await?;

        return Ok(());
    }

    let reason = command.data.options[1].value.as_str().unwrap();

    let user = ctx.http.get_user(user).await?;

    let dmed = user
        .create_dm_channel(&ctx)
        .await?
        .send_message(
            &ctx,
            CreateMessage::new().content(format!("You have been warned for: {}", reason)),
        )
        .await
        .is_ok();

    let channel = ChannelId::new(1209471689264603167);

    channel
        .send_message(
            &ctx,
            CreateMessage::new().embed(
                CreateEmbed::default()
                    .color(0xff0000)
                    .title(format!("Warned {}!", user.name))
                    .field("User", user.mention().to_string(), true)
                    .field("Reason", reason, true)
                    .footer(CreateEmbedFooter::new(format!(
                        "Warned by {}",
                        command.user.name
                    ))),
            ),
        )
        .await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!(
                    "{} has been warned {}",
                    user.name,
                    if dmed {
                        "and dmed"
                    } else {
                        "BUT DID NOT GET A DM"
                    }
                ))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "User").required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "reason", "Reason").required(true),
        )
        .description("Warn a user")
        .default_member_permissions(Permissions::MANAGE_NICKNAMES)
}
