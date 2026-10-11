# Configuration

A Paquets app works without configuration. When you need to change something, there are three places: environment variables (set by your host, or by you), methods on `App`, and Cargo features. This page lists them all, shows how to read your own settings and share state between requests, and explains how the server runs.

## Environment variables

| Variable | Used for | Default |
|---|---|---|
| `PORT` | the port the server listens on | `3000` |
| `DATABASE_URL` | the database (see [Database](database.md#choosing-the-database)) | in memory |
| `DYNO` | set by Heroku; the default shield then reads the client IP from `X-Forwarded-For` | — |

Set them in your shell for one run:

```sh
PORT=8080 DATABASE_URL=sqlite://app.db paquets dev
```

On Heroku, `PORT` is set for you, and `DATABASE_URL` too when you add the Heroku Postgres add-on. Your own variables go in **Settings → Config Vars** (see [Deploy to Heroku](deploy-heroku-mobile.md#settings-config-vars)). With Docker, pass them with `-e NAME=value` (see [Docker](deploy-docker.md)).

Paquets doesn't read `.env` files. If you like them, the [`dotenvy`](https://crates.io/crates/dotenvy) crate loads one with a single call at the start of `main`.

## App settings

`App::new()` gives you a ready app; these methods change it. Call them before `app.run()`, in any order.

| Method | Effect | Default |
|---|---|---|
| `app.port(8080)` | the port, instead of `PORT` | `PORT`, then `3000` |
| `app.quiet()` | turns off the request log | one line per request |
| `app.pwa(manifest)` | installable app that works offline (see [Pages](pages.md#installable-app-pwa)) | not installable |
| `app.database(url_or_db)` | the database, instead of `DATABASE_URL` | `DATABASE_URL`, then memory |
| `app.security(shield)` | another security shield, or `None` to turn it off | `paquets::shield()` |
| `app.on_start(hook)` | code run once the database is ready | — |
| `app.api::<M>(path)` | a REST API and a table for a model | — |
| `app.model::<M>()` | a table for a model | — |

`App` also gives access to every method of vitesse's app:

| Method | Effect | Default |
|---|---|---|
| `app.state(value)` | shared state, read with `req.state::<T>()` (see below) | — |
| `app.body_limit(bytes)` | the largest body a handler can read | 2 MiB, like the shield's body limit |
| `app.middleware(m)` | a global middleware (see [Routing](routing.md#middleware)) | — |
| `app.mount(prefix, router)` | a router under a prefix | — |
| `app.static_dir(prefix, dir)` | serves a folder under a prefix | `public/` at `/` |
| `app.fallback(handler)` | the answer when no route matches | `404` JSON |
| `app.on_error(f)` | the format of every error response (see [Errors](errors.md#changing-the-format-of-every-error)) | `{"error": "..."}` |
| `app.workers(n)` | the number of server threads (see below) | one per CPU core |
| `app.thread_per_core(false)` | tokio's work-stealing runtime instead of one event loop per core (see below) | `true` on Linux |

The methods that return `&mut App` can be chained: `app.port(8080).quiet();`. Only `run()`, `serve()` and `test()` consume the app, so they come last.

## Your own settings

Read your settings once, at startup, into a struct, and keep that struct in the app's state:

```rust
use paquets::prelude::*;

/// The settings of the app, read once at startup.
struct Settings {
    site_name: String,
    admin_email: String,
}

impl Settings {
    fn from_env() -> Settings {
        Settings {
            site_name: std::env::var("SITE_NAME").unwrap_or_else(|_| "My site".into()),
            admin_email: std::env::var("ADMIN_EMAIL").unwrap_or_default(),
        }
    }
}

fn app() -> App {
    let mut app = App::new();
    app.state(Settings::from_env());
    app.get("/", |req: Request| async move {
        let settings = req.state::<Settings>();
        Page::new(settings.site_name.clone())
            .body(h1().text(format!("Welcome to {}", settings.site_name)))
    });
    app
}
```

Keep secrets (API keys, passwords) in environment variables, never in the code or in Git.

## Shared state

`app.state(value)` registers a value that every request can read with `req.state::<T>()`: settings, a cache, a counter, a channel (see the chat room in [WebSocket](websocket.md#a-chat-room)).

- State is found **by type**: one value per type. Give each piece of state its own type (a struct), rather than registering a bare `String`.
- The type must be `Send + Sync + 'static`, because all the server's threads share it.
- `req.state::<T>()` returns a `&T` that lives as long as the app: no cloning, no locking.
- Asking for a type that was never registered panics, which gives the client a `500`. `req.try_state::<T>()` returns an `Option` instead.

To change state from a request, use a type made for sharing between threads: an atomic for a counter, a `Mutex` or `RwLock` for a collection.

```rust
use std::sync::atomic::{AtomicU64, Ordering};

app.state(AtomicU64::new(0));
app.get("/visits", |req: Request| async move {
    let n = req.state::<AtomicU64>().fetch_add(1, Ordering::Relaxed) + 1;
    format!("visit #{n}")
});
```

State lives in memory: it is lost when the server restarts, and each server has its own copy. Data that must last belongs in the [database](database.md). More patterns in [vitesse's state guide](https://maxlestage.github.io/Vitesse/en/docs/state/).

## How the server runs

`app.run()` prepares the app (database, tables, `on_start` hooks, shield, static files, islands), prints the startup line, and serves until `Ctrl+C` or `SIGTERM`:

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: none
```

- It listens on **every network interface** (`0.0.0.0`), which is what containers and hosts such as Heroku need.
- With the in-memory database or SQLite, it runs **one thread per CPU core**, each with its own event loop: vitesse's fastest mode. `app.workers(n)` and `app.thread_per_core(false)` apply in this mode.
- With a database server (PostgreSQL, MySQL, Redis, MongoDB), it runs on a tokio runtime with one thread per core, shared by the database connections. `workers` and `thread_per_core` don't apply.
- On `Ctrl+C` or `SIGTERM`, it stops accepting connections and gives the requests in progress up to 10 seconds to finish (graceful shutdown). Heroku and Docker send `SIGTERM` before stopping a container.

## Async initialisation

`app.run()` creates its own runtime, so `main` doesn't need to be `async`. When you need to `.await` something before the app starts (opening a database pool with custom settings, fetching a remote configuration), make `main` async with tokio and start the app with `app.serve().await`:

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use paquets::orm::PgPool;
use paquets::prelude::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut app = app();
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if url.starts_with("postgres") {
            // At most 10 connections, for a small hosted plan.
            let pool = PgPool::connect_with(&url, 10).await.map_err(std::io::Error::other)?;
            app.database(Db::from(pool));
        }
    }
    app.serve().await
}
```

`serve()` does everything `run()` does, on the current runtime (`workers` and `thread_per_core` don't apply).

## Cargo features

Databases, extra column types and HTTP/3 are Cargo features of the `paquets` crate: see [Installation](installation.md#cargo-features).

## Production checklist

- [ ] Build in release mode: `paquets build` (or `cargo build --release`). The `Dockerfile` of a generated project does it.
- [ ] Use a database that persists: `DATABASE_URL` pointing to PostgreSQL on Heroku (not SQLite, not memory).
- [ ] Listen on `PORT` (automatic) and launch the program from the folder that contains `public/` and `pkg/`.
- [ ] Behind a proxy other than Heroku's, tell the shield where the client IP is (see [Security](security.md#the-client-ip-behind-a-proxy)).
- [ ] Keep secrets in environment variables.
- [ ] Serve the site over HTTPS: Heroku does it on `*.herokuapp.com` and with its certificate management on custom domains; elsewhere, use a proxy or CDN.
- [ ] Protect what changes data: a middleware, `Auth`, or sessions (see [Security](security.md#authentication)).
- [ ] Run the tests on every push: the CI workflow does it (see [Automation](automation.md)).

## Next steps

- [Performance](performance.md): what makes Paquets fast and how to keep it fast.
- [Deploy to Heroku](deploy-heroku-mobile.md): config vars and logs on Heroku.
- [Security](security.md): the shield's settings.
