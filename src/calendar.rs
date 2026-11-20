use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Days, NaiveDateTime, Offset, TimeZone, Utc};
use sqlx::{PgPool, Row};

use crate::{bad, db, mail, missing, valid_token, ApiError};

pub async fn download(
    State(pool): State<PgPool>,
    Path(public): Path<String>,
) -> Result<Response, ApiError> {
    if !valid_token(&public) {
        return Err(missing());
    }
    let event = sqlx::query(
        "SELECT e.title, e.location, e.description, e.timezone, e.updated_at,
                s.slot_date, s.start_time, s.end_time
         FROM events e JOIN event_slots s ON s.id=e.confirmed_slot_id
         WHERE e.public_token=$1 AND e.status='CONFIRMED'",
    )
    .bind(&public)
    .fetch_optional(&pool)
    .await
    .map_err(db)?
    .ok_or_else(missing)?;

    let timezone = event
        .get::<String, _>("timezone")
        .parse()
        .map_err(|_| bad())?;
    let date: chrono::NaiveDate = event.get("slot_date");
    let start = to_utc(timezone, date.and_time(event.get("start_time"))).ok_or_else(bad)?;
    let end = event
        .get::<Option<chrono::NaiveTime>, _>("end_time")
        .and_then(|time| to_utc(timezone, date.and_time(time)))
        .filter(|end| *end > start);
    let stamp: DateTime<Utc> = event.get("updated_at");

    let mut lines = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//Frizzi//Event Calendar//EN".into(),
        "BEGIN:VEVENT".into(),
        format!("UID:{public}@frizzi"),
        format!("DTSTAMP:{}", timestamp(stamp)),
        format!("DTSTART:{}", timestamp(start)),
    ];
    if let Some(end) = end {
        lines.push(format!("DTEND:{}", timestamp(end)));
    }
    for (property, value) in [
        ("SUMMARY", event.get::<String, _>("title")),
        ("LOCATION", event.get::<String, _>("location")),
        (
            "DESCRIPTION",
            event
                .get::<Option<String>, _>("description")
                .unwrap_or_default(),
        ),
        ("URL", format!("{}/{public}", mail::base_url())),
    ] {
        lines.push(format!("{property}:{}", escape_text(&value)));
    }
    lines.extend(["END:VEVENT".into(), "END:VCALENDAR".into()]);
    let mut body = String::new();
    for line in lines {
        let mut column = 0;
        for character in line.chars() {
            if column + character.len_utf8() > 75 {
                body.push_str("\r\n ");
                column = 1;
            }
            body.push(character);
            column += character.len_utf8();
        }
        body.push_str("\r\n");
    }
    Ok((
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"frizzi.ics\"",
            ),
        ],
        body,
    )
        .into_response())
}

fn timestamp(date: DateTime<Utc>) -> String {
    date.format("%Y%m%dT%H%M%SZ").to_string()
}

fn to_utc(timezone: chrono_tz::Tz, time: NaiveDateTime) -> Option<DateTime<Utc>> {
    if let Some(date) = timezone.from_local_datetime(&time).earliest() {
        return Some(date.to_utc());
    }
    // iCalendar interprets times skipped by a DST jump using the previous offset.
    let before = timezone
        .from_local_datetime(&time.checked_sub_days(Days::new(1))?)
        .earliest()?;
    Some(
        time.and_local_timezone(before.offset().fix())
            .single()?
            .to_utc(),
    )
}

fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}
