use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, Rng};
use serenity::{
    builder::{CreateApplicationCommand, CreateInteractionResponseFollowup},
    model::prelude::{
        command::CommandOptionType,
        component::{ActionRowComponent, InputTextStyle},
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
            message_component::MessageComponentInteraction,
            modal::ModalSubmitInteraction,
            InteractionResponseType,
        },
        UserId,
    },
    prelude::{Context, Mentionable},
};
use sqlx::SqlitePool;

use crate::database::{self, VerificationData};

pub const NAME: &str = "register";
const UID_ID: &str = "uid";

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

    let option = command.data.options[0]
        .resolved
        .as_ref()
        .ok_or_else(|| anyhow!("No option"))?;

    let CommandDataOptionValue::Integer(uid) = *option else {
        return Err(anyhow!("Not an integer"));
    };

    if let Ok(score_data) = database::get_score_by_uid(uid, pool).await {
        return Err(anyhow!(
            "Already registered to {}",
            UserId(score_data.user as u64).mention()
        ));
    }

    if database::get_verification_by_uid(uid, pool).await.is_ok() {
        return Err(anyhow!(
            "Awaiting verification. Check verification status with /status."
        ));
    };

    let user = &command.user;
    let user_id = user.id.0 as i64;
    let otp = otp();
    let name = user.name.clone();

    let data = VerificationData::new(uid, user_id, name, otp);

    database::set_verification(&data, pool).await?;

    command
        .create_followup_message(ctx, |m| response(m, &data.otp))
        .await?;

    Ok(())
}

pub async fn modal(
    ctx: &Context,
    interaction: &ModalSubmitInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
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
        .ok_or_else(|| anyhow!("No uid"))?
        .value
        .parse()?;

    if let Ok(score_data) = database::get_score_by_uid(uid, pool).await {
        return Err(anyhow!(
            "Already registered to {}",
            UserId(score_data.user as u64).mention()
        ));
    }

    if database::get_verification_by_uid(uid, pool).await.is_ok() {
        return Err(anyhow!(
            "Awaiting verification. Check verification status with /status."
        ));
    };

    let user = &interaction.user;
    let user_id = user.id.0 as i64;
    let otp = otp();
    let name = user.name.clone();

    let data = VerificationData::new(uid, user_id, name, otp);

    database::set_verification(&data, pool).await?;

    interaction
        .create_followup_message(ctx, |m| response(m, &data.otp))
        .await?;

    Ok(())
}

pub async fn component(ctx: &Context, interaction: &MessageComponentInteraction) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::Modal)
                .interaction_response_data(|d| {
                    d.custom_id(NAME)
                        .title("Please put in your uid")
                        .components(|c| {
                            c.create_action_row(|r| {
                                r.create_input_text(|i| {
                                    i.custom_id(UID_ID)
                                        .label("uid")
                                        .placeholder("123456789")
                                        .style(InputTextStyle::Short)
                                })
                            })
                        })
                })
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Register your account")
        .create_option(|o| {
            o.name("uid")
                .description("Your uid")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
}

fn response<'a, 'b>(
    response: &'b mut CreateInteractionResponseFollowup<'a>,
    otp: &str,
) -> &'b mut CreateInteractionResponseFollowup<'a> {
    let text = format!("Please verify that your UID belongs to you by appending the following 6 characters to your bio of your player account in game. The bio must have the 6 characters last.\n\n**{otp}**\n\nThen wait 5 - 15 mins. The bot will verify your ownership and add your account's achievements to the rankings. Once you are added, you're free to change your comment section.\n\nIf you encounter an issue, please message us in the <#1010268018028327062> channel.");

    response
        .embed(|e| {
            e.title("Verification")
                .description(text)
                .footer(|f| f.text("Check verification status with /status"))
        })
        .ephemeral(true)
}

fn otp() -> String {
    loop {
        let otp: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        if !otp.to_lowercase().contains("gay") {
            break otp;
        }
    }
}
