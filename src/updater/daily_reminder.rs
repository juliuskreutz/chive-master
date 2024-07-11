use std::sync::Arc;

use chrono::{Datelike, Month, Utc};
use serenity::all::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Http};
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn spawn(http: Arc<Http>) -> anyhow::Result<()> {
    let  scheduler = JobScheduler::new().await?;

    scheduler
        .add(Job::new_async("* 0 16 * * *", move |_,  _| {
            let http = http.clone();
            
            Box::pin(async move {
                let channel = ChannelId::new(1260702515008110734);

                let now = Utc::now();
                let month = Month::try_from(now.month() as u8).unwrap().name();
                let day = now.day();


                let embed = CreateEmbed::new().title("Daily Check-Ins:").description("- [Genshin Impact](<https://act.hoyolab.com/ys/event/signin-sea-v3/index.html?act_id=e202102251931481>)
- [Honkai: Star Rail](<https://act.hoyolab.com/bbs/event/signin/hkrpg/index.html?act_id=e202303301540311>)
- [Zenless Zone Zero](<https://act.hoyolab.com/bbs/event/signin/zzz/e202406031448091.html?act_id=e202406031448091>)").footer(CreateEmbedFooter::new(format!("{month} - Day {day}"))).color(0x2b9b7b);
                             
                channel.send_message(&http, CreateMessage::new().content("<@1260628582032605314>").embed(embed)).await.unwrap();
            })
        })?)
        .await?;

    scheduler.start().await?;

    Ok(())
}
