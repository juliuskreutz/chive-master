use anyhow::Result;
use chrono::{Duration, NaiveDateTime, Utc};

pub fn by_uid(uid: i64) -> Result<NaiveDateTime> {
    let date = Utc::now()
        + match uid.to_string().chars().next() {
            Some('6') => Duration::hours(-5),
            Some('7') => Duration::hours(1),
            _ => Duration::hours(8),
        };

    Ok(date.naive_utc())
}
