//! The database: one handle for every backend of `memoire-orm`, chosen by a URL.

use std::fmt;

use memoire_orm::{
    Database, Dialect, ExecResult, Executor, FromRow, MemoryDb, MemoryTransaction, Query, Result,
    Transaction,
};

#[cfg(feature = "mongodb")]
use memoire_orm::{MongoDb, MongoTransaction};
#[cfg(feature = "mysql")]
use memoire_orm::{MySqlPool, MySqlTransaction};
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
/// The backend comes from the `DATABASE_URL` environment variable (or
/// [`App::database`](crate::App::database)):
///
/// | URL | Backend | Cargo feature |
/// |---|---|---|
/// | *(none)*, `memory` | in memory, emptied when the server stops | always |
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
        match Self::kind_of(url) {
            "memory" => Ok(Db::memory()),
            #[cfg(feature = "sqlite")]
            "sqlite" => {
                let path = url
                    .strip_prefix("sqlite://")
                    .or_else(|| url.strip_prefix("sqlite:"))
                    .unwrap_or(url);
                if path.is_empty() || path == ":memory:" {
                    Ok(Db::Sqlite(SqliteDb::open_in_memory()?))
                } else {
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
            "unknown" => Err(memoire_orm::Error::Unsupported(format!(
                "unknown database URL scheme `{scheme}` (expected memory, sqlite, postgres, mysql, redis or mongodb)"
            ))),
            kind => Err(memoire_orm::Error::Unsupported(format!(
                "`{scheme}` URLs need the `{kind}` feature: paquets = {{ version = \"0.1\", features = [\"{kind}\"] }}"
            ))),
        }
    }

    /// The backend named by `url`, whether or not its feature is enabled.
    pub(crate) fn kind_of(url: &str) -> &'static str {
        let url = url.trim();
        let lower = url.to_ascii_lowercase();
        let scheme = lower.split_once("://").map(|(s, _)| s);
        match scheme {
            None if lower.is_empty() || lower == "memory" || lower == "memory:" => "memory",
            Some("memory") => "memory",
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

    /// The backend: `"memory"`, `"sqlite"`, `"postgres"`, `"mysql"`, `"redis"` or
    /// `"mongodb"`.
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
        }
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
        assert_eq!(Db::kind_of("sqlite://app.db"), "sqlite");
        assert_eq!(Db::kind_of("sqlite::memory:"), "sqlite");
        assert_eq!(Db::kind_of("data/app.sqlite3"), "sqlite");
        assert_eq!(Db::kind_of("postgres://u:p@host:5432/db"), "postgres");
        assert_eq!(Db::kind_of("postgresql://host/db"), "postgres");
        assert_eq!(Db::kind_of("mariadb://host/db"), "mysql");
        assert_eq!(Db::kind_of("rediss://host:6379"), "redis");
        assert_eq!(Db::kind_of("mongodb+srv://cluster/db"), "mongodb");
        assert_eq!(Db::kind_of("ftp://nope"), "unknown");
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
}
