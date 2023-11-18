use anyhow::{anyhow, Result};
use serde_json::json;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        RoleId,
    },
    prelude::Context,
};
use sqlx::SqlitePool;
use url::Url;

use crate::database;

pub const NAME: &str = "rolestats";

pub async fn command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    pool: &SqlitePool,
) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(
                    |d: &mut serenity::builder::CreateInteractionResponseData<'_>| {
                        d.ephemeral(true)
                    },
                )
        })
        .await?;

    let Some(guild) = command.guild_id else {
        return Err(anyhow!("This command needs to be used in a guild"));
    };

    let role_ids = database::get_roles(pool)
        .await?
        .iter()
        .filter(|rd| rd.chives >= 0)
        .map(|rd| rd.role as u64)
        .collect::<Vec<_>>();

    let mut members = guild.members(&ctx, None, None).await?;
    members.retain(|m| m.roles.iter().any(|r| role_ids.contains(&r.0)));

    let roles = guild.roles(&ctx).await?;

    let mut data = Vec::new();
    let mut background_color = Vec::new();
    let mut labels = Vec::new();

    for role_id in role_ids {
        let count = members
            .iter()
            .filter(|m| m.roles.iter().any(|r| r.0 == role_id))
            .count();

        if count == 0 {
            continue;
        }

        data.push(count);

        let role = &roles[&RoleId(role_id)];
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
        .create_followup_message(ctx, |m| {
            m.content(
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
            )
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Shows the role distribution of this server")
}
