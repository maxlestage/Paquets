//! The database: one handle for every backend of `memoire-orm`, chosen by a URL.

use std::fmt;

use memoire_orm::{
    Database, Dialect, ExecResult, Executor, FromRow, MemoryDb, MemoryTransaction, Query, Result,
    TableSchema, Transaction, Value,
};

#[cfg(feature = "mongodb")]
use memoire_orm::{MongoDb, MongoTransaction};
#[cfg(feature = "mysql")]
use memoire_orm::{MySqlPool, MySqlTransaction};
#[cfg(feature = "nuage")]
use memoire_orm::{NuageDb, NuageTransaction};
#[cfg(feature = "postgres")]
use memoire_orm::{PgPool, PgTransaction};
#[cfg(feature = "redis")]
use memoire_orm::{RedisDb, RedisTransaction};
#[cfg(feature = "sqlite")]
use memoire_orm::{SqliteDb, SqliteTransaction};

/// The application's database, whatever the backend.
///
/// Handlers get it with [`req.db()`](crate::RequestExt::db). Every model method
/// and query of `memoire-orm` accepts it:
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
/// async fn open_todos(req: Request) -> Result<Json<Vec<Todo>>> {
///     let todos = Todo::query()
///         .filter(Todo::DONE.eq(false))
///         .order_by(Todo::TITLE.asc())
///         .all(req.db())
///         .await?;
///     Ok(Json(todos))
/// }
/// ```
///
/// The backend comes from the `NUAGE_URL` or `DATABASE_URL` environment variable
/// (or [`App::database`](crate::App::database), [`App::nuage`](crate::App::nuage)):
///
/// | URL | Backend | Cargo feature |
/// |---|---|---|
/// | `https://…`, `http://…` | a Nuage DB server, with the token of `NUAGE_TOKEN` | `nuage` (default) |
/// | `nuage:data` | Nuage DB's local mode: a database kept in the folder `data` | always |
/// | *(none)*, `memory` | in memory, emptied when the server stops | always |
/// | `memory://data/app.memoire` | in memory, stored in a file and read back at startup | always |
/// | `sqlite://app.db`, `app.db`, `sqlite::memory:` | SQLite file | `sqlite` (default) |
/// | `postgres://…` | PostgreSQL, TLS when the server offers it | `postgres` (default) |
/// | `mysql://…`, `mariadb://…` | MySQL / MariaDB | `mysql` |
/// | `redis://…`, `rediss://…` | Redis | `redis` |
/// | `mongodb://…`, `mongodb+srv://…` | MongoDB | `mongodb` |
#[derive(Clone)]
#[non_exhaustive]
pub enum Db {
    /// In memory ([`MemoryDb`]).
    Memory(MemoryDb),
    /// SQLite.
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteDb),
    /// PostgreSQL.
    #[cfg(feature = "postgres")]
    Postgres(PgPool),
    /// MySQL / MariaDB.
    #[cfg(feature = "mysql")]
    MySql(MySqlPool),
    /// Redis.
    #[cfg(feature = "redis")]
    Redis(RedisDb),
    /// MongoDB.
    #[cfg(feature = "mongodb")]
    Mongo(MongoDb),
    /// A Nuage DB server ([`NuageDb`]): branches with
    /// [`create_branch`](NuageDb::create_branch) and [`on_branch`](NuageDb::on_branch).
    #[cfg(feature = "nuage")]
    Nuage(NuageDb),
}

/// Runs the same expression on whichever backend `$db` holds.
macro_rules! each {
    ($db:expr, $enum:ident, $d:ident => $e:expr) => {
        match $db {
            $enum::Memory($d) => $e,
            #[cfg(feature = "sqlite")]
            $enum::Sqlite($d) => $e,
            #[cfg(feature = "postgres")]
            $enum::Postgres($d) => $e,
            #[cfg(feature = "mysql")]
            $enum::MySql($d) => $e,
            #[cfg(feature = "redis")]
            $enum::Redis($d) => $e,
            #[cfg(feature = "mongodb")]
            $enum::Mongo($d) => $e,
            #[cfg(feature = "nuage")]
            $enum::Nuage($d) => $e,
        }
    };
}

impl Db {
    /// An empty in-memory database.
    pub fn memory() -> Self {
        Db::Memory(MemoryDb::new())
    }

    /// Opens the database described by `url` (see the table above).
    ///
    /// ```
    /// # paquets::orm::block_on(async {
    /// let db = paquets::Db::connect("memory").await?;
    /// assert_eq!(db.kind(), "memory");
    /// # Ok::<(), paquets::orm::Error>(())
    /// # }).unwrap();
    /// ```
    pub async fn connect(url: &str) -> Result<Self> {
        let url = url.trim();
        let scheme = url.split_once(':').map_or("", |(s, _)| s);
        if let Some(folder) = nuage_folder(url) {
            return Self::nuage_local(folder);
        }
        match Self::kind_of(url) {
            "memory" => match memory_file(url) {
                Some(path) => {
                    // `memory://data/app.memoire`: the folder is created like the file.
                    create_folder(path)?;
                    Ok(Db::Memory(MemoryDb::open(path)?))
                }
                None => Ok(Db::memory()),
            },
            #[cfg(feature = "sqlite")]
            "sqlite" => {
                let path = url
                    .strip_prefix("sqlite://")
                    .or_else(|| url.strip_prefix("sqlite:"))
                    .unwrap_or(url);
                if path.is_empty() || path == ":memory:" {
                    Ok(Db::Sqlite(SqliteDb::open_in_memory()?))
                } else {
                    // `sqlite://data/app.db`: the folder is created like the file.
                    create_folder(path)?;
                    Ok(Db::Sqlite(SqliteDb::open(path)?))
                }
            }
            #[cfg(feature = "postgres")]
            "postgres" => Ok(Db::Postgres(PgPool::connect(url).await?)),
            #[cfg(feature = "mysql")]
            "mysql" => Ok(Db::MySql(
                MySqlPool::connect(&url.replacen("mariadb://", "mysql://", 1)).await?,
            )),
            #[cfg(feature = "redis")]
            "redis" => Ok(Db::Redis(RedisDb::connect(url).await?)),
            #[cfg(feature = "mongodb")]
            "mongodb" => Ok(Db::Mongo(MongoDb::connect(url).await?)),
            #[cfg(feature = "nuage")]
            "nuage" => {
                let token = env("NUAGE_TOKEN");
                let branch = env("NUAGE_BRANCH");
                Self::nuage_server(url, token.as_deref(), branch.as_deref()).await
            }
            "unknown" => Err(memoire_orm::Error::Unsupported(format!(
                "unknown database URL scheme `{scheme}` (expected https:// or nuage: for Nuage DB, memory, sqlite, postgres, mysql, redis or mongodb)"
            ))),
            kind => Err(memoire_orm::Error::Unsupported(format!(
                "`{scheme}` URLs need the `{kind}` feature: paquets = {{ git = \"https://github.com/maxlestage/Paquets\", features = [\"{kind}\"] }}"
            ))),
        }
    }

    /// Nuage DB: a server for an `https://` or `http://` URL, with `token`, or
    /// the local mode for anything else, a folder (`"data"`), where the database
    /// is kept by Memoire's engine, the one Nuage DB runs statements with.
    ///
    /// It is what `NUAGE_URL` and `NUAGE_TOKEN` open: the same code uses a folder
    /// in development and a server in production.
    ///
    /// ```
    /// # paquets::orm::block_on(async {
    /// # let folder = std::env::temp_dir().join(format!("paquets-doc-{}", std::process::id()));
    /// # let folder = folder.to_str().unwrap();
    /// let db = paquets::Db::nuage(folder, None).await?; // e.g. "data"
    /// assert_eq!(db.kind(), "memory");
    /// # drop(db); std::fs::remove_dir_all(folder).unwrap();
    /// # Ok::<(), paquets::orm::Error>(())
    /// # }).unwrap();
    /// ```
    pub async fn nuage(target: &str, token: Option<&str>) -> Result<Self> {
        let target = target.trim();
        if Self::kind_of(target) != "nuage" {
            return Self::nuage_local(nuage_folder(target).unwrap_or(target));
        }
        #[cfg(feature = "nuage")]
        {
            let branch = env("NUAGE_BRANCH");
            Self::nuage_server(target, token, branch.as_deref()).await
        }
        #[cfg(not(feature = "nuage"))]
        {
            let _ = token;
            Self::connect(target).await
        }
    }

    /// The local mode of Nuage DB: the folder is created if it is missing, and
    /// the database is kept in `app.memoire` inside it.
    fn nuage_local(folder: &str) -> Result<Self> {
        let folder = if folder.is_empty() { "." } else { folder };
        let file = std::path::Path::new(folder).join(LOCAL_FILE);
        std::fs::create_dir_all(folder).map_err(|e| memoire_orm::Error::Database(Box::new(e)))?;
        Ok(Db::Memory(MemoryDb::open(file)?))
    }

    /// A Nuage DB server, on the branch `branch` if one is given: it is made
    /// from the main database the first time (instant, copy-on-write), for a
    /// preview or a test.
    #[cfg(feature = "nuage")]
    async fn nuage_server(url: &str, token: Option<&str>, branch: Option<&str>) -> Result<Self> {
        let mut builder = NuageDb::builder(url);
        if let Some(token) = token {
            builder = builder.token(token);
        }
        let db = builder.connect().await?;
        let Some(branch) = branch else {
            return Ok(Db::Nuage(db));
        };
        let exists = |list: Vec<memoire_orm::nuage::Branch>| list.iter().any(|b| b.name == branch);
        // Another instance may have made it in the meantime.
        if !exists(db.branches().await?)
            && let Err(error) = db.create_branch(branch, None).await
            && !exists(db.branches().await?)
        {
            return Err(error);
        }
        Ok(Db::Nuage(db.on_branch(branch)))
    }

    /// The backend named by `url`, whether or not its feature is enabled.
    pub(crate) fn kind_of(url: &str) -> &'static str {
        let url = url.trim();
        let lower = url.to_ascii_lowercase();
        let scheme = lower.split_once("://").map(|(s, _)| s);
        if nuage_folder(&lower).is_some() {
            return "memory";
        }
        match scheme {
            None if lower.is_empty() || lower == "memory" || lower == "memory:" => "memory",
            Some("memory") => "memory",
            Some("http" | "https") => "nuage",
            Some("sqlite") => "sqlite",
            Some("postgres" | "postgresql") => "postgres",
            Some("mysql" | "mariadb") => "mysql",
            Some("redis" | "rediss") => "redis",
            Some("mongodb" | "mongodb+srv") => "mongodb",
            Some(_) => "unknown",
            None if lower.starts_with("sqlite:")
                || [".db", ".sqlite", ".sqlite3"]
                    .iter()
                    .any(|ext| lower.ends_with(ext)) =>
            {
                "sqlite"
            }
            None => "unknown",
        }
    }

    /// The backend: `"memory"` (Nuage DB's local mode too), `"sqlite"`,
    /// `"postgres"`, `"mysql"`, `"redis"`, `"mongodb"` or `"nuage"` (a Nuage DB
    /// server).
    pub fn kind(&self) -> &'static str {
        match self {
            Db::Memory(_) => "memory",
            #[cfg(feature = "sqlite")]
            Db::Sqlite(_) => "sqlite",
            #[cfg(feature = "postgres")]
            Db::Postgres(_) => "postgres",
            #[cfg(feature = "mysql")]
            Db::MySql(_) => "mysql",
            #[cfg(feature = "redis")]
            Db::Redis(_) => "redis",
            #[cfg(feature = "mongodb")]
            Db::Mongo(_) => "mongodb",
            #[cfg(feature = "nuage")]
            Db::Nuage(_) => "nuage",
        }
    }
}

/// The file of Nuage DB's local mode, in its folder.
const LOCAL_FILE: &str = "app.memoire";

/// The folder of `nuage:data` (or `nuage://data`), the local mode of Nuage DB.
fn nuage_folder(url: &str) -> Option<&str> {
    let lower = url.get(..6)?.to_ascii_lowercase();
    if lower != "nuage:" {
        return None;
    }
    let rest = &url[6..];
    Some(rest.strip_prefix("//").unwrap_or(rest))
}

/// An environment variable that is set and not empty.
pub(crate) fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// The file of `memory://path`: `None` for a database that is only in memory.
fn memory_file(url: &str) -> Option<&str> {
    let (_, path) = url.split_once("://")?;
    (!path.is_empty()).then_some(path)
}

/// Creates the folder of a database file, if it is missing.
fn create_folder(path: &str) -> Result<()> {
    match std::path::Path::new(path).parent() {
        Some(dir) => {
            std::fs::create_dir_all(dir).map_err(|e| memoire_orm::Error::Database(Box::new(e)))
        }
        None => Ok(()),
    }
}

impl Default for Db {
    fn default() -> Self {
        Db::memory()
    }
}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Db").field(&self.kind()).finish()
    }
}

impl From<MemoryDb> for Db {
    fn from(db: MemoryDb) -> Self {
        Db::Memory(db)
    }
}

#[cfg(feature = "sqlite")]
impl From<SqliteDb> for Db {
    fn from(db: SqliteDb) -> Self {
        Db::Sqlite(db)
    }
}

#[cfg(feature = "postgres")]
impl From<PgPool> for Db {
    fn from(db: PgPool) -> Self {
        Db::Postgres(db)
    }
}

#[cfg(feature = "mysql")]
impl From<MySqlPool> for Db {
    fn from(db: MySqlPool) -> Self {
        Db::MySql(db)
    }
}

#[cfg(feature = "redis")]
impl From<RedisDb> for Db {
    fn from(db: RedisDb) -> Self {
        Db::Redis(db)
    }
}

#[cfg(feature = "mongodb")]
impl From<MongoDb> for Db {
    fn from(db: MongoDb) -> Self {
        Db::Mongo(db)
    }
}

#[cfg(feature = "nuage")]
impl From<NuageDb> for Db {
    fn from(db: NuageDb) -> Self {
        Db::Nuage(db)
    }
}

impl Executor for Db {
    fn dialect(&self) -> Option<Dialect> {
        each!(self, Db, d => d.dialect())
    }

    async fn fetch_all<O>(&self, query: Query<'_>) -> Result<Vec<O>>
    where
        O: FromRow + Send,
    {
        each!(self, Db, d => d.fetch_all(query).await)
    }

    async fn execute(&self, query: Query<'_>) -> Result<ExecResult> {
        each!(self, Db, d => d.execute(query).await)
    }

    async fn insert_rows(&self, table: &'static TableSchema, rows: Vec<Vec<Value>>) -> Result<u64> {
        each!(self, Db, d => d.insert_rows(table, rows).await)
    }
}

impl Database for Db {
    type Tx<'c> = DbTransaction;

    async fn begin(&self) -> Result<DbTransaction> {
        Ok(match self {
            Db::Memory(d) => DbTransaction::Memory(d.begin().await?),
            #[cfg(feature = "sqlite")]
            Db::Sqlite(d) => DbTransaction::Sqlite(d.begin().await?),
            #[cfg(feature = "postgres")]
            Db::Postgres(d) => DbTransaction::Postgres(d.begin().await?),
            #[cfg(feature = "mysql")]
            Db::MySql(d) => DbTransaction::MySql(d.begin().await?),
            #[cfg(feature = "redis")]
            Db::Redis(d) => DbTransaction::Redis(d.begin().await?),
            #[cfg(feature = "mongodb")]
            Db::Mongo(d) => DbTransaction::Mongo(d.begin().await?),
            #[cfg(feature = "nuage")]
            Db::Nuage(d) => DbTransaction::Nuage(d.begin().await?),
        })
    }
}

/// A transaction opened with [`Db::begin`](Database::begin): every change is
/// discarded unless you call [`commit`](Transaction::commit).
///
/// ```
/// use paquets::prelude::*;
///
/// #[model]
/// struct Account {
///     id: i64,
///     balance: i64,
/// }
///
/// async fn transfer(db: &Db, from: i64, to: i64, amount: i64) -> Result<()> {
///     let tx = db.begin().await?;
///     Account::update_many().decrement(Account::BALANCE, amount).filter(Account::ID.eq(from)).execute(&tx).await?;
///     Account::update_many().increment(Account::BALANCE, amount).filter(Account::ID.eq(to)).execute(&tx).await?;
///     tx.commit().await?;
///     Ok(())
/// }
/// ```
#[non_exhaustive]
// A transaction lives for a few statements: boxing the big variants would cost an
// allocation for nothing.
#[allow(clippy::large_enum_variant)]
pub enum DbTransaction {
    /// In memory.
    Memory(MemoryTransaction),
    /// SQLite.
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteTransaction),
    /// PostgreSQL.
    #[cfg(feature = "postgres")]
    Postgres(PgTransaction),
    /// MySQL / MariaDB.
    #[cfg(feature = "mysql")]
    MySql(MySqlTransaction),
    /// Redis.
    #[cfg(feature = "redis")]
    Redis(RedisTransaction),
    /// MongoDB.
    #[cfg(feature = "mongodb")]
    Mongo(MongoTransaction),
    /// A Nuage DB server.
    #[cfg(feature = "nuage")]
    Nuage(NuageTransaction),
}

impl Executor for DbTransaction {
    fn dialect(&self) -> Option<Dialect> {
        each!(self, DbTransaction, d => d.dialect())
    }

    async fn fetch_all<O>(&self, query: Query<'_>) -> Result<Vec<O>>
    where
        O: FromRow + Send,
    {
        each!(self, DbTransaction, d => d.fetch_all(query).await)
    }

    async fn execute(&self, query: Query<'_>) -> Result<ExecResult> {
        each!(self, DbTransaction, d => d.execute(query).await)
    }

    async fn insert_rows(&self, table: &'static TableSchema, rows: Vec<Vec<Value>>) -> Result<u64> {
        each!(self, DbTransaction, d => d.insert_rows(table, rows).await)
    }
}

impl Transaction for DbTransaction {
    async fn commit(self) -> Result<()> {
        each!(self, DbTransaction, d => d.commit().await)
    }

    async fn rollback(self) -> Result<()> {
        each!(self, DbTransaction, d => d.rollback().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_name_their_backend() {
        assert_eq!(Db::kind_of(""), "memory");
        assert_eq!(Db::kind_of("memory"), "memory");
        assert_eq!(Db::kind_of("memory://data/app.memoire"), "memory");
        assert_eq!(Db::kind_of("sqlite://app.db"), "sqlite");
        assert_eq!(Db::kind_of("sqlite::memory:"), "sqlite");
        assert_eq!(Db::kind_of("data/app.sqlite3"), "sqlite");
        assert_eq!(Db::kind_of("postgres://u:p@host:5432/db"), "postgres");
        assert_eq!(Db::kind_of("postgresql://host/db"), "postgres");
        assert_eq!(Db::kind_of("mariadb://host/db"), "mysql");
        assert_eq!(Db::kind_of("rediss://host:6379"), "redis");
        assert_eq!(Db::kind_of("mongodb+srv://cluster/db"), "mongodb");
        assert_eq!(Db::kind_of("ftp://nope"), "unknown");
        assert_eq!(Db::kind_of("https://db.example.com"), "nuage");
        assert_eq!(Db::kind_of("http://127.0.0.1:8080"), "nuage");
        assert_eq!(Db::kind_of("nuage:data"), "memory");
        assert_eq!(Db::kind_of("NUAGE://data"), "memory");
        assert_eq!(nuage_folder("nuage:data"), Some("data"));
        assert_eq!(nuage_folder("nuage:///srv/app"), Some("/srv/app"));
        assert_eq!(nuage_folder("data"), None);
    }

    #[test]
    fn nuage_local_mode_keeps_the_data_in_its_folder() {
        use memoire_orm::Model;

        #[derive(memoire_orm::Model, Clone, Debug)]
        #[memoire(table = "notes")]
        struct Note {
            id: i64,
            text: String,
        }

        let dir = std::env::temp_dir().join(format!("paquets-nuage-{}", std::process::id()));
        let folder = dir.join("data");
        memoire_orm::block_on(async {
            let db = Db::nuage(folder.to_str().unwrap(), None).await.unwrap();
            db.create_table_if_not_exists::<Note>().await.unwrap();
            Note::insert_many(
                &db,
                &[Note {
                    id: 1,
                    text: "kept".into(),
                }],
            )
            .await
            .unwrap();
            drop(db);

            let db = Db::connect(&format!("nuage:{}", folder.display()))
                .await
                .unwrap();
            db.create_table_if_not_exists::<Note>().await.unwrap();
            assert_eq!(Note::get(&db, 1).await.unwrap().text, "kept");
        });
        assert!(folder.join(LOCAL_FILE).is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(feature = "nuage")]
    #[tokio::test]
    async fn a_nuage_server_that_does_not_answer_is_an_error() {
        assert!(
            Db::nuage("http://127.0.0.1:9", Some("token"))
                .await
                .is_err()
        );
    }

    #[test]
    fn opens_memory_and_sqlite() {
        memoire_orm::block_on(async {
            assert_eq!(Db::connect("memory").await.unwrap().kind(), "memory");
            #[cfg(feature = "sqlite")]
            assert_eq!(
                Db::connect("sqlite::memory:").await.unwrap().kind(),
                "sqlite"
            );
            let err = Db::connect("ftp://nope").await.unwrap_err();
            assert!(err.to_string().contains("unknown database URL scheme"));
        });
    }

    #[test]
    fn a_memory_database_can_live_in_a_file() {
        use memoire_orm::Model;

        #[derive(memoire_orm::Model, Clone, Debug)]
        #[memoire(table = "notes")]
        struct Note {
            id: i64,
            text: String,
        }

        let dir = std::env::temp_dir().join(format!("paquets-memory-{}", std::process::id()));
        let url = format!(
            "memory://{}",
            dir.join("data").join("app.memoire").display()
        );
        assert_eq!(memory_file("memory://"), None);
        assert_eq!(memory_file("memory"), None);
        memoire_orm::block_on(async {
            let db = Db::connect(&url).await.unwrap();
            db.create_table_if_not_exists::<Note>().await.unwrap();
            let notes = [Note {
                id: 1,
                text: "kept".into(),
            }];
            Note::insert_many(&db, &notes).await.unwrap();
            drop(db);

            let db = Db::connect(&url).await.unwrap();
            db.create_table_if_not_exists::<Note>().await.unwrap();
            assert_eq!(Note::get(&db, 1).await.unwrap().text, "kept");
        });
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn a_sqlite_file_gets_its_folder() {
        let dir = std::env::temp_dir().join(format!("paquets-sqlite-{}", std::process::id()));
        let file = dir.join("data").join("app.db");
        memoire_orm::block_on(async {
            let db = Db::connect(&format!("sqlite://{}", file.display()))
                .await
                .unwrap();
            assert_eq!(db.kind(), "sqlite");
        });
        assert!(file.is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
