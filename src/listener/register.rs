use anyhow::{anyhow, Result};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use serenity::{
    all::{
        ActionRowComponent, CommandInteraction, CommandOptionType, ComponentInteraction,
        InputTextStyle, Mentionable, ModalInteraction, UserId,
    },
    builder::{
        CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInputText, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateModal,
    },
    client::Context,
};
use sqlx::SqlitePool;

use crate::{database, stardb};

const UID_ID: &str = "uid";

pub struct Register;

impl super::Listener for Register {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Register your account")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "uid", "Your uid")
                    .required(true),
            )
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

        let uid = command.data.options[0].value.as_i64().unwrap();

        if uid == 420 {
            return Err(anyhow!("Try 69 next.."));
        }

        if uid == 69 {
            return Err(anyhow!(
                "Try the answer to life next.. (If you don't know it, just google it)"
            ));
        }

        if uid == 42 {
            return Err(anyhow!("Don't forget to stay hydrated! Next answer is 💧"));
        }

        if let Ok(score_data) = database::get_connection_by_uid(uid, pool).await {
            return Err(anyhow!(
                "Already registered to {}",
                UserId::new(score_data.user as u64).mention()
            ));
        }

        if database::get_verification_by_uid(uid, pool).await.is_ok() {
            return Err(anyhow!(
                "Awaiting verification. Check verification status with /status."
            ));
        };

        if stardb::get(uid).await.is_err() {
            return Err(anyhow!("This uid does not exist or our api is down"));
        }

        let user = command.user.id.get() as i64;
        let name = command.user.name.clone();
        let otp = otp();

        let verification = database::DbVerification {
            uid,
            user,
            name,
            otp: otp.clone(),
            timestamp: Utc::now().naive_utc(),
        };
        database::set_verification(&verification, pool).await?;

        command.create_followup(&ctx, response(&otp)).await?;

        Ok(())
    }

    async fn modal(ctx: &Context, interaction: &ModalInteraction, pool: &SqlitePool) -> Result<()> {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(
                    CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        let uid: i64 = interaction
            .data
            .components
            .iter()
            .flat_map(|r| &r.components)
            .filter_map(|c| match c {
                ActionRowComponent::InputText(input) => Some(input),
                _ => None,
            })
            .find(|i| i.custom_id == UID_ID)
            .and_then(|i| i.value.as_ref())
            .ok_or_else(|| anyhow!("No uid"))?
            .parse()?;

        if let Ok(score_data) = database::get_connection_by_uid(uid, pool).await {
            return Err(anyhow!(
                "Already registered to {}",
                UserId::new(score_data.user as u64).mention()
            ));
        }

        if database::get_verification_by_uid(uid, pool).await.is_ok() {
            return Err(anyhow!(
                "Awaiting verification. Check verification status with /status."
            ));
        };

        if stardb::get(uid).await.is_err() {
            return Err(anyhow!("This uid does not exist or our api is down"));
        }

        let user = interaction.user.id.get() as i64;
        let name = interaction.user.name.clone();
        let otp = otp();

        let verification = database::DbVerification {
            uid,
            user,
            name,
            otp: otp.clone(),
            timestamp: Utc::now().naive_utc(),
        };
        database::set_verification(&verification, pool).await?;

        interaction.create_followup(&ctx, response(&otp)).await?;

        Ok(())
    }

    async fn component(
        ctx: &Context,
        interaction: &ComponentInteraction,
        _: &SqlitePool,
    ) -> Result<()> {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Modal(
                    CreateModal::new(
                        super::ListenerName::Register.to_string(),
                        "Please put in your uid",
                    )
                    .components(vec![CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "uid", UID_ID)
                            .placeholder("123456789"),
                    )]),
                ),
            )
            .await?;

        Ok(())
    }
}

fn response(otp: &str) -> CreateInteractionResponseFollowup {
    let text = format!("Please verify that your UID belongs to you by appending the following 6 characters to your bio of your player account in game. The bio must have the 6 characters last.\n\n**{otp}**\n\nThen wait 5 - 15 mins. The bot will verify your ownership and add your account's achievements to the rankings. Once you are added, you're free to change your comment section.\n\nIf you encounter an issue, please message us in the <#1010268018028327062> channel.");

    CreateInteractionResponseFollowup::new()
        .embed(
            CreateEmbed::new()
                .title("Verification")
                .description(text)
                .footer(CreateEmbedFooter::new(
                    "Check verification status with /status",
                )),
        )
        .ephemeral(true)
}

fn otp() -> String {
    let disallowed = ['I', 'l', 'O', '0'];

    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|&c| !disallowed.contains(&(c as char)))
        .take(6)
        .map(char::from)
        .collect()
}
