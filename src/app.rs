//! The application: a `vitesse` server with security, a database, a REST API
//! generator, pages and static files already plugged in.

use std::future::Future;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use memoire_orm::{Executor, Model};
use securite::Shield;
use vitesse::{BoxFuture, Middleware, Next, Request, Response};

use crate::api::Resource;
use crate::shield::{ShieldMiddleware, shield_for};
use crate::{Db, page};

/// Where the database comes from: a URL (`"sqlite://app.db"`) or an open handle.
#[derive(Debug)]
pub enum DbSource {
    /// A URL, as in `DATABASE_URL` (see [`Db`]).
    Url(String),
    /// An already opened database.
    Handle(Db),
}

impl From<&str> for DbSource {
    fn from(url: &str) -> Self {
        DbSource::Url(url.to_owned())
    }
}

impl From<String> for DbSource {
    fn from(url: String) -> Self {
        DbSource::Url(url)
    }
}

impl From<Db> for DbSource {
    fn from(db: Db) -> Self {
        DbSource::Handle(db)
    }
}

impl From<memoire_orm::MemoryDb> for DbSource {
    fn from(db: memoire_orm::MemoryDb) -> Self {
        DbSource::Handle(Db::Memory(db))
    }
}

type CreateTable = fn(&'static Db) -> BoxFuture<memoire_orm::Result<()>>;
type StartHook = Box<dyn FnOnce(&'static Db) -> BoxFuture<crate::Result<()>> + Send>;

/// A full-stack application.
///
/// ```no_run
/// use paquets::prelude::*;
///
/// #[model]
/// struct Todo {
///     id: i64,
///     title: String,
///     done: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     let mut app = App::new();
///     app.get("/", |_| async { Page::new("Home").body(h1().text("Hello 👋")) });
///     app.api::<Todo>("/api/todos");
///     app.run()
/// }
/// ```
///
/// `App` is a [`vitesse::App`] (it dereferences to it), so every routing method
/// is there: `get`, `post`, `put`, `patch`, `delete`, `ws`, `middleware`, `mount`,
/// `static_dir`, `state`, `on_error`… What `App` adds, without configuration:
///
/// - **security**: securite's shield in front of every request (see
///   [`App::security`]);
/// - **a database** from `DATABASE_URL`, in memory otherwise (see
///   [`App::database`]), available in handlers with
///   [`req.db()`](crate::RequestExt::db);
/// - **REST APIs** for your models with [`App::api`], tables created at startup;
/// - **static files** from `public/`, and the browser bundle of your islands from
///   `pkg/` (built by `paquets build`);
/// - **logs** of every request, and the port from `PORT` (Heroku), 3000 otherwise.
pub struct App {
    inner: vitesse::App,
    shield: Arc<OnceLock<Option<Shield>>>,
    security: Option<Option<Shield>>,
    logs: Arc<OnceLock<bool>>,
    database: Option<DbSource>,
    tables: Vec<CreateTable>,
    startup: Vec<StartHook>,
    port: Option<u16>,
    pwa: Option<crate::Pwa>,
    api_paths: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl Deref for App {
    type Target = vitesse::App;

    fn deref(&self) -> &vitesse::App {
        &self.inner
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut vitesse::App {
        &mut self.inner
    }
}

impl App {
    /// A new application, secure and ready to run.
    pub fn new() -> Self {
        let mut inner = vitesse::App::new();
        let logs = Arc::new(OnceLock::new());
        let shield = Arc::new(OnceLock::new());
        inner.middleware(Logs {
            enabled: logs.clone(),
            logger: Box::new(vitesse::middleware::logger()),
        });
        inner.middleware(ShieldMiddleware(shield.clone()));
        // The same limit as the shield's (2 MiB): one setting decides.
        inner.body_limit(2 * 1024 * 1024);
        App {
            inner,
            shield,
            security: None,
            logs,
            database: None,
            tables: Vec::new(),
            startup: Vec::new(),
            port: None,
            pwa: None,
            api_paths: Vec::new(),
        }
    }

    /// A REST API for a model, under `path`, its table created at startup:
    ///
    /// | Request | Effect |
    /// |---|---|
    /// | `GET /path?limit=100&offset=0` | the rows, in key order (total in `X-Total-Count`) |
    /// | `GET /path/:id` | one row, or `404` |
    /// | `POST /path` | creates a row from the JSON body: `201` with the generated key |
    /// | `PUT /path/:id` | replaces the row |
    /// | `PATCH /path/:id` | changes the fields present in the JSON body |
    /// | `DELETE /path/:id` | deletes the row: `204`, or `404` |
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// #[model]
    /// struct Todo {
    ///     id: i64,
    ///     title: String,
    ///     done: bool,
    /// }
    ///
    /// let mut app = App::new();
    /// app.api::<Todo>("/api/todos");
    /// ```
    #[track_caller]
    pub fn api<M: Resource>(&mut self, path: &str) -> &mut Self {
        crate::api::routes::<M>(&mut self.inner, path);
        self.api_paths.push(path.trim_end_matches('/').to_owned());
        self.model::<M>()
    }

    /// Creates the table of a model at startup (if it does not exist yet), for models
    /// used without [`App::api`].
    pub fn model<M: Model>(&mut self) -> &mut Self {
        fn create<M: Model>(db: &'static Db) -> BoxFuture<memoire_orm::Result<()>> {
            Box::pin(db.create_table_if_not_exists::<M>())
        }
        self.tables.push(create::<M>);
        self
    }

    /// The database: a URL or an open handle. By default it comes from the
    /// `DATABASE_URL` environment variable (Heroku sets it), and is in memory when
    /// there is none.
    ///
    /// ```no_run
    /// # let mut app = paquets::App::new();
    /// app.database("sqlite://app.db"); // a file next to the program
    /// ```
    pub fn database(&mut self, source: impl Into<DbSource>) -> &mut Self {
        self.database = Some(source.into());
        self
    }

    /// The security layer: securite's recommended shield by default (see
    /// [`shield`](crate::shield())), whose Content-Security-Policy also allows the
    /// inline code of the HTML files of `public/`. Pass another [`Shield`] to change
    /// it, or `None` to turn it off.
    ///
    /// ```
    /// use paquets::prelude::*;
    /// use paquets::securite::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
    /// ```
    pub fn security(&mut self, shield: impl Into<Option<Shield>>) -> &mut Self {
        self.security = Some(shield.into());
        self
    }

    /// Runs `hook` once the database is ready, before the first request: seed data,
    /// check the configuration…
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// #[model]
    /// struct Todo {
    ///     id: i64,
    ///     title: String,
    /// }
    ///
    /// let mut app = App::new();
    /// app.api::<Todo>("/api/todos");
    /// app.on_start(|db| async move {
    ///     if Todo::count(db).await? == 0 {
    ///         Todo { id: 0, title: "Try Paquets".into() }.insert(db).await?;
    ///     }
    ///     Ok(())
    /// });
    /// ```
    pub fn on_start<F, Fut>(&mut self, hook: F) -> &mut Self
    where
        F: FnOnce(&'static Db) -> Fut + Send + 'static,
        Fut: Future<Output = crate::Result<()>> + Send + 'static,
    {
        self.startup.push(Box::new(move |db| Box::pin(hook(db))));
        self
    }

    /// The format of every error response (`{"error": "message"}` by default), for
    /// the errors of your handlers, 404s, invalid bodies, panics… Set the status
    /// yourself, as in the example.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.on_error(|err: Error| {
    ///     res::status(err.status()).json(json!({
    ///         "error": { "status": err.status().as_u16(), "message": err.message() }
    ///     }))
    /// });
    /// ```
    pub fn on_error<F, R>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(crate::Error) -> R + Send + Sync + 'static,
        R: vitesse::IntoResponse,
    {
        self.inner
            .on_error(move |err: vitesse::Error| handler(crate::Error::from(err)));
        self
    }

    /// Makes the app installable on phones and computers (a PWA), and usable offline:
    /// serves the web app manifest at `/manifest.webmanifest` and a service worker at
    /// `/sw.js`, and every [`Page`](crate::Page) links them.
    ///
    /// The start page, the islands bundle and the icons are available offline; pages
    /// come from the network first (the cached copy when offline); the REST APIs of
    /// [`App::api`] always come from the network. Without icons in the manifest, the
    /// Paquets box is used.
    ///
    /// A site in several languages writes `{lang}` in the start URL,
    /// `.start_url("/{lang}/")`: each language gets its own manifest
    /// (`/fr/manifest.webmanifest`), linked by the pages in that language
    /// ([`Page::lang`](crate::Page::lang)), so the installed app opens in the language
    /// of the page it was installed from.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.pwa(
    ///     Pwa::new("My todos")
    ///         .short_name("Todos")
    ///         .theme_color("#6d4aff")
    ///         .background_color("#0b0b12"),
    /// );
    /// ```
    pub fn pwa(&mut self, manifest: crate::Pwa) -> &mut Self {
        self.pwa = Some(manifest);
        self
    }

    /// The port (by default `PORT`, then 3000).
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    /// Turns off the request log.
    pub fn quiet(&mut self) -> &mut Self {
        let _ = self.logs.set(false);
        self
    }

    /// Starts the server and blocks until `Ctrl+C` or `SIGTERM` (in-flight requests
    /// get up to 10 s to finish).
    ///
    /// With an in-memory or SQLite database, the server runs one thread per core,
    /// vitesse's fastest mode; with a database server (PostgreSQL…), on a tokio
    /// runtime shared by its connections.
    pub fn run(self) -> io::Result<()> {
        let result = self.start();
        if let Err(error) = &result {
            eprintln!("📦 Paquets could not start: {error}");
        }
        result
    }

    fn start(self) -> io::Result<()> {
        let kind = self.database_kind();
        if matches!(kind, "memory" | "sqlite") {
            let startup = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let (app, port) = startup.block_on(self.prepare(false))?;
            drop(startup);
            app.run(port)
        } else {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(self.serve())
        }
    }

    /// Starts the server on the current tokio runtime (`#[tokio::main]`).
    pub async fn serve(self) -> io::Result<()> {
        let (app, port) = self.prepare(false).await?;
        app.listen(port).await
    }

    /// The application in a test client, without opening a port: the database is
    /// in memory unless [`App::database`] chose one, and the log is off.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let mut app = App::new();
    /// app.get("/", |_| async { "Hello" });
    ///
    /// let client = app.test().await;
    /// let res = client.get("/").await;
    /// assert_eq!(res.status(), 200);
    /// assert_eq!(res.text(), "Hello");
    /// # }
    /// ```
    pub async fn test(mut self) -> vitesse::test::TestClient {
        self.quiet();
        let (app, _) = self
            .prepare(true)
            .await
            .unwrap_or_else(|e| panic!("paquets: the app did not start: {e}"));
        vitesse::test::TestClient::new(app)
    }

    fn database_url(&self, testing: bool) -> String {
        match &self.database {
            Some(DbSource::Url(url)) => url.clone(),
            Some(DbSource::Handle(db)) => db.kind().to_owned(),
            None if testing => "memory".to_owned(),
            None => std::env::var("DATABASE_URL").unwrap_or_default(),
        }
    }

    fn database_kind(&self) -> &'static str {
        Db::kind_of(&self.database_url(false))
    }

    /// Opens the database, creates the tables, runs the startup hooks and plugs in
    /// security, static files and islands.
    async fn prepare(mut self, testing: bool) -> io::Result<(vitesse::App, u16)> {
        let url = self.database_url(testing);
        let db = match self.database.take() {
            Some(DbSource::Handle(db)) => db,
            _ => Db::connect(&url)
                .await
                .map_err(|e| io::Error::other(format!("database: {e}")))?,
        };
        let db: &'static Db = Box::leak(Box::new(db));
        for create in &self.tables {
            create(db)
                .await
                .map_err(|e| io::Error::other(format!("database: {e}")))?;
        }
        for hook in std::mem::take(&mut self.startup) {
            hook(db)
                .await
                .map_err(|e| io::Error::other(format!("startup: {}", e.describe())))?;
        }
        self.inner.state(DbHandle(db));

        let shield = self
            .security
            .take()
            .unwrap_or_else(|| Some(shield_for(Path::new("public"))));
        let guards = shield.as_ref().map_or(0, |s| s.guard_names().len());
        let _ = self.shield.set(shield);
        let _ = self.logs.set(true);

        if Path::new("public").is_dir() {
            self.inner.middleware(vitesse::ServeDir::new("public"));
        }
        let bundle = find_bundle(Path::new("pkg"));
        if let Some((js, wasm)) = &bundle {
            self.inner.static_dir("/pkg", "pkg");
            page::set_bundle(js.clone(), wasm.clone());
        }
        if let Some(manifest) = self.pwa.take() {
            crate::pwa::install(&mut self.inner, manifest, bundle.as_ref(), &self.api_paths);
        }

        let port = self
            .port
            .or_else(|| std::env::var("PORT").ok()?.parse().ok())
            .unwrap_or(3000);
        if !testing && self.logs.get() != Some(&false) {
            let security = match guards {
                0 => "off".to_owned(),
                n => format!("on ({n} guards)"),
            };
            let islands = bundle.map_or("none".to_owned(), |(js, _)| js);
            println!(
                "📦 Paquets · http://localhost:{port}\n   database: {} · security: {security} · islands: {islands}",
                db.kind()
            );
        }
        Ok((self.inner, port))
    }
}

/// `pkg/<name>.js` and `pkg/<name>_bg.wasm`, as written by `wasm-bindgen --target web`.
fn find_bundle(dir: &Path) -> Option<(String, String)> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    names.sort();
    names.iter().find_map(|name| {
        let stem = name.strip_suffix("_bg.wasm")?;
        dir.join(format!("{stem}.js"))
            .is_file()
            .then(|| (format!("/pkg/{stem}.js"), format!("/pkg/{name}")))
    })
}

/// The database, in vitesse's application state.
struct DbHandle(&'static Db);

/// Gives handlers the database.
pub trait RequestExt {
    /// The application's database.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// #[model]
    /// struct Todo {
    ///     id: i64,
    ///     title: String,
    /// }
    ///
    /// async fn count(req: Request) -> Result<String> {
    ///     Ok(format!("{} todos", Todo::count(req.db()).await?))
    /// }
    /// ```
    fn db(&self) -> &'static Db;
}

impl RequestExt for Request {
    fn db(&self) -> &'static Db {
        match self.try_state::<DbHandle>() {
            Some(handle) => handle.0,
            None => panic!("req.db() needs an app created with paquets::App"),
        }
    }
}

/// The request log, which `App::quiet` can turn off after it was installed.
struct Logs {
    enabled: Arc<OnceLock<bool>>,
    logger: Box<dyn Middleware>,
}

impl Middleware for Logs {
    fn handle(&'static self, req: Request, next: Next) -> BoxFuture<Response> {
        if self.enabled.get() == Some(&true) {
            self.logger.handle(req, next)
        } else {
            next.run(req)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_browser_bundle() {
        let dir = std::env::temp_dir().join(format!("paquets-pkg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(find_bundle(&dir), None);
        std::fs::write(dir.join("app_bg.wasm"), b"").unwrap();
        assert_eq!(find_bundle(&dir), None, "no JavaScript module yet");
        std::fs::write(dir.join("app.js"), b"").unwrap();
        assert_eq!(
            find_bundle(&dir),
            Some(("/pkg/app.js".into(), "/pkg/app_bg.wasm".into()))
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
