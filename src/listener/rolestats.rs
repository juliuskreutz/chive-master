use anyhow::Result;
use serde_json::json;
use serenity::{
    all::{CommandInteraction, RoleId},
    builder::{
        CreateCommand, CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage,
    },
    client::Context,
};
use sqlx::SqlitePool;
use url::Url;

use crate::{database, GUILD_ID};

pub struct Rolestats;

impl super::Listener for Rolestats {
    fn register(name: &str) -> CreateCommand {
        CreateCommand::new(name).description("Shows the role distribution of this server")
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

        let role_ids = database::get_roles(pool)
            .await?
            .iter()
            .filter(|rd| rd.chives >= 0)
            .map(|rd| rd.role as u64)
            .collect::<Vec<_>>();

        let mut members = GUILD_ID.members(&ctx, None, None).await?;
        members.retain(|m| m.roles.iter().any(|r| role_ids.contains(&r.get())));

        let roles = GUILD_ID.roles(&ctx).await?;

        let mut data = Vec::new();
        let mut background_color = Vec::new();
        let mut labels = Vec::new();

        for role_id in role_ids {
            let count = members
                .iter()
                .filter(|m| m.roles.iter().any(|r| r.get() == role_id))
                .count();

            if count == 0 {
                continue;
            }

            data.push(count);

            let role = &roles[&RoleId::new(role_id)];
            let color = format!(
                "rgb({}, {}, {})",
                role.colour.r(),
                role.colour.g(),
                role.colour.b()
            );
            background_color.push(color);

            let label = role.name.clone();
            labels.push(label);
        }

        command
            .create_followup(
                &ctx,
                CreateInteractionResponseFollowup::new().content(
                    Url::parse(&format!(
                        "https://quickchart.io/chart?bkg=%23ffffff&c={}",
                        json!({
                                            "type": "outlabeledPie",
                                            "data": {
                                                "datasets": [
                                                {
                                                    "data": data,
                                                    "backgroundColor": background_color
                                                }
                                                ],
                                                "labels": labels
                                            },
                                            "options": {
                                                "plugins": {
                                                    "legend": false,
                                                    "outlabels": {
                                                        "text": "%l %p",
                                                        "color": "white",
                                                        "font": {
                                                            "minSize": 8
                                                        }
                                                    }
                                                }
                        }
                                        })
                    ))
                    .unwrap(),
                ),
            )
            .await?;

        Ok(())
    }
}
