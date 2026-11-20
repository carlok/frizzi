use axum::{
    extract::{MatchedPath, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{Days, NaiveDate, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};
use tracing::Instrument;
use uuid::Uuid;

const DAILY_LIMIT: u64 = 2 * 1024 * 1024;
static LOG_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var_os("FRIZZI_LOG_DIR")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
});
static LOG_LOCK: Mutex<()> = Mutex::new(());
static UI_BUDGET: Mutex<Option<(Instant, u32)>> = Mutex::new(None);

pub fn init_tracing() {
    prune_logs();
    let _ = tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_env_filter(
            if std::env::var("FRIZZI_LOG_LEVEL").as_deref() == Ok("debug") {
                "off,frizzi=debug,local=debug"
            } else {
                "off,frizzi=info,local=info"
            },
        )
        .with_writer(LogWriterFactory)
        .try_init();
    std::panic::set_hook(Box::new(|panic| {
        tracing::error!(
            event = "panic",
            file = panic.location().map(|location| location.file()),
            line = panic.location().map(|location| location.line())
        );
    }));
}

struct LogWriterFactory;
struct LogWriter(u64);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogWriterFactory {
    type Writer = LogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        LogWriter(DAILY_LIMIT)
    }

    fn make_writer_for(&'a self, metadata: &tracing::Metadata<'_>) -> Self::Writer {
        // Reserve the end of each daily file for warnings and errors.
        LogWriter(match *metadata.level() {
            tracing::Level::ERROR => DAILY_LIMIT,
            tracing::Level::WARN => DAILY_LIMIT * 7 / 8,
            _ => DAILY_LIMIT * 3 / 4,
        })
    }
}

impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(dir) = LOG_DIR.as_ref() else {
            return io::stdout().write(bytes);
        };
        let _guard = LOG_LOCK
            .lock()
            .map_err(|_| io::Error::other("log writer unavailable"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(format!("{}.jsonl", Utc::now().date_naive())))?;
        if file.metadata()?.len() + bytes.len() as u64 <= self.0 {
            file.write_all(bytes)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn prune_logs() {
    let Some(dir) = LOG_DIR.as_ref() else { return };
    let result = fs::create_dir_all(dir).and_then(|_| {
        let _guard = LOG_LOCK
            .lock()
            .map_err(|_| io::Error::other("log writer unavailable"))?;
        prune_log_files(dir, Utc::now().date_naive())
    });
    if let Err(error) = result {
        eprintln!("Frizzi log rotation unavailable: {:?}", error.kind());
    }
}

fn prune_log_files(dir: &Path, today: NaiveDate) -> io::Result<()> {
    let cutoff = today - Days::new(9);
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let date = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok());
        if path.extension().is_some_and(|ext| ext == "jsonl")
            && date.is_some_and(|day| day < cutoff)
        {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn operation(method: &Method, route: &str) -> Option<&'static str> {
    Some(match (method.as_str(), route) {
        ("POST", "/api/events") => "create_event",
        ("POST", "/api/events/{public}/votes") => "submit_vote",
        ("PUT", "/api/events/{public}/participant/{secret}") => "edit_vote",
        ("POST", "/api/events/{public}/recover") => "recover_links",
        ("PUT", "/api/events/{public}/admin/{secret}") => "update_status",
        ("PUT", "/api/events/{public}/admin/{secret}/details") => "update_details",
        ("PUT", "/api/events/{public}/admin/{secret}/tiebreak") => "tiebreak",
        ("POST", "/api/events/{public}/admin/{secret}/messages") => "message_participants",
        ("POST", "/api/places/suggest") => "suggest_place",
        ("GET", "/api/events/{public}/calendar.ics") => "download_calendar",
        _ => return None,
    })
}

pub async fn request(State(pool): State<PgPool>, request: Request, next: Next) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".into());
    let method = request.method().clone();
    let operation = operation(&method, &route);
    let runtime = request
        .extensions()
        .get::<vercel_runtime::AppState>()
        .cloned();
    let id = Uuid::new_v4();
    let method = match method {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        _ => "OTHER",
    };
    let span = tracing::info_span!("api_request", request_id = %id, route, method,
        operation = operation.unwrap_or("read"), version = env!("CARGO_PKG_VERSION"));
    async move {
        let start = Instant::now();
        let mut response = next.run(request).await;
        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status();
        let outcome = response
            .extensions()
            .get::<&'static str>()
            .copied()
            .unwrap_or_else(|| match status {
                status if status.is_success() => "ok",
                StatusCode::PAYLOAD_TOO_LARGE => "payload_too_large",
                StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => "invalid_request",
                StatusCode::UNSUPPORTED_MEDIA_TYPE => "unsupported_media_type",
                status if status.is_server_error() => "server_error",
                _ => "request_rejected",
            });
        if status.is_server_error() && route != "/api/ready" {
            tracing::error!(
                event = "request_failed",
                status = status.as_u16(),
                outcome,
                duration_ms
            );
        } else if duration_ms >= 750 && route != "/api/ready" {
            tracing::warn!(
                event = "request_slow",
                status = status.as_u16(),
                outcome,
                duration_ms
            );
        } else if operation.is_some()
            && (status.is_success() || status == StatusCode::CONFLICT)
            && operation != Some("suggest_place")
        {
            tracing::info!(
                event = "request_completed",
                status = status.as_u16(),
                outcome,
                duration_ms
            );
        }
        if let Some(action) = operation {
            let count = async move { record(&pool, action, outcome).await };
            if let Some(runtime) = runtime {
                runtime.wait_until(count);
            } else {
                tokio::spawn(count);
            }
        }
        response.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_str(&id.to_string()).unwrap(),
        );
        response
    }
    .instrument(span)
    .await
}

async fn record(pool: &PgPool, action: &'static str, outcome: &'static str) {
    let query = sqlx::query(
        "INSERT INTO usage_metrics (day, action, outcome) VALUES ((now() AT TIME ZONE 'UTC')::date, $1, $2)
         ON CONFLICT (day, action, outcome) DO UPDATE SET total=usage_metrics.total+1",
    ).bind(action).bind(outcome).execute(pool);
    match tokio::time::timeout(Duration::from_millis(300), query).await {
        Ok(Ok(_)) => {}
        Ok(Err(error)) => tracing::debug!(
            event = "metrics_write_failed",
            action,
            reason = db_class(&error)
        ),
        Err(_) => tracing::debug!(event = "metrics_write_failed", action, reason = "timeout"),
    }
}

pub fn db_class(error: &sqlx::Error) -> &'static str {
    match error {
        sqlx::Error::PoolTimedOut => "pool_timeout",
        sqlx::Error::PoolClosed => "pool_closed",
        sqlx::Error::Io(_) => "connection_io",
        sqlx::Error::Database(_) => "database",
        sqlx::Error::RowNotFound => "row_not_found",
        _ => "sqlx",
    }
}

pub fn upstream_class(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.status().is_some() {
        "http_error"
    } else if error.is_connect() {
        "connection"
    } else if error.is_decode() {
        "invalid_response"
    } else {
        "transport"
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    VoteCalendar,
    VoteList,
    VoteBoth,
    LocationSuggested,
    LocationManual,
    ShareNative,
    ShareCopy,
    CreateDuplicateDate,
    CreateTimeRange,
    CreateTimezone,
    BrowserNetwork,
    BrowserInvalidResponse,
    BrowserClipboard,
    BrowserRuntime,
    WebRender,
}

pub async fn interaction(
    State(pool): State<PgPool>,
    Json(action): Json<Interaction>,
) -> StatusCode {
    if !take_interaction_slot() {
        return StatusCode::TOO_MANY_REQUESTS;
    }
    let (action, outcome) = match action {
        Interaction::VoteCalendar => ("vote_selection", "calendar"),
        Interaction::VoteList => ("vote_selection", "list"),
        Interaction::VoteBoth => ("vote_selection", "both"),
        Interaction::LocationSuggested => ("event_location", "suggested"),
        Interaction::LocationManual => ("event_location", "manual"),
        Interaction::ShareNative => ("share", "native"),
        Interaction::ShareCopy => ("share", "copy"),
        Interaction::CreateDuplicateDate => ("create_validation", "duplicate_date"),
        Interaction::CreateTimeRange => ("create_validation", "time_range"),
        Interaction::CreateTimezone => ("create_validation", "timezone"),
        Interaction::BrowserNetwork => ("browser_error", "network"),
        Interaction::BrowserInvalidResponse => ("browser_error", "invalid_response"),
        Interaction::BrowserClipboard => ("browser_error", "clipboard"),
        Interaction::BrowserRuntime => ("browser_error", "runtime"),
        Interaction::WebRender => ("web_error", "render"),
    };
    record(&pool, action, outcome).await;
    StatusCode::NO_CONTENT
}

fn take_interaction_slot() -> bool {
    let Ok(mut budget) = UI_BUDGET.lock() else {
        return false;
    };
    let now = Instant::now();
    let (start, count) = budget.get_or_insert((now, 0));
    if now.duration_since(*start) >= Duration::from_secs(60) {
        (*start, *count) = (now, 0);
    }
    if *count >= 120 {
        return false;
    }
    *count += 1;
    true
}

pub async fn purge_metrics(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM usage_metrics WHERE day < (now() AT TIME ZONE 'UTC')::date - 9")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn monitor_public_urls() {
    let mut targets: Vec<_> = [
        ("primary", "FRIZZI_PROBE_PRIMARY"),
        ("legacy", "FRIZZI_PROBE_LEGACY"),
    ]
    .into_iter()
    .filter_map(|(site, key)| {
        Some((
            site,
            reqwest::Url::parse(&std::env::var(key).ok()?).ok()?,
            0u32,
        ))
    })
    .collect();
    if targets.is_empty() {
        return;
    }
    let client = reqwest::Client::new();
    let mut timer = tokio::time::interval(Duration::from_secs(60));
    loop {
        timer.tick().await;
        for (site, url, failures) in &mut targets {
            let start = Instant::now();
            let result = client
                .get(url.clone())
                .timeout(Duration::from_secs(5))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status);
            let failure = match result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    match response.json::<serde_json::Value>().await {
                        Ok(body) if body["ok"] == true => None,
                        _ => Some(("invalid_readiness_response", Some(status))),
                    }
                }
                Err(error) => Some((upstream_class(&error), error.status().map(|s| s.as_u16()))),
            };
            match failure {
                None => {
                    if *failures > 0 {
                        tracing::info!(
                            event = "public_probe_recovered",
                            site = *site,
                            failures = *failures
                        );
                    }
                    *failures = 0;
                }
                Some((reason, status)) => {
                    *failures = failures.saturating_add(1);
                    let duration_ms = start.elapsed().as_millis() as u64;
                    if *failures == 1 {
                        tracing::warn!(
                            event = "public_probe_failed",
                            site = *site,
                            reason,
                            status,
                            duration_ms
                        );
                    } else if *failures == 3 {
                        tracing::error!(
                            event = "public_probe_unavailable",
                            site = *site,
                            reason,
                            status,
                            duration_ms
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_keeps_today_and_nine_previous_days() {
        let dir = std::env::temp_dir().join(format!("frizzi-logs-{}", Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        for name in [
            "2026-11-10.jsonl",
            "2026-11-11.jsonl",
            "2026-11-20.jsonl",
            "notes.txt",
        ] {
            fs::write(dir.join(name), "").unwrap();
        }
        prune_log_files(&dir, NaiveDate::from_ymd_opt(2026, 11, 20).unwrap()).unwrap();
        let exists = |name| dir.join(name).exists();
        assert!(!exists("2026-11-10.jsonl"));
        assert!(exists("2026-11-11.jsonl") && exists("2026-11-20.jsonl") && exists("notes.txt"));
        fs::remove_dir_all(dir).unwrap();
    }
}
