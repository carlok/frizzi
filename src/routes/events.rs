//! Creating an event and reading its public page.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::{bad, db, error, missing, ApiResult};
use crate::limits::{per_ip, rate_limits, ClientIp, CREATE_PER_IP, CREATE_PER_ORGANIZER};
use crate::mail::MailKind;
use crate::places::LocationGeo;
use crate::slots::{insert_slots, SlotInput};
use crate::status::Status;
use crate::validate::{
    clean_description, deadline_passed, deadline_within_slots, default_locale, email, locale,
    require_database_timezone, valid_token, validate_deadline, validate_details,
    validate_slot_dates,
};
use crate::{mail, schedule_mail, Runtime};

#[derive(Deserialize)]
pub(super) struct EventInput {
    title: String,
    description: Option<String>,
    location: String,
    location_geo: Option<LocationGeo>,
    organizer_name: String,
    organizer_email: String,
    #[serde(default = "default_locale")]
    locale: String,
    allow_multiple_choices: bool,
    timezone: String,
    voting_deadline: Option<NaiveDate>,
    slots: Vec<SlotInput>,
}
pub(super) async fn create(
    State(pool): State<PgPool>,
    runtime: Runtime,
    ip: ClientIp,
    Json(mut e): Json<EventInput>,
) -> ApiResult {
    let organizer_email = email(&e.organizer_email)?;
    e.description = clean_description(e.description.take());
    if !validate_details(
        &e.title,
        &e.description,
        &e.location,
        &e.organizer_name,
        &e.timezone,
        &e.slots,
    ) || e.slots.iter().any(|s| s.id.is_some())
    {
        return Err(bad());
    }
    validate_slot_dates(&e.slots, &[], &e.timezone)?;
    validate_deadline(e.voting_deadline, &e.timezone)?;
    if !deadline_within_slots(e.voting_deadline, &e.slots) {
        return Err(error(StatusCode::BAD_REQUEST, "invalid_deadline"));
    }
    if let Some(geo) = &e.location_geo {
        geo.validate()?;
    }
    let public = nanoid::nanoid!();
    let secret = nanoid::nanoid!();
    rate_limits(
        &pool,
        &[
            (ip.key("create"), per_ip(CREATE_PER_IP)),
            (format!("create:{organizer_email}"), CREATE_PER_ORGANIZER),
        ],
    )
    .await?;
    let mut tx = pool.begin().await.map_err(db)?;
    require_database_timezone(&mut tx, &e.timezone).await?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO events (public_token,admin_token,title,description,location,organizer_name,organizer_email,organizer_locale,allow_multiple_choices,timezone,voting_deadline,location_geo) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) RETURNING id")
        .bind(&public).bind(&secret).bind(e.title.trim()).bind(&e.description).bind(e.location.trim())
        .bind(e.organizer_name.trim()).bind(&organizer_email).bind(locale(&e.locale))
        .bind(e.allow_multiple_choices).bind(&e.timezone).bind(e.voting_deadline).bind(e.location_geo.map(sqlx::types::Json)).fetch_one(&mut *tx).await.map_err(db)?;
    insert_slots(&mut tx, id, &e.slots).await?;
    mail::enqueue(
        &mut tx,
        id,
        &organizer_email,
        locale(&e.locale),
        MailKind::Admin,
        &e.title,
        &format!("{}/{public}/admin/{secret}", mail::base_url()),
        None,
        None,
        None,
    )
    .await?;
    tx.commit().await.map_err(db)?;
    schedule_mail(pool, runtime);
    Ok(Json(json!({"public_token": public})))
}
pub(super) async fn public_event(
    State(pool): State<PgPool>,
    Path(public): Path<String>,
) -> ApiResult {
    get_event(&pool, &public).await
}
pub(super) async fn get_event(pool: &PgPool, public: &str) -> ApiResult {
    if !valid_token(public) {
        return Err(missing());
    }
    let e = sqlx::query("SELECT id,title,description,location,organizer_name,allow_multiple_choices,status::text,confirmed_slot_id,tiebreak_slot_id,timezone,voting_deadline,location_geo FROM events WHERE public_token=$1")
        .bind(public).fetch_optional(pool).await.map_err(db)?.ok_or_else(missing)?;
    let id: Uuid = e.get("id");
    let slots = sqlx::query("SELECT id,slot_date,start_time,end_time FROM event_slots WHERE event_id=$1 ORDER BY slot_date")
        .bind(id).fetch_all(pool).await.map_err(db)?.iter().map(|r| json!({"id":r.get::<Uuid,_>("id"),"date":r.get::<NaiveDate,_>("slot_date"),"start":r.get::<NaiveTime,_>("start_time"),"end":r.get::<Option<NaiveTime>,_>("end_time")})).collect::<Vec<_>>();
    let people = sqlx::query("SELECT p.id,p.name,COALESCE(array_agg(v.slot_id) FILTER (WHERE v.slot_id IS NOT NULL), '{}') AS slots FROM participants p LEFT JOIN votes v ON v.participant_id=p.id WHERE p.event_id=$1 GROUP BY p.id ORDER BY p.created_at")
        .bind(id).fetch_all(pool).await.map_err(db)?.iter().map(|r| json!({"name":r.get::<String,_>("name"),"slots":r.get::<Vec<Uuid>,_>("slots")})).collect::<Vec<_>>();
    Ok(Json(
        json!({"title":e.get::<String,_>("title"),"description":e.get::<Option<String>,_>("description"),"location":e.get::<String,_>("location"),"location_geo":e.get::<Option<sqlx::types::Json<LocationGeo>>,_>("location_geo"),"organizer_name":e.get::<String,_>("organizer_name"),"allow_multiple_choices":e.get::<bool,_>("allow_multiple_choices"),"status":e.get::<Status,_>("status"),"confirmed_slot_id":e.get::<Option<Uuid>,_>("confirmed_slot_id"),"tiebreak_slot_id":e.get::<Option<Uuid>,_>("tiebreak_slot_id"),"timezone":e.get::<String,_>("timezone"),"voting_deadline":e.get::<Option<NaiveDate>,_>("voting_deadline"),"voting_ended":deadline_passed(e.get("voting_deadline"),&e.get::<String,_>("timezone")),"slots":slots,"participants":people}),
    ))
}
