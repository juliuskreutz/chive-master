use std::iter;

use anyhow::{anyhow, Result};
use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{
        CreateAttachment, CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Permissions,
};
use sqlx::{Column, Row, SqlitePool, TypeInfo};
use tabled::{builder::Builder, settings::Style};

pub struct Sql;

impl super::Listener for Sql {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name)
            .description("Sql command")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "query", "Query")
                    .required(true),
            )
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }

    async fn command(ctx: &Context, command: &CommandInteraction, pool: &SqlitePool) -> Result<()> {
        let user = command.user.id.get() as i64;

        if user != 246684413075652612 {
            command
                .create_response(
                    &ctx,
                    CreateInteractionResponse::Defer(
                        CreateInteractionResponseMessage::new().ephemeral(true),
                    ),
                )
                .await?;
            return Err(anyhow!("Nope"));
        }

        command
            .create_response(
                &ctx,
                CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
            )
            .await?;

        let query = command.data.options[0].value.as_str().unwrap();

        let rows = sqlx::query(query).fetch_all(pool).await?;

        if rows.is_empty() {
            command
                .create_followup(
                    &ctx,
                    CreateInteractionResponseFollowup::new()
                        .content("<:MarchHype:1108800996043403304>"),
                )
                .await?;

            return Ok(());
        }

        let headers = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect::<Vec<_>>();

        let values = rows
            .iter()
            .map(|r| {
                (0..r.len())
                    .map(|i| match r.column(i).type_info().name() {
                        "NULL" => "NULL".to_string(),
                        "TEXT" | "BLOB" => r.get::<String, _>(i),
                        "REAL" => r.get::<f64, _>(i).to_string(),
                        "INTEGER" | "NUMERIC" => r.get::<i64, _>(i).to_string(),
                        "BOOLEAN" => r.get::<bool, _>(i).to_string(),
                        "DATE" => r.get::<chrono::NaiveDate, _>(i).to_string(),
                        "TIME" => r.get::<chrono::NaiveTime, _>(i).to_string(),
                        "DATETIME" => r.get::<chrono::NaiveDateTime, _>(i).to_string(),
                        _ => "Unknown".to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let result = Builder::from_iter(iter::once(headers).chain(values))
            .build()
            .with(Style::modern())
            .to_string();

        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new()
                    .add_file(CreateAttachment::bytes(result, "result.txt")),
            )
            .await?;

        Ok(())
    }
}
