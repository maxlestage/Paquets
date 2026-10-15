//! Operations: health checks, Prometheus metrics, live reload during development,
//! and the commands run by the `paquets` tool (`routes`, `db:migrate`, `db:seed`).

use std::net::IpAddr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use memoire_orm::{Database, Executor, Transaction};
use securite::headers::ContentSecurityPolicy;
use vitesse::openapi::Operation;
use vitesse::{IntoResponse, Next, Request, Response, StatusCode, json};

use crate::{Db, RequestExt};

/// The environment variable that asks the app for a command instead of serving:
/// `routes`, `migrate` or `seed` (set by `paquets routes`, `paquets db:migrate`…).
pub(crate) const COMMAND_VAR: &str = "PAQUETS_COMMAND";

/// Set by `paquets dev --watch`: the pages reload when the server restarts.
pub(crate) const LIVE_RELOAD_VAR: &str = "PAQUETS_LIVE_RELOAD";

/// Where the pages listen for a restart of the server.
pub(crate) const LIVE_RELOAD_PATH: &str = "/__paquets/live-reload";

/// When this process started.
fn started() -> Instant {
    static STARTED: OnceLock<Instant> = OnceLock::new();
    *STARTED.get_or_init(Instant::now)
}

// ----- Health ----------------------------------------------------------------------

/// Whether the database answers: `SELECT 1` on SQL databases, a transaction on the
/// others.
pub(crate) async fn database_ok(db: &Db) -> bool {
    if db.dialect().is_some() {
        db.raw::<i64>("SELECT 1", &[]).await.is_ok()
    } else {
        match db.begin().await {
            Ok(tx) => tx.rollback().await.is_ok(),
            Err(_) => false,
        }
    }
}

/// `GET /health`: `200 {"status": "ok", …}`, or `503` when the database is down.
pub(crate) fn health(app: &mut vitesse::App, path: &str) {
    started();
    app.get(path, |req: Request| async move {
        let database = database_ok(req.db()).await;
        let status = if database {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };
        vitesse::res::status(status)
            .header("cache-control", "no-store")
            .json(json!({
                "status": if database { "ok" } else { "error" },
                "database": { "kind": req.db().kind(), "ok": database },
                "uptime_seconds": started().elapsed().as_secs(),
                "paquets": crate::VERSION,
            }))
    })
    .doc(
        Operation::new()
            .summary("Health check")
            .tag("operations")
            .response(200, "The app and its database work")
            .response(503, "The database does not answer"),
    );
}

// ----- Metrics ---------------------------------------------------------------------

struct Counters {
    in_flight: AtomicU64,
    by_class: [AtomicU64; 5],
    duration_micros: AtomicU64,
    /// Requests faster than 5 ms, 25 ms, 100 ms, 500 ms, 2.5 s.
    buckets: [AtomicU64; 5],
}

const BUCKETS: [(f64, u64); 5] = [
    (0.005, 5_000),
    (0.025, 25_000),
    (0.1, 100_000),
    (0.5, 500_000),
    (2.5, 2_500_000),
];

fn counters() -> &'static Counters {
    static COUNTERS: OnceLock<Counters> = OnceLock::new();
    COUNTERS.get_or_init(|| Counters {
        in_flight: AtomicU64::new(0),
        by_class: Default::default(),
        duration_micros: AtomicU64::new(0),
        buckets: Default::default(),
    })
}

/// Counts every request, then serves them in Prometheus' text format at `path`, to
/// local requests and to those with `Authorization: Bearer $METRICS_TOKEN`.
pub(crate) fn metrics(app: &mut vitesse::App, path: &str) {
    started();
    let path_owned = path.to_owned();
    app.middleware(move |req: Request, next: Next| {
        let counted = req.path() != path_owned;
        async move {
            if !counted {
                return next.run(req).await;
            }
            let c = counters();
            c.in_flight.fetch_add(1, Ordering::Relaxed);
            let start = Instant::now();
            let res = next.run(req).await;
            let micros = start.elapsed().as_micros() as u64;
            c.in_flight.fetch_sub(1, Ordering::Relaxed);
            let class = (res.status_code().as_u16() / 100).clamp(1, 5) as usize - 1;
            c.by_class[class].fetch_add(1, Ordering::Relaxed);
            c.duration_micros.fetch_add(micros, Ordering::Relaxed);
            for (bucket, (_, limit)) in c.buckets.iter().zip(BUCKETS) {
                if micros <= limit {
                    bucket.fetch_add(1, Ordering::Relaxed);
                }
            }
            res
        }
    });
    app.get(path, |req: Request| async move {
        if !metrics_allowed(&req) {
            return crate::Error::forbidden(
                "metrics: local requests, or Authorization: Bearer $METRICS_TOKEN",
            )
            .into_response();
        }
        Response::new()
            .content_type("text/plain; version=0.0.4")
            .header("cache-control", "no-store")
            .send(prometheus())
    })
    .doc(
        Operation::new()
            .summary("Prometheus metrics")
            .tag("operations")
            .response(200, "Requests, durations, uptime"),
    );
}

fn metrics_allowed(req: &Request) -> bool {
    if let Ok(token) = std::env::var("METRICS_TOKEN")
        && !token.is_empty()
    {
        let given = req
            .header("authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or_default();
        return constant_time_eq(given.as_bytes(), token.as_bytes());
    }
    is_local(req)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// A request from this machine, not relayed by a proxy.
pub(crate) fn is_local(req: &Request) -> bool {
    let forwarded = ["x-forwarded-for", "forwarded", "x-real-ip"]
        .iter()
        .any(|h| req.header(h).is_some());
    !forwarded
        && req.remote_addr().is_some_and(|addr| match addr.ip() {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => {
                ip.is_loopback() || ip.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
            }
        })
}

/// The metrics, in Prometheus' text format.
pub(crate) fn prometheus() -> String {
    let c = counters();
    let mut out = String::new();
    out.push_str("# HELP paquets_requests_total Requests answered, by status class.\n");
    out.push_str("# TYPE paquets_requests_total counter\n");
    let mut total = 0;
    for (i, count) in c.by_class.iter().enumerate() {
        let count = count.load(Ordering::Relaxed);
        total += count;
        out.push_str(&format!(
            "paquets_requests_total{{status=\"{}xx\"}} {count}\n",
            i + 1
        ));
    }
    out.push_str("# HELP paquets_requests_in_flight Requests being answered.\n");
    out.push_str("# TYPE paquets_requests_in_flight gauge\n");
    out.push_str(&format!(
        "paquets_requests_in_flight {}\n",
        c.in_flight.load(Ordering::Relaxed)
    ));
    out.push_str("# HELP paquets_request_duration_seconds Time to answer a request.\n");
    out.push_str("# TYPE paquets_request_duration_seconds histogram\n");
    for (bucket, (le, _)) in c.buckets.iter().zip(BUCKETS) {
        out.push_str(&format!(
            "paquets_request_duration_seconds_bucket{{le=\"{le}\"}} {}\n",
            bucket.load(Ordering::Relaxed)
        ));
    }
    out.push_str(&format!(
        "paquets_request_duration_seconds_bucket{{le=\"+Inf\"}} {total}\n"
    ));
    out.push_str(&format!(
        "paquets_request_duration_seconds_sum {}\n",
        c.duration_micros.load(Ordering::Relaxed) as f64 / 1e6
    ));
    out.push_str(&format!("paquets_request_duration_seconds_count {total}\n"));
    out.push_str("# HELP paquets_uptime_seconds Time since the server started.\n");
    out.push_str("# TYPE paquets_uptime_seconds gauge\n");
    out.push_str(&format!(
        "paquets_uptime_seconds {}\n",
        started().elapsed().as_secs()
    ));
    out
}

// ----- Live reload -----------------------------------------------------------------

/// Whether `paquets dev --watch` runs this server.
pub(crate) fn live_reload() -> bool {
    std::env::var_os(LIVE_RELOAD_VAR).is_some_and(|v| v != "0" && !v.is_empty())
}

/// The identity of this run of the server: a new one after each restart.
fn boot_id() -> &'static str {
    static BOOT: OnceLock<String> = OnceLock::new();
    BOOT.get_or_init(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        format!("{}-{nanos}", std::process::id())
    })
}

/// The script of the pages: reloads them when the server comes back with a new
/// identity; and its CSP hash.
pub(crate) fn live_reload_script() -> &'static (String, String) {
    static SCRIPT: OnceLock<(String, String)> = OnceLock::new();
    SCRIPT.get_or_init(|| {
        let script = format!(
            "(()=>{{let boot;const s=new EventSource(\"{LIVE_RELOAD_PATH}\");\
s.onmessage=e=>{{if(boot&&boot!==e.data)location.reload();boot=e.data;}};}})();"
        );
        let hash =
            ContentSecurityPolicy::hashes_for_inline_scripts(&format!("<script>{script}</script>"))
                .pop()
                .unwrap_or_default();
        (script, hash)
    })
}

/// The event stream the pages listen to: the identity of the server, then nothing
/// until it stops.
pub(crate) fn live_reload_route(app: &mut vitesse::App) {
    app.get(LIVE_RELOAD_PATH, |_| async {
        let (sender, stream) = vitesse::sse::channel(1);
        let _ = sender.try_send(
            vitesse::sse::Event::new()
                .data(boot_id())
                .retry(Duration::from_millis(500)),
        );
        tokio::spawn(async move { sender.closed().await });
        stream
    })
    .doc(Operation::new().hidden());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_are_in_prometheus_format() {
        let text = prometheus();
        assert!(text.contains("# TYPE paquets_requests_total counter"));
        assert!(text.contains("paquets_requests_total{status=\"2xx\"}"));
        assert!(text.contains("paquets_request_duration_seconds_bucket{le=\"+Inf\"}"));
        assert!(text.contains("paquets_uptime_seconds"));
    }

    #[test]
    fn tokens_are_compared_whole() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secre"));
        assert!(!constant_time_eq(b"secret", b"secreT"));
    }

    #[test]
    fn the_live_reload_script_has_its_hash() {
        let (script, hash) = live_reload_script();
        assert!(script.contains(LIVE_RELOAD_PATH));
        assert!(hash.starts_with("'sha256-"));
    }
}
