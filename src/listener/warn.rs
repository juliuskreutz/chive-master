use anyhow::Result;
use serenity::{
    all::{
        ActionRowComponent, ChannelId, Color, CommandInteraction, CommandOptionType, CommandType,
        CreateActionRow, CreateAttachment, CreateEmbedAuthor, CreateInputText, CreateModal,
        InputTextStyle, Mentionable, Message, MessageId, ModalInteraction, Timestamp, User, UserId,
    },
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use std::collections::HashMap;

use crate::GUILD_ID;

#[derive(serde::Serialize, serde::Deserialize)]
struct WarnedMessage {
    content: String,
    embeds: Vec<Embed>,
    attachments: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Embed {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<Timestamp>,
    color: Option<Color>,
    footer: Option<Footer>,
    image: Option<String>,
    thumbnail: Option<String>,
    author: Option<Author>,
    fields: Vec<Field>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Footer {
    text: String,
    icon_url: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Author {
    name: String,
    icon_url: Option<String>,
    url: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Field {
    name: String,
    value: String,
    inline: bool,
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
            .add_option(CreateCommandOption::new(
                CommandOptionType::Integer,
                "message",
                "Message",
            ))
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

    let values = command
        .data
        .options
        .iter()
        .map(|o| (o.name.clone(), o.value.clone()))
        .collect::<HashMap<_, _>>();

    let user = values["user"].as_user_id().unwrap();
    let reason = values["reason"].as_str().unwrap().to_string();
    let message = values.get("message").map(|m| m.as_i64().unwrap());

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

    command.defer_ephemeral(&ctx).await?;

    let warned_message = if let Some(message) = message {
        get_warned_message(ctx, message as u64).await?
    } else {
        None
    };

    warn(ctx, user, &reason, warned_message, &command.user, pool).await?;

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
    interaction.defer_ephemeral(&ctx).await?;

    let reason = inputs["reason"].clone();
    let warned_message = if let Some(message) = inputs.get("message") {
        get_warned_message(ctx, message.parse()?).await?
    } else {
        None
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
    _: &SqlitePool,
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

    if let Some(create_message) = create_message.clone() {
        let _ = dm_channel.send_message(ctx, create_message).await;
    }

    let user = ctx.http.get_user(user).await?;
    let channel = ChannelId::new(1209471689264603167);

    channel
        .send_message(
            ctx,
            CreateMessage::new().embed(
                CreateEmbed::default()
                    .color(0xff0000)
                    .title(format!("Warned {}!", user.name))
                    .field("User", user.mention().to_string(), true)
                    .field("Reason", reason, true)
                    .field("Dm", if dmed { "✅" } else { "❌" }, false)
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
                        CreateInputText::new(InputTextStyle::Short, "User", "user")
                            .value(message.author.id.to_string()),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                            .placeholder("Short and precise reason"),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, "Message", "message")
                            .value(message.id.to_string()),
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
                        CreateInputText::new(InputTextStyle::Short, "User", "user")
                            .value(user.id.to_string()),
                    ),
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Paragraph, "Reason", "reason")
                            .placeholder("Short and precise reason"),
                    ),
                ]),
            ),
        )
        .await?;

    Ok(())
}

async fn get_warned_message(ctx: &Context, message_id: u64) -> Result<Option<WarnedMessage>> {
    for channel in GUILD_ID.channels(&ctx).await?.values() {
        if let Ok(message) = channel.message(&ctx, MessageId::new(message_id)).await {
            message.delete(&ctx).await?;
            return Ok(Some(message_to_warned_message(&message)));
        }
    }

    Ok(None)
}

fn message_to_warned_message(message: &Message) -> WarnedMessage {
    let content = message.content.clone();

    let mut embeds = Vec::new();
    for embed in &message.embeds {
        if embed.kind.as_deref() != Some("rich") {
            continue;
        }

        let title = embed.title.clone();
        let description = embed.description.clone();
        let url = embed.url.clone();
        let timestamp = embed.timestamp;
        let color = embed.colour;
        let footer = embed.footer.clone().map(|f| Footer {
            text: f.text,
            icon_url: f.icon_url,
        });
        let image = embed.image.clone().map(|i| i.url);
        let thumbnail = embed.thumbnail.clone().map(|t| t.url);
        let author = embed.author.clone().map(|a| Author {
            name: a.name,
            icon_url: a.icon_url,
            url: a.url,
        });
        let fields = embed
            .fields
            .iter()
            .map(|f| Field {
                name: f.name.clone(),
                value: f.value.clone(),
                inline: f.inline,
            })
            .collect();

        embeds.push(Embed {
            title,
            description,
            url,
            timestamp,
            color,
            footer,
            image,
            thumbnail,
            author,
            fields,
        });
    }

    let attachments = message.attachments.iter().map(|a| a.url.clone()).collect();

    WarnedMessage {
        content,
        embeds,
        attachments,
    }
}

async fn warned_message_to_create_message(ctx: &Context, message: &WarnedMessage) -> CreateMessage {
    let mut create_embeds = Vec::new();

    for embed in &message.embeds {
        let mut create_embed = CreateEmbed::new();

        if let Some(title) = &embed.title {
            create_embed = create_embed.title(title);
        }

        if let Some(description) = &embed.description {
            create_embed = create_embed.description(description);
        }

        if let Some(url) = &embed.url {
            create_embed = create_embed.url(url);
        }

        if let Some(timestamp) = embed.timestamp {
            create_embed = create_embed.timestamp(timestamp);
        }

        if let Some(color) = embed.color {
            create_embed = create_embed.color(color);
        }

        if let Some(footer) = &embed.footer {
            let mut create_footer = CreateEmbedFooter::new(&footer.text);

            if let Some(icon_url) = &footer.icon_url {
                create_footer = create_footer.icon_url(icon_url);
            }

            create_embed = create_embed.footer(create_footer);
        }

        if let Some(image) = &embed.image {
            create_embed = create_embed.image(image);
        }

        if let Some(thumbnail) = &embed.thumbnail {
            create_embed = create_embed.thumbnail(thumbnail);
        }

        if let Some(author) = &embed.author {
            let mut create_author = CreateEmbedAuthor::new(&author.name);

            if let Some(icon_url) = &author.icon_url {
                create_author = create_author.icon_url(icon_url);
            }

            if let Some(url) = &author.url {
                create_author = create_author.url(url);
            }

            create_embed = create_embed.author(create_author);
        }

        for field in &embed.fields {
            create_embed = create_embed.field(&field.name, &field.value, field.inline);
        }

        create_embeds.push(create_embed);
    }

    let mut create_attachments = Vec::new();

    for attachment in &message.attachments {
        create_attachments.push(CreateAttachment::url(&ctx, attachment).await.unwrap());
    }

    CreateMessage::new()
        .content(&message.content)
        .embeds(create_embeds)
        .files(create_attachments)
}
