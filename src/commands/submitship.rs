use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};
use sqlx::SqlitePool;

use crate::database::{self, DbShip};

pub const NAME: &str = "submitship";

const CHARACTERS: &[&str] = &[
    "Arlan",
    "Asta",
    "Bailu",
    "Blade",
    "Bronya",
    "Caelus",
    "Clara",
    "Dan Heng",
    "Fu Xuan",
    "Gepard",
    "Guinaifen",
    "Herta",
    "Himeko",
    "Hook",
    "Imbibitor Lunae",
    "Jing Yuan",
    "Jingliu",
    "Kafka",
    "Luka",
    "Luocha",
    "Lynx",
    "March 7th",
    "Natasha",
    "Pela",
    "Qingque",
    "Sampo",
    "Seele",
    "Serval",
    "Silver Wolf",
    "Stelle",
    "Sushang",
    "Svarog",
    "Tingyun",
    "Topaz",
    "Welt",
    "Yanqing",
    "Yukong",
];

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

    let input1 = command
        .data
        .options
        .get(0)
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let input2 = command
        .data
        .options
        .get(1)
        .and_then(|o| o.value.as_ref())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    if !CHARACTERS.contains(&input1.as_str()) {
        return Err(anyhow!(
            "{input1} is not a valid character. Please use the search function."
        ));
    }

    if !CHARACTERS.contains(&input2.as_str()) {
        return Err(anyhow!(
            "{input2} is not a valid character. Please use the search function."
        ));
    }

    let ship = if input1.cmp(&input2) == Ordering::Less {
        format!("{input1} x {input2}")
    } else {
        format!("{input2} x {input1}")
    };

    if input1 == input2 || ship == "Caelus x Stelle" || ship == "Dan Heng x Imbibitor Lunae" {
        return Err(anyhow!("Nice try degen :D"));
    }

    let db_ship = DbShip {
        user: command.user.id.0 as i64,
        ship: ship.clone(),
    };
    database::set_ship(&db_ship, pool).await?;

    command
        .create_followup_message(ctx, |m| {
            m.content(format!("Ship submitted! Your current ship is: {ship}"))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}

pub async fn autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) -> Result<()> {
    let mut focused1 = false;
    let mut input1 = "".to_string();

    if let Some(option) = autocomplete.data.options.get(0) {
        focused1 = option.focused;
        input1 = option
            .value
            .as_ref()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
    }

    let mut focused2 = false;
    let mut input2 = "".to_string();

    if let Some(option) = autocomplete.data.options.get(1) {
        focused2 = option.focused;
        input2 = option
            .value
            .as_ref()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
    }

    autocomplete
        .create_autocomplete_response(ctx, |r| {
            match (focused1, focused2) {
                (true, false) => {
                    for character in CHARACTERS
                        .iter()
                        .filter(|&&c| {
                            input2 != c && c.to_lowercase().contains(&input1.to_lowercase())
                        })
                        .take(25)
                    {
                        r.add_string_choice(character, character);
                    }
                }
                (false, true) => {
                    for character in CHARACTERS
                        .iter()
                        .filter(|&&c| {
                            input1 != c && c.to_lowercase().contains(&input2.to_lowercase())
                        })
                        .take(25)
                    {
                        r.add_string_choice(character, character);
                    }
                }
                _ => {}
            }

            r
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Ship your favourite characters")
        .create_option(|o| {
            o.name("character1")
                .description("Your first character")
                .kind(CommandOptionType::String)
                .set_autocomplete(true)
                .required(true)
        })
        .create_option(|o| {
            o.name("character2")
                .description("Your second character")
                .kind(CommandOptionType::String)
                .set_autocomplete(true)
                .required(true)
        })
}
