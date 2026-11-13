# Nuage DB

Nuage DB is the database of Paquets projects: `paquets new` uses it since version 0.3.3. It keeps your data in a folder of the project while you develop, and on a Nuage DB server in production, with the same code. This page covers how it works in Paquets, running and deploying a server, connecting an app, branches for previews and tests, the console and the dashboard.

## Why Nuage DB

[Nuage DB](https://github.com/maxlestage/Nuage) is the database of Memoire, the ORM of Paquets, by the same author:

- **Written entirely in Rust**, like the rest of your app: one language from the browser to the disk.
- **It looks after itself**: a write-ahead log synced to disk before each commit, crash recovery at startup, automatic checkpoints and compaction, checksums on the log and on the pages, and automatic backups to a folder or to S3-compatible storage, restored on their own when the server starts with an empty disk.
- **On disk, not limited by memory**: tables and indexes live in a B-tree file with a bounded page cache; lookups by primary key or index read only the rows they need.
- **Instant branches**: a copy-on-write copy of the whole database, made in an instant whatever its size, for a preview or a test.
- **One server, always on**, which serves the database, the `/admin` console and the live `/app` dashboard on one port.
- **From WebAssembly too**: the client speaks HTTP(S), natively and through the browser's `fetch`.

Your models and queries don't change: Nuage DB runs Memoire's statements with Memoire's own engine, so `req.db()`, the query builder, relations, transactions and migrations work as with every other database.

## Development and production

A project made by `paquets new` chooses Nuage DB in `src/main.rs`:

```rust
fn app() -> App {
    let mut app = App::new();
    // Nuage DB: kept in data/ in development, on a Nuage DB server in production
    // (NUAGE_URL and NUAGE_TOKEN), without changing this code.
    app.nuage("data");
    // ...
    app
}
```

The environment decides where the data goes:

| Environment | Database | Startup line |
|---|---|---|
| nothing (development) | the local mode of Nuage DB, in the `data/` folder | `database: nuage, local (data)` |
| `NUAGE_URL=https://my-nuage.herokuapp.com`, `NUAGE_TOKEN=…` | a Nuage DB server | `database: nuage (https://my-nuage.herokuapp.com)` |
| `NUAGE_URL=other-folder` | the local mode, in that folder | `database: nuage, local (other-folder)` |
| `DATABASE_URL=…` and no `NUAGE_URL` | that database: PostgreSQL, SQLite… (see [Database](database.md#choosing-the-database)) | `database: postgres`… |

`NUAGE_URL` wins over `DATABASE_URL`, which wins over the folder of `app.nuage(...)`. A database chosen in the code with `app.database(...)` wins over all of them, and tests made with `app.test()` always get a fresh in-memory database.

**The local mode** keeps the database in `data/app.memoire`, with the engine Nuage DB runs statements with (Memoire's): a query gives the same rows on your laptop and on the server. The folder is created if it is missing; `paquets new` lists it in `.gitignore` and `.dockerignore`. The whole database is held in memory while the app runs, which suits development; branches, the console and the dashboard come with a server, which you can also run on your machine (next section).

## Run a server on your machine

The server is the `nuage` program of the [Nuage repository](https://github.com/maxlestage/Nuage):

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
NUAGE_TOKEN=dev-token NUAGE_ADMIN_PASSWORD=dev-password cargo run -p nuage --release
# nuage: data is stored in /home/me/Nuage/nuage-data on this server's disk
# nuage: listening on 0.0.0.0:8080
```

Then, in your project:

```bash
NUAGE_URL=http://localhost:8080 NUAGE_TOKEN=dev-token paquets dev
#   database: nuage (http://localhost:8080) · security: on (10 guards) · islands: /pkg/my_app.js
```

Open http://localhost:8080/admin for the console and http://localhost:8080/app for the dashboard (sign in with `NUAGE_ADMIN_PASSWORD`).

## Deploy a Nuage DB server

The server is one binary configured by environment variables. Deploy it once, next to your apps:

| Variable | Meaning |
|---|---|
| `PORT` | the port (set by Heroku and most hosts; `8080` otherwise) |
| `NUAGE_TOKEN` | the token of the database API: without it, the API refuses every request |
| `NUAGE_ADMIN_PASSWORD` | the password of `/admin` and `/app`: without it, the console is off |
| `NUAGE_DATA_DIR` | the data folder (`nuage-data` by default) |
| `NUAGE_BACKUP_DIR` | backups to a folder |
| `NUAGE_BACKUP_S3_BUCKET`, `_REGION`, `_ENDPOINT`, `_ACCESS_KEY_ID`, `_SECRET_ACCESS_KEY` | backups to S3-compatible storage |
| `NUAGE_CACHE_MB` | the page cache, in MiB (64 by default) |

**On Heroku**, the root of the Nuage repository deploys as is (`Procfile`, `RustConfig`, `heroku-build.sh`, and `app.json` for a **Deploy to Heroku** button):

1. Create the app with the Rust buildpack: `heroku create my-nuage --buildpack emk/rust`, or from the dashboard, the button of the Nuage repository.
2. Create a bucket (AWS S3, Cloudflare R2, Scaleway…) and an access key limited to it.
3. Set the config vars:

   ```bash
   heroku config:set NUAGE_TOKEN=$(openssl rand -hex 32) \
     NUAGE_ADMIN_PASSWORD='a long passphrase' \
     NUAGE_BACKUP_S3_BUCKET=my-bucket NUAGE_BACKUP_S3_REGION=eu-west-3 \
     NUAGE_BACKUP_S3_ACCESS_KEY_ID=... NUAGE_BACKUP_S3_SECRET_ACCESS_KEY=...
   ```

4. Deploy, then open `https://my-nuage.herokuapp.com/admin`.

> [!WARNING]
> A dyno's disk is erased at every restart (at least once a day). The data folder is only a working copy: **the data lives in the backups**, and each start restores it from them. Configure S3 backups on Heroku, and run **exactly one** Basic dyno (it never sleeps): two dynos would be two databases writing to the same backups. Commits since the last shipped log segment (30 seconds by default) are lost only if the dyno stops without the `SIGTERM` Heroku sends first.

**With Docker**, on any host with a persistent volume:

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
docker build -t nuage .
docker run -p 8080:8080 -v nuage-data:/data -e NUAGE_TOKEN=... -e NUAGE_ADMIN_PASSWORD=... nuage
```

Serve the server over HTTPS (Heroku and most hosts do it for you): the token travels in a header, and the console's cookie is `Secure`.

## Connect your app

Give your app the address and the token of the server. On Heroku: your app → **Settings** → **Reveal Config Vars**:

| Config var | Value |
|---|---|
| `NUAGE_URL` | `https://my-nuage.herokuapp.com` |
| `NUAGE_TOKEN` | the server's `NUAGE_TOKEN` |

The app restarts, connects, checks the token and creates the tables of your models. If the server can't be reached or refuses the token, the app doesn't start and says why. The token is never part of the URL, and the startup line never shows it.

Paquets keeps the HTTP connections to the server alive between requests, and uses TLS for `https://` addresses. With a server, your app runs on a multi-threaded tokio runtime, as with PostgreSQL.

To open the database yourself, `Db::nuage` opens either kind (a server's address, or a folder), and `Db::connect` accepts a server's address too, with the token of `NUAGE_TOKEN`. Do it from an async `main` that starts the app with `app.serve().await` (see [Configuration](configuration.md#async-initialisation)), so that the connections live on the app's runtime:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("NUAGE_TOKEN").ok();
    let db = Db::nuage("https://my-nuage.herokuapp.com", token.as_deref())
        .await
        .map_err(std::io::Error::other)?;
    let mut app = app();
    app.database(db);
    app.serve().await
}
```

## Branches for previews and tests

A branch is a copy of the database made in an instant, whatever its size: it shares every page of its parent and only stores its own changes. Later changes of the main database don't reach it.

**Previews.** Set `NUAGE_BRANCH` next to `NUAGE_URL` on a preview app (a Heroku review app, a staging app): at startup, Paquets creates the branch from the main database if it doesn't exist yet, then runs every query on it. The preview works on real data and can't touch production's.

```text
NUAGE_URL=https://my-nuage.herokuapp.com
NUAGE_TOKEN=…
NUAGE_BRANCH=preview-42
```

Reset or delete a branch from the console (**Branches**), or in code, through the client of `Db::Nuage`:

```rust
if let Db::Nuage(nuage) = req.db() {
    nuage.reset_branch("preview-42").await?; // forget its changes
    nuage.delete_branch("preview-42").await?;
}
```

**Tests.** `app.test()` uses an in-memory database: fast, and the right choice for most tests. To test against a copy of real data, give a test its own branch:

```rust
use paquets::orm::NuageDb;

#[tokio::test]
async fn against_a_copy_of_production() -> paquets::orm::Result<()> {
    let (Ok(url), Ok(token)) = (std::env::var("NUAGE_URL"), std::env::var("NUAGE_TOKEN")) else {
        return Ok(()); // no server: skipped
    };
    let server = NuageDb::connect(&url, &token).await?;
    let name = format!("test-{}", std::process::id());
    server.create_branch(&name, None).await?;

    let mut app = app();
    app.database(Db::from(server.on_branch(&name)));
    let client = app.test().await;
    assert_eq!(client.get("/").await.status(), 200);

    server.delete_branch(&name).await?;
    Ok(())
}
```

## The console and the dashboard

The server you deployed has two pages, protected by `NUAGE_ADMIN_PASSWORD`:

- **`/admin`**, the console: an overview (size, tables, rows, branches, uptime, connections, cache, last checkpoint, backups), each table and its rows page by page, a query console (a small subset of SQL, with a confirmation before changes), the branches, and storage, with buttons for a checkpoint, a backup, an integrity check and a compaction. It is rendered by the server and works without JavaScript, on a phone too.
- **`/app`**, the live dashboard: requests per second, latency, connections, storage, cache, write-ahead log, backups, databases and branches, and a live event log, refreshed every second. It installs as an app on a phone.

`GET /api/health` answers `{"status":"ok"}` for uptime monitors. With `NUAGE_PUBLIC_STATUS=true`, `/app` shows a public subset without signing in.

## From the browser

The client also runs in WebAssembly, in an island, through the browser's `fetch`. Add the `nuage` feature to the browser side of `Cargo.toml`:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client", "nuage"] }
```

`paquets::orm::NuageDb` is then available in `src/lib.rs`, with the same models and queries.

> [!WARNING]
> The token opens the whole database. Send it to the browser only in an app for yourself or your team (an internal dashboard behind a login), never in a public page: public pages read and write through your server's API, which keeps the token on the server.

## Moving an existing project

Projects made before 0.3.3 don't change: without `NUAGE_URL` and without `app.nuage(...)`, the database still comes from `DATABASE_URL`, and is in memory without it. To move to Nuage DB:

1. In `src/main.rs`, after `App::new()`, add `app.nuage("data");`.
2. Add `/data` to `.gitignore` and `data` to `.dockerignore`.
3. In the `Dockerfile`, before `USER app`, add `RUN mkdir data && chown app data`, so that the app can write the folder when `NUAGE_URL` is not set.
4. Deploy a Nuage DB server, set `NUAGE_URL` and `NUAGE_TOKEN` on the app.

To copy existing data, run a one-off program, or a [seed](database.md#data-at-startup), that reads from the old database and writes to the new one:

```rust
let old = Db::connect(&std::env::var("OLD_DATABASE_URL").unwrap()).await?;
let token = std::env::var("NUAGE_TOKEN").ok();
let new = Db::nuage(&std::env::var("NUAGE_URL").unwrap(), token.as_deref()).await?;
new.push::<Todo>().await?;
let todos = Todo::query().all(&old).await?;
Todo::insert_many(&new, &todos).await?;
```

The `nuage` feature is on by default. A project that doesn't use it can leave it out with `default-features = false` (see [Installation](installation.md#cargo-features)).

## Limits

- **No SQL.** Nuage DB runs Memoire's statements: `db.raw(...)` and the `.sql(...)` steps of migrations answer "unsupported", as with the in-memory database. For SQL written by hand, use PostgreSQL through `DATABASE_URL`.
- **One writer at a time.** Writes are applied one after another, reads run in parallel; one server per database.
- **The local mode is for development**: it holds the whole database in memory while the app runs, and has no branches, console or dashboard (run a server for them).
- **Joins, `GROUP BY` and aggregates** other than `COUNT(*)` read the rows they combine into memory, after the filters of each table.

The whole guide of Nuage DB, with its backups, transactions and every setting, is in [Memoire's documentation](https://memoire-7f4525f1fdaf.herokuapp.com/en/docs/nuage/) and the [Nuage repository](https://github.com/maxlestage/Nuage).

## Next steps

- [Database](database.md): models, queries, transactions and the other databases.
- [Configuration](configuration.md): every environment variable.
- [Deploy to Heroku](deploy-heroku-mobile.md): your app, from your phone.
