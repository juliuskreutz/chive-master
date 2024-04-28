use std::sync::Arc;

use anyhow::Result;
use serenity::all::{
    ChannelId, CreateEmbed, CreateEmbedAuthor, CreateMessage, Http, Mentionable, RoleId,
};
use sqlx::SqlitePool;

use crate::database;

#[derive(serde::Deserialize)]
struct Json<T> {
    data: Data<T>,
}

#[derive(serde::Deserialize)]
struct Data<T> {
    list: Vec<T>,
}

#[derive(serde::Deserialize)]
struct Event {
    id: String,
    name: String,
    start: String,
    end: String,
    banner_url: String,
    desc: String,
    web_path: String,
    create_at: String,
}

#[derive(serde::Deserialize)]
enum ArticleType {
    Notice,
    Info,
}

#[derive(serde::Deserialize)]
struct Article {
    post: Post,
    image_list: Vec<Image>,
}

#[derive(serde::Deserialize)]
struct Post {
    post_id: String,
    subject: String,
    content: String,
    created_at: i64,
}

#[derive(serde::Deserialize)]
struct Image {
    url: String,
}

pub async fn update(http: &Arc<Http>, pool: &SqlitePool) -> Result<()> {
    let client = reqwest::Client::new();

    let json_events: Json<Event> = client.get("https://bbs-api-os.hoyolab.com/community/community_contribution/wapi/event/list?gids=6&page_size=15&size=15").header("x-rpc-client_type", "4").send().await.unwrap().json().await.unwrap();

    let role = RoleId::new(1229730323672338462);

    let channel = ChannelId::new(1229514777781473492);
    for event in json_events.data.list.iter().rev() {
        let id = event.id.parse().unwrap();

        if database::get_post_by_id(id, pool).await.is_ok() {
            continue;
        }

        database::set_post(database::DbPost { id }, pool).await?;

        let url = if event.web_path.starts_with("http") {
            event.web_path.clone()
        } else {
            format!("https://www.hoyolab.com{}", event.web_path)
        };

        let embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new("Event"))
            .color(0x06d6a0)
            .title(&event.name)
            .url(url)
            .description(&event.desc)
            .thumbnail(&event.banner_url)
            .field("Posted", format!("<t:{}:R>", event.create_at), true)
            .field("Start", format!("<t:{}:R>", event.start), true)
            .field("End", format!("<t:{}:R>", event.end), true);

        channel
            .send_message(
                http,
                CreateMessage::new().content(format!("{}", role.mention())),
            )
            .await?;
        channel
            .send_message(http, CreateMessage::new().embed(embed))
            .await?;
    }

    let json_notices: Json<Article> = client.get("https://bbs-api-os.hoyolab.com/community/post/wapi/getNewsList?gids=6&page_size=15&type=1").send().await.unwrap().json().await.unwrap();
    let json_infos: Json<Article> = client.get("https://bbs-api-os.hoyolab.com/community/post/wapi/getNewsList?gids=6&page_size=15&type=3").send().await.unwrap().json().await.unwrap();

    let mut articles = json_notices
        .data
        .list
        .into_iter()
        .map(|a| (a, ArticleType::Notice))
        .chain(
            json_infos
                .data
                .list
                .into_iter()
                .map(|a| (a, ArticleType::Info)),
        )
        .collect::<Vec<_>>();

    articles.sort_unstable_by_key(|(a, _)| a.post.created_at);

    let channel = ChannelId::new(1229466203538587689);
    for (article, article_type) in articles {
        let (title, color) = match article_type {
            ArticleType::Notice => ("Notice", 0xffd166),
            ArticleType::Info => ("Info", 0x118ab2),
        };

        let id = article.post.post_id.parse().unwrap();

        if database::get_post_by_id(id, pool).await.is_ok() {
            continue;
        }
        database::set_post(database::DbPost { id }, pool).await?;

        let url = format!("https://www.hoyolab.com/article/{}", article.post.post_id);

        let mut embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new(title))
            .color(color)
            .title(&article.post.subject)
            .url(url)
            .description(&article.post.content)
            .field(
                "Posted",
                format!("<t:{}:R>", article.post.created_at),
                false,
            );

        if let Some(image) = article.image_list.first() {
            embed = embed.thumbnail(&image.url);
        }

        channel
            .send_message(
                http,
                CreateMessage::new().content(format!("{}", role.mention())),
            )
            .await?;
        channel
            .send_message(http, CreateMessage::new().embed(embed))
            .await?;
    }

    Ok(())
}
