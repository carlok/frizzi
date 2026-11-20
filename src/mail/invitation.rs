use crate::{db, ApiError};
use serde_json::{json, Value};
use sqlx::{types::Json, PgPool, Postgres, Row, Transaction};
use std::{fmt::Write, process::Stdio, time::Duration};
use tokio::{io::AsyncWriteExt, process::Command};
use uuid::Uuid;

use super::template::Details;

pub(super) const FILENAME: &str = "invito-frizzi.pdf";

pub(super) async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    event: Uuid,
    lang: &str,
    title: &str,
    organizer: &str,
    intro: Option<&str>,
    details: &Details,
) -> Result<Option<Uuid>, ApiError> {
    if std::env::var_os("FRIZZI_TYPST").is_none_or(|value| value.is_empty()) {
        return Ok(None);
    }
    let default_intro = rust_i18n::t!("mail.confirmedBody", locale = lang);
    let payload = json!({
        "lang": lang,
        "title": title,
        "organizer": organizer,
        "headline": rust_i18n::t!("mail.confirmedSubject", locale = lang),
        "intro": intro.unwrap_or(&default_intro),
        "description": details.description,
        "location": details.location,
        "dates": details.dates,
        "timezone": details.timezone,
        "directions": details.directions.as_str(),
        "website": details.website.as_ref().map(reqwest::Url::as_str),
        "labels": {
            "invitation": rust_i18n::t!("pdf.invitation", locale = lang),
            "when": rust_i18n::t!("pdf.when", locale = lang),
            "where": rust_i18n::t!("pdf.where", locale = lang),
            "timezone": rust_i18n::t!("ui.timezone", locale = lang),
            "directions": rust_i18n::t!("mail.directions", locale = lang),
            "website": rust_i18n::t!("mail.venueWebsite", locale = lang),
            "organizer": rust_i18n::t!("ui.organized", locale = lang),
            "description": rust_i18n::t!("ui.description", locale = lang),
        },
    });
    sqlx::query_scalar(
        "INSERT INTO confirmation_invitations (event_id,payload) VALUES ($1,$2) RETURNING id",
    )
    .bind(event)
    .bind(Json(payload))
    .fetch_one(&mut **tx)
    .await
    .map(Some)
    .map_err(db)
}

pub(super) async fn attachment(pool: &PgPool, id: Uuid) -> Result<Option<Vec<u8>>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    // Serialize rendering across recipients and retain the result for retries.
    let row =
        sqlx::query("SELECT payload,pdf FROM confirmation_invitations WHERE id=$1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let mut pdf = None;
    if let Some(row) = row {
        if let Some(Json(payload)) = row.get::<Option<Json<Value>>, _>("payload") {
            match render(&payload).await {
                Ok(attachment) => pdf = Some(attachment),
                Err(reason) => tracing::warn!(event = "invitation_pdf_fallback", reason),
            }
            sqlx::query("UPDATE confirmation_invitations SET payload=NULL,pdf=$2 WHERE id=$1")
                .bind(id)
                .bind(&pdf)
                .execute(&mut *tx)
                .await?;
        } else {
            pdf = row.get("pdf");
        }
    }
    tx.commit().await?;
    Ok(pdf)
}

async fn render(payload: &Value) -> Result<Vec<u8>, &'static str> {
    let executable = std::env::var_os("FRIZZI_TYPST")
        .filter(|path| !path.is_empty())
        .ok_or("typst_unconfigured")?;
    let mut command = Command::new(executable);
    command
        .args(["compile", "--root", ".", "--ignore-system-fonts"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Some(fonts) = std::env::var_os("FRIZZI_PDF_FONTS").filter(|path| !path.is_empty()) {
        command.arg("--font-path").arg(fonts);
    }
    command.args(["-", "-"]);
    let source = source(payload);
    let output = tokio::time::timeout(Duration::from_secs(6), async {
        let mut child = command.spawn().map_err(|_| "typst_start_failed")?;
        let mut stdin = child.stdin.take().ok_or("typst_input_unavailable")?;
        stdin
            .write_all(source.as_bytes())
            .await
            .map_err(|_| "typst_input_failed")?;
        drop(stdin);
        child
            .wait_with_output()
            .await
            .map_err(|_| "typst_output_failed")
    })
    .await
    .map_err(|_| "timeout")??;
    if !output.status.success() {
        return Err("typst_compile_failed");
    }
    if !output.stdout.starts_with(b"%PDF-") || output.stdout.len() > 512 * 1024 {
        return Err("invalid_pdf");
    }
    Ok(output.stdout)
}

fn source(payload: &Value) -> String {
    // Treat the snapshot as bytes so event text cannot become Typst source code.
    let mut source = String::from("#let invitation = json(bytes((");
    for byte in payload.to_string().bytes() {
        let _ = write!(source, "{byte},");
    }
    source.push_str(")))\n");
    source.push_str(include_str!("invitation.typ"));
    source
}
