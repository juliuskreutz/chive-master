use std::time::Duration;

use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, UserId},
    builder::{
        CreateCommand, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::SqlitePool;

use crate::database;

pub fn register(name: &str, commands: &mut Vec<CreateCommand>) {
    commands.push(
        CreateCommand::new(name)
            .description("Disband this match")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .dm_permission(false),
    );
}

pub async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
    command
        .create_response(
            &ctx,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await?;

    let channel = command.channel_id.get() as i64;

    let Ok(db_match) = database::get_match_by_channel(channel, pool).await else {
        return Err(anyhow!(
            "This command has to be executed in a match channel"
        ));
    };

    database::delete_match_by_channel(channel, pool).await?;

    let text = "Your Support Contract with your partner has ended. If you would like to re-match with someone, please go back to https://discord.com/channels/1008493665116758167/1144488145228923020 and hit the Match button.";

    if let Ok(channel) = UserId::new(db_match.user1 as u64)
        .create_dm_channel(&ctx)
        .await
    {
        let _ = channel
            .send_message(ctx, CreateMessage::new().content(text))
            .await;
    }

    if let Ok(channel) = UserId::new(db_match.user2 as u64)
        .create_dm_channel(&ctx)
        .await
    {
        let _ = channel
            .send_message(ctx, CreateMessage::new().content(text))
            .await;
    }

    command
        .create_followup(
            &ctx,
            CreateInteractionResponseFollowup::new()
                .content("Disbanded match. Channel will be deleted in 5s"),
        )
        .await?;

    {
        let ctx = ctx.clone();
        let channel = command.channel_id;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let _ = channel.delete(&ctx).await;
        });
    }

    Ok(())
}
