use anyhow::Result;
use serenity::{
    all::{
        ActionRowComponent, ChannelId, CommandInteraction, CommandOptionType, CommandType,
        CreateActionRow, CreateAttachment, CreateInputText, CreateModal, InputTextStyle,
        Mentionable, Message, MessageId, ModalInteraction, User, UserId,
    },
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseFollowup, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use std::collections::HashMap;

use crate::{database, handler::MessageCache};

#[derive(serde::Serialize, serde::Deserialize)]
struct WarnedMessage {
    content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<String>,
}

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(
        CreateCommand::new(name)
            .add_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "User").required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "reason", "Reason")
                    .required(true),
            )
            .description("Warn a user")
            .default_member_permissions(Permissions::MANAGE_NICKNAMES)
            .dm_permission(false),
    );

    commands.push(
        CreateCommand::new(name)
            .kind(CommandType::Message)
            .default_member_permissions(Permissions::MANAGE_NICKNAMES)
            .dm_permission(false),
    );

    commands.push(
        CreateCommand::new(name)
            .kind(CommandType::User)
            .default_member_permissions(Permissions::MANAGE_NICKNAMES)
            .dm_permission(false),
    );
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    match command.data.kind {
        CommandType::Message => {
            message(ctx, command).await?;
            return Ok(());
        }
        CommandType::User => {
            user(ctx, command).await?;
            return Ok(());
        }
        _ => {}
    }

    command.defer_ephemeral(&ctx).await?;

    let values = command
        .data
        .options
        .iter()
        .map(|o| (o.name.clone(), o.value.clone()))
        .collect::<HashMap<_, _>>();

    let user = values["user"].as_user_id().unwrap();
    let reason = values["reason"].as_str().unwrap().to_string();

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

    warn(ctx, user, &reason, None, &command.user, pool).await?;

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("<#1209471689264603167>")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

pub async fn modal(ctx: &Context, interaction: &ModalInteraction, pool: &SqlitePool) -> Result<()> {
    interaction.defer_ephemeral(&ctx).await?;

    let inputs = interaction
        .data
        .components
        .iter()
        .flat_map(|r| &r.components)
        .filter_map(|c| match c {
            ActionRowComponent::InputText(i) => Some(i),
            _ => None,
        })
        .map(|i| (i.custom_id.clone(), i.value.clone().unwrap()))
        .collect::<HashMap<_, _>>();

    let user = UserId::new(inputs["user"].parse()?);
    if user == 246684413075652612 {
        interaction
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new()
                    .content("Nice try nerd")
                    .ephemeral(true),
            )
            .await?;

        return Ok(());
    }

    let reason = inputs["reason"].clone();
    let warned_message = match (inputs.get("message"), inputs.get("channel")) {
        (Some(message_id), Some(channel_id)) => {
            let channel_id = ChannelId::new(channel_id.parse()?);
            let message_id = MessageId::new(message_id.parse()?);

            let message = {
                let message_cache_lock = {
                    let data = ctx.data.read().await;

                    data.get::<MessageCache>().unwrap().clone()
                };

                let message_cache = message_cache_lock.lock().await;

                if let Some(message) = message_cache.get(&(channel_id.get(), message_id.get())) {
                    message.clone()
                } else {
                    channel_id.message(&ctx, message_id).await?
                }
            };

            let _ = message.delete(&ctx).await;

            Some(message_to_warned_message(&message))
        }
        _ => None,
    };

    warn(ctx, user, &reason, warned_message, &interaction.user, pool).await?;

    interaction
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("<#1209471689264603167>")
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

async fn warn(
    ctx: &Context,
    user: UserId,
    reason: &str,
    warned_message: Option<WarnedMessage>,
    moderator: &User,
    pool: &SqlitePool,
) -> Result<()> {
    let dm_channel = user.create_dm_channel(ctx).await?;

    let dmed = dm_channel
        .send_message(
            ctx,
            CreateMessage::new().content(format!("You have been warned for: {}", reason)),
        )
        .await
        .is_ok();

    let create_message = match &warned_message {
        Some(warned_message) => Some(warned_message_to_create_message(ctx, warned_message).await),
        None => None,
    };

    if dmed {
        if let Some(create_message) = create_message.clone() {
            dm_channel
                .send_message(ctx, CreateMessage::new().content("Violating message:"))
                .await?;
            dm_channel.send_message(ctx, create_message).await?;
        }
    }

    let user = ctx.http.get_user(user).await?;
    let channel = ChannelId::new(1209471689264603167);

    let db_warn = database::DbWarn {
        id: 0,
        user: user.id.get() as i64,
        moderator: moderator.id.get() as i64,
        reason: reason.to_string(),
        dm: dmed,
        message: warned_message.map(|m| serde_json::to_string(&m).unwrap()),
    };

    let id = database::set_warn(db_warn, pool).await?;

    channel
        .send_message(
            ctx,
            CreateMessage::new().embed(
                CreateEmbed::default()
                    .color(0xff0000)
                    .title(format!("Warned {}!", user.name))
                    .field("Id", id.to_string(), true)
                    .field("User", user.mention().to_string(), true)
                    .field("Reason", reason, true)
                    .field("Dm", if dmed { "✅" } else { "❌" }, true)
                    .field(
                        "Message",
                        if create_message.is_some() {
                            "✅"
                        } else {
                            "❌"
                        },
                        true,
                    )
                    .footer(CreateEmbedFooter::new(format!(
                        "Warned by {}",
                        moderator.name
                    ))),
            ),
        )
        .await?;

    if let Some(create_message) = create_message.clone() {
        channel
            .send_message(ctx, CreateMessage::new().content("Violating message:"))
            .await?;
        channel.send_message(ctx, create_message).await?;
    }

    Ok(())
}

async fn message(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let message = command.data.resolved.messages.values().next().unwrap();

    command
        .create_response(
            ctx,
            CreateInteractionResponse::Modal(
                CreateModal::new("warn", "Warn a user message").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                            .placeholder("Short and precise reason"),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "User", "user")
                            .value(message.author.id.to_string()),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Message", "message")
                            .value(message.id.to_string()),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Channel", "channel")
                            .value(message.channel_id.to_string()),
                    ),
                ]),
            ),
        )
        .await?;

    Ok(())
}

async fn user(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let user = command.data.resolved.users.values().next().unwrap();

    command
        .create_response(
            ctx,
            CreateInteractionResponse::Modal(
                CreateModal::new("warn", "Warn a user message").components(vec![
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                            .placeholder("Short and precise reason"),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "User", "user")
                            .value(user.id.to_string()),
                    ),
                ]),
            ),
        )
        .await?;

    Ok(())
}

fn message_to_warned_message(message: &Message) -> WarnedMessage {
    let mut attachments: Vec<_> = message.attachments.iter().map(|a| a.url.clone()).collect();

    let mut sanitized_content_parts = Vec::new();

    let content = message.content.replace('|', "");
    let content_parts = content.split_whitespace().collect::<Vec<_>>();
    for content_part in content_parts {
        if content_part.starts_with("http") {
            if content_part.ends_with("png")
                || content_part.ends_with("jpg")
                || content_part.ends_with("jpeg")
                || content_part.ends_with("webp")
            {
                attachments.push(content_part.to_string());
            } else {
                sanitized_content_parts.push(format!("||{content_part}||"));
            }
        } else {
            sanitized_content_parts.push(content_part.to_string());
        }
    }

    let content = sanitized_content_parts.join(" ");

    WarnedMessage {
        content,
        attachments,
    }
}

async fn warned_message_to_create_message(ctx: &Context, message: &WarnedMessage) -> CreateMessage {
    let mut create_attachments = Vec::new();

    for attachment in &message.attachments {
        let mut create_attachment = CreateAttachment::url(&ctx, attachment).await.unwrap();
        if !create_attachment.filename.starts_with("SPOILER_") {
            create_attachment.filename = format!("SPOILER_{}", create_attachment.filename);
        }
        create_attachments.push(create_attachment);
    }

    CreateMessage::new()
        .content(&message.content)
        .files(create_attachments)
}
