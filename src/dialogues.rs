use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use serde::Deserialize;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType, ChannelId,
        InteractionResponseType,
    },
    prelude::{Context, TypeMap, TypeMapKey},
    CacheAndHttp, Client,
};
use tokio::sync::RwLock;

struct Dialogues;

// '1001' - Hot-Blooded Trailblazer - 28 dialogue choices
// '1002' - Pessimistic Trailblazer - 21 dialogue choices
// '1003' - Apologetic Trailblazer - 14 dialogue choices
// '1004' - Silent Trailblazer - 18 dialogue choices
// '2001' - The Echoer - 12 dialogue choices
// '3001' - The Meaning of Choice - 5 dialogue choices

impl TypeMapKey for Dialogues {
    type Value = Arc<RwLock<HashMap<u64, HashSet<String>>>>;
}

pub async fn init(client: &Client) -> Result<()> {
    let dialogues = Arc::new(RwLock::new(get().await?));

    {
        let mut data = client.data.write().await;

        data.insert::<Dialogues>(dialogues.clone());
    }

    let data = client.data.clone();
    let cache = client.cache_and_http.clone();
    tokio::spawn(async move {
        let minutes = 5;

        let mut timer = tokio::time::interval(Duration::from_secs(60 * minutes));

        loop {
            timer.tick().await;

            if let Err(e) = update(&data, &cache).await {
                crate::updater::log(&format!("Dialogues {}", e), &cache).await;
            }

            crate::updater::log("Updated dialogues! Next in 5min", &cache).await;
        }
    });

    Ok(())
}

async fn update(data: &Arc<RwLock<TypeMap>>, cache: &Arc<CacheAndHttp>) -> Result<()> {
    let channel = ChannelId(1107084275192447036);
    let new_dialogues = get().await?;

    let old_dialogues = {
        let data = data.read().await;

        let dialogues_lock = data.get::<Dialogues>().unwrap().read().await;

        dialogues_lock.clone()
    };

    if old_dialogues[&1001] != new_dialogues[&1001] {
        channel
            .send_message(&cache.http, |m| {
                m.content(
                    "YOYO. New hot blooded dialogues dropped. Check them out with `/dialogues hot`",
                )
            })
            .await?;
    }

    if old_dialogues[&1002] != new_dialogues[&1002] {
        channel
            .send_message(&cache.http, |m| m.content("YOYO. New pessimistic dialogues dropped. Check them out with `/dialogues pessimistic`"))
            .await?;
    }

    if old_dialogues[&1003] != new_dialogues[&1003] {
        channel
            .send_message(&cache.http, |m| m.content("YOYO. New apologetic dialogues dropped. Check them out with `/dialogues apologetic`"))
            .await?;
    }

    if old_dialogues[&1004] != new_dialogues[&1004] {
        channel
            .send_message(&cache.http, |m| {
                m.content(
                    "YOYO. New silent dialogues dropped. Check them out with `/dialogues silent`",
                )
            })
            .await?;
    }

    if old_dialogues[&2001] != new_dialogues[&2001] {
        channel
            .send_message(&cache.http, |m| {
                m.content(
                    "YOYO. New echoer dialogues dropped. Check them out with `/dialogues echoer`",
                )
            })
            .await?;
    }

    if old_dialogues[&3001] != new_dialogues[&3001] {
        channel
            .send_message(&cache.http, |m| m.content("YOYO. New GENDER dialogues dropped!!!!!!!! Check them out with `/dialogues gender`"))
            .await?;
    }

    {
        let data = data.read().await;

        let mut dialogues_lock = data.get::<Dialogues>().unwrap().write().await;

        *dialogues_lock = new_dialogues;
    }

    Ok(())
}

#[derive(Deserialize)]
struct InclinationText {
    #[serde(rename = "TalkSentenceID")]
    talk_sequence_id: u64,
    #[serde(rename = "InclinationTypeList")]
    inclination_type_list: Vec<u64>,
}

#[derive(Deserialize)]
struct TalkSentenceConfig {
    #[serde(rename = "TalkSentenceText")]
    talk_sentence_hash: TextHash,
}

#[derive(Deserialize)]
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

async fn get() -> Result<HashMap<u64, HashSet<String>>> {
    let url = "https://raw.githubusercontent.com/Dimbreath/StarRailData/master";

    let text_map: HashMap<String, String> = reqwest::get(&format!("{url}/TextMap/TextMapEN.json"))
        .await?
        .json()
        .await?;

    let inclination_text: HashMap<String, InclinationText> =
        reqwest::get(&format!("{url}/ExcelOutput/InclinationText.json"))
            .await?
            .json()
            .await?;

    let talk_sentence_config: HashMap<String, TalkSentenceConfig> =
        reqwest::get(&format!("{url}/ExcelOutput/TalkSentenceConfig.json"))
            .await?
            .json()
            .await?;

    let mut dialogues: HashMap<_, HashSet<_>> = HashMap::new();

    let inclinations = [1001, 1002, 1003, 1004, 2001, 3001];

    for inclination in inclinations {
        for inclination_text in inclination_text
            .values()
            .filter(|it| it.inclination_type_list.contains(&inclination))
        {
            let talk_sentence_config =
                &talk_sentence_config[&inclination_text.talk_sequence_id.to_string()];

            let text = text_map[&talk_sentence_config.talk_sentence_hash.hash.to_string()].clone();

            dialogues.entry(inclination).or_default().insert(text);
        }
    }

    Ok(dialogues)
}

pub const NAME: &str = "dialogues";

pub async fn command(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
    command
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|d| d.ephemeral(true))
        })
        .await?;

    let inclination = match command.data.options[0].name.as_str() {
        "hot" => 1001,
        "pessimistic" => 1002,
        "apologetic" => 1003,
        "silent" => 1004,
        "echoer" => 2001,
        "gender" => 3001,
        _ => unreachable!(),
    };

    let dialogues = {
        let data = ctx.data.read().await;

        let dialogues_lock = data.get::<Dialogues>().unwrap().read().await;

        dialogues_lock[&inclination].clone()
    };

    command
        .create_followup_message(ctx, |m| {
            m.content(dialogues.into_iter().collect::<Vec<_>>().join("\n"))
                .ephemeral(true)
        })
        .await?;

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(NAME)
        .description("Dialoges")
        .create_option(|o| {
            o.name("hot")
                .description("Hot Blooded")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("pessimistic")
                .description("Pessimistic")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("apologetic")
                .description("Apologetic")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("silent")
                .description("Silent")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("echoer")
                .description("Echoer")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|o| {
            o.name("gender")
                .description("Gender")
                .kind(CommandOptionType::SubCommand)
        })
}
