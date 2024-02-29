use anyhow::{anyhow, Result};
use serenity::{
    all::{ButtonStyle, CommandInteraction, CommandOptionType},
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

pub struct Message;

impl super::Listener for Message {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Message")
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "verify",
                "Verify message",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "match",
                "Match message",
            ))
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }

    async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
        match command.data.options[0].name.as_str() {
            "verify" => verify(ctx, command, pool).await,
            "match" => r#match(ctx, command, pool).await,
            _ => Err(anyhow!("Not a subcommand")),
        }
    }
}

async fn verify(ctx: &Context, command: &CommandInteraction, _: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let embed = CreateEmbed::new().title("Join StarDB Completionist Community")
        .thumbnail("https://cdn.discordapp.com/emojis/1112854178302267452.png")
        .description("Enter your UID to become a verified member and gain access to leaderboards, giveaways, and other server tools.\n\nYour UID and achievements will be automatically verified and added to the leaderboard.\n\nClick the \"Register\" button below and follow the on-screen instructions.");

    command
        .channel_id
        .send_message(
            &ctx,
            CreateMessage::new()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    super::ListenerName::Register.to_string(),
                )
                .label("Register")
                .style(ButtonStyle::Primary)])]),
        )
        .await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("Sent message")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn r#match(ctx: &Context, command: &CommandInteraction, _: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let embed = CreateEmbed::new().title("Support Contract")
        .thumbnail("https://cdn.discordapp.com/emojis/1112854178302267452.png")
        .description("This tool randomly matches you with a player in your region to mutually earn 20k support credits daily, and for as long as both parties agree to the support contract.\n\nDisclaimer: You can't choose which player, nor know the level of their units.\n\nRequirement: You must have the @Chive Verified role via https://discord.com/channels/1008493665116758167/1138771945517764608 in order to hit the apply button.");

    command
        .channel_id
        .send_message(
            &ctx,
            CreateMessage::new()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(super::ListenerName::Apply.to_string())
                        .label("Apply")
                        .style(ButtonStyle::Primary),
                    CreateButton::new(super::ListenerName::Unapply.to_string())
                        .label("Unapply")
                        .style(ButtonStyle::Danger),
                ])]),
        )
        .await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("Sent message")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
