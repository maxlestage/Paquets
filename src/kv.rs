//! The key-value store of an app: see [`Kv`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use memoire_orm::{Error, Result};

/// Memory of the store of this process: past it, the keys that expired go, then others.
const LOCAL_MAX_BYTES: usize = 64 << 20;

/// The key-value store: a cache, sessions, counters, rate limits and locks, the way Redis
/// is used.
///
/// Handlers get it with [`req.kv()`](crate::RequestExt::kv):
///
/// ```
/// use std::time::Duration;
/// use paquets::prelude::*;
///
/// async fn visits(req: Request) -> Result<String> {
///     let kv = req.kv();
///     // A counter, with the window of a rate limit: it starts over every minute.
///     let this_minute = kv.incr_by("visits:minute", 1, Some(Duration::from_secs(60))).await?;
///     // A cache in one call: read, or computed and kept five minutes.
///     let total: i64 = kv.remember("visits:total", Duration::from_secs(300), || async { Ok(42) }).await?;
///     // One worker at a time: a lock that frees itself after 30 seconds.
///     if kv.set_if_absent("lock:report", "me", Some(Duration::from_secs(30))).await? {
///         kv.delete("lock:report").await?;
///     }
///     Ok(format!("{this_minute} this minute, {total} in all"))
/// }
/// ```
///
/// On a Nuage DB server (`NUAGE_URL=https://…`), it is the application's store on the
/// server (`NuageKv`, a Nuage DB of 0.4.4 or later): every instance of the app shares it.
/// With any other database (the local mode, memory, SQLite, PostgreSQL...), it is a store in
/// this process's memory, with the same methods: an app is written once, and runs alone
/// on a laptop as it does on several servers. Either way it is memory: a restart empties
/// it, and what must survive goes in the database.
///
/// Cloning it is cheap: the clones are the same store.
#[derive(Clone)]
pub struct Kv(Inner);

#[derive(Clone)]
enum Inner {
    Local(Arc<Local>),
    #[cfg(feature = "nuage")]
    Nuage(memoire_orm::NuageKv),
}

impl std::fmt::Debug for Kv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.is_shared() {
            "Kv(Nuage DB)"
        } else {
            "Kv(this process)"
        })
    }
}

#[derive(Default)]
struct Local {
    entries: Mutex<Entries>,
}

#[derive(Default)]
struct Entries {
    map: HashMap<String, (Vec<u8>, Option<Instant>)>,
    bytes: usize,
}

impl Entries {
    /// The entry of `key`, unless its time has come.
    fn alive(&mut self, key: &str) -> Option<&mut (Vec<u8>, Option<Instant>)> {
        if self
            .map
            .get(key)
            .is_some_and(|(_, at)| at.is_some_and(|at| at <= Instant::now()))
        {
            self.remove(key);
        }
        self.map.get_mut(key)
    }

    fn remove(&mut self, key: &str) -> bool {
        match self.map.remove(key) {
            Some((value, _)) => {
                self.bytes -= key.len() + value.len();
                true
            }
            None => false,
        }
    }

    fn write(&mut self, key: &str, value: Vec<u8>, expires: Option<Instant>) {
        self.remove(key);
        self.bytes += key.len() + value.len();
        self.map.insert(key.to_owned(), (value, expires));
        if self.bytes > LOCAL_MAX_BYTES {
            let now = Instant::now();
            let expired: Vec<String> = self
                .map
                .iter()
                .filter(|(_, (_, at))| at.is_some_and(|at| at <= now))
                .map(|(k, _)| k.clone())
                .collect();
            for k in expired {
                self.remove(&k);
            }
            while self.bytes > LOCAL_MAX_BYTES {
                let Some(other) = self.map.keys().find(|k| k.as_str() != key).cloned() else {
                    break;
                };
                self.remove(&other);
            }
        }
    }
}

impl Local {
    fn with<T>(&self, f: impl FnOnce(&mut Entries) -> T) -> T {
        f(&mut self.entries.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

/// The store of this process: one for the whole app.
fn local() -> Arc<Local> {
    static LOCAL: std::sync::OnceLock<Arc<Local>> = std::sync::OnceLock::new();
    LOCAL.get_or_init(Arc::default).clone()
}

fn after(ttl: Duration) -> Option<Instant> {
    Instant::now().checked_add(ttl.max(Duration::from_millis(1)))
}

impl Kv {
    /// The store that goes with `db`: the application's store on a Nuage DB server, the
    /// store of this process otherwise.
    pub fn of(db: &crate::Db) -> Kv {
        match db {
            #[cfg(feature = "nuage")]
            crate::Db::Nuage(nuage) => Kv(Inner::Nuage(nuage.kv())),
            _ => Kv(Inner::Local(local())),
        }
    }

    /// A store of its own in this process's memory: for a test.
    pub fn in_memory() -> Kv {
        Kv(Inner::Local(Arc::default()))
    }

    /// The store is on a Nuage DB server: every instance of the app shares it.
    pub fn is_shared(&self) -> bool {
        match &self.0 {
            Inner::Local(_) => false,
            #[cfg(feature = "nuage")]
            Inner::Nuage(_) => true,
        }
    }

    /// The store on the Nuage DB server, for what only it does (`command`, `pipeline`: the
    /// commands of Redis); `None` for the store of this process.
    #[cfg(feature = "nuage")]
    pub fn nuage(&self) -> Option<&memoire_orm::NuageKv> {
        match &self.0 {
            Inner::Local(_) => None,
            Inner::Nuage(kv) => Some(kv),
        }
    }

    /// The value of `key`; `None` when there is none (or it expired).
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| e.alive(key).map(|(v, _)| v.clone()))),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.get(key).await,
        }
    }

    /// The value of `key` as text.
    pub async fn get_string(&self, key: &str) -> Result<Option<String>> {
        match self.get(key).await? {
            Some(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| Error::Conversion(format!("the value of `{key}` is not UTF-8 text"))),
            None => Ok(None),
        }
    }

    /// Writes `value` under `key`, until it is deleted or overwritten.
    pub async fn set(&self, key: &str, value: impl AsRef<[u8]>) -> Result<()> {
        match &self.0 {
            Inner::Local(l) => {
                l.with(|e| e.write(key, value.as_ref().to_vec(), None));
                Ok(())
            }
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.set(key, value).await,
        }
    }

    /// Writes `value` under `key`; it expires after `ttl`.
    pub async fn set_for(&self, key: &str, value: impl AsRef<[u8]>, ttl: Duration) -> Result<()> {
        match &self.0 {
            Inner::Local(l) => {
                l.with(|e| e.write(key, value.as_ref().to_vec(), after(ttl)));
                Ok(())
            }
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.set_for(key, value, ttl).await,
        }
    }

    /// Writes `value` under `key` only when the key does not exist; `true` when it was
    /// written. With a `ttl`, a lock that frees itself if its holder never does.
    pub async fn set_if_absent(
        &self,
        key: &str,
        value: impl AsRef<[u8]>,
        ttl: Option<Duration>,
    ) -> Result<bool> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| {
                if e.alive(key).is_some() {
                    return false;
                }
                e.write(key, value.as_ref().to_vec(), ttl.and_then(after));
                true
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.set_if_absent(key, value, ttl).await,
        }
    }

    /// Deletes `key`; `true` when it existed.
    pub async fn delete(&self, key: &str) -> Result<bool> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| e.alive(key).is_some() && e.remove(key))),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.delete(key).await,
        }
    }

    /// `key` exists.
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| e.alive(key).is_some())),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.exists(key).await,
        }
    }

    /// Adds 1 to the counter `key` (0 when it does not exist) and returns it.
    pub async fn incr(&self, key: &str) -> Result<i64> {
        self.incr_by(key, 1, None).await
    }

    /// Adds `by` to the counter `key` and returns it. `ttl_if_new` is the expiry of a
    /// counter this call creates: `incr_by(key, 1, Some(window))` counts the events of a
    /// window, a rate limit in one step.
    pub async fn incr_by(&self, key: &str, by: i64, ttl_if_new: Option<Duration>) -> Result<i64> {
        match &self.0 {
            Inner::Local(l) => l.with(|e| {
                let (current, expires) = match e.alive(key) {
                    Some((value, expires)) => {
                        let n = std::str::from_utf8(value)
                            .ok()
                            .and_then(|s| s.parse::<i64>().ok());
                        (
                            n.ok_or_else(|| {
                                Error::Validation(format!(
                                    "the value of `{key}` is not a whole number"
                                ))
                            })?,
                            *expires,
                        )
                    }
                    None => (0, ttl_if_new.and_then(after)),
                };
                let next = current.checked_add(by).ok_or_else(|| {
                    Error::Validation(format!("the counter `{key}` would overflow"))
                })?;
                e.write(key, next.to_string().into_bytes(), expires);
                Ok(next)
            }),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.incr_by(key, by, ttl_if_new).await,
        }
    }

    /// Makes `key` expire after `ttl`; `false` when it does not exist.
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| match e.alive(key) {
                Some(entry) => {
                    entry.1 = after(ttl);
                    true
                }
                None => false,
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.expire(key, ttl).await,
        }
    }

    /// Makes `key` never expire; `false` when it does not exist.
    pub async fn persist(&self, key: &str) -> Result<bool> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| match e.alive(key) {
                Some(entry) => {
                    entry.1 = None;
                    true
                }
                None => false,
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.persist(key).await,
        }
    }

    /// How long `key` still lives: `None` when it never expires, or does not exist.
    pub async fn ttl(&self, key: &str) -> Result<Option<Duration>> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| {
                e.alive(key)
                    .and_then(|(_, at)| *at)
                    .map(|at| at.saturating_duration_since(Instant::now()))
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.ttl(key).await,
        }
    }

    /// The keys that start with `prefix` (all of them for `""`), sorted.
    pub async fn keys(&self, prefix: &str) -> Result<Vec<String>> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| {
                let now = Instant::now();
                let mut keys: Vec<String> = e
                    .map
                    .iter()
                    .filter(|(k, (_, at))| k.starts_with(prefix) && at.is_none_or(|at| at > now))
                    .map(|(k, _)| k.clone())
                    .collect();
                keys.sort();
                keys
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.keys(prefix).await,
        }
    }

    /// Deletes every key; returns how many there were.
    pub async fn clear(&self) -> Result<u64> {
        match &self.0 {
            Inner::Local(l) => Ok(l.with(|e| {
                let n = e.map.len() as u64;
                *e = Entries::default();
                n
            })),
            #[cfg(feature = "nuage")]
            Inner::Nuage(kv) => kv.clear().await,
        }
    }

    /// The value of `key`, read from the JSON [`set_json`](Self::set_json) wrote.
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| {
                Error::Conversion(format!(
                    "the value of `{key}` is not the JSON expected: {e}"
                ))
            }),
            None => Ok(None),
        }
    }

    /// Writes `value` under `key` as JSON; with a `ttl`, it expires.
    pub async fn set_json<T: serde::Serialize + ?Sized>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|e| Error::Conversion(e.to_string()))?;
        match ttl {
            Some(ttl) => self.set_for(key, bytes, ttl).await,
            None => self.set(key, bytes).await,
        }
    }

    /// A cache in one call: the value kept under `key`, or else what `compute` answers,
    /// kept for `ttl`. A value kept that does not read as `T` (the type changed) is
    /// computed again and replaced.
    pub async fn remember<T, F, Fut>(&self, key: &str, ttl: Duration, compute: F) -> Result<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        if let Some(bytes) = self.get(key).await?
            && let Ok(value) = serde_json::from_slice(&bytes)
        {
            return Ok(value);
        }
        let value = compute().await?;
        self.set_json(key, &value, Some(ttl)).await?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_store_of_this_process() {
        let kv = Kv::in_memory();
        assert!(!kv.is_shared());
        assert_eq!(kv.get("a").await.unwrap(), None);
        kv.set("a", "1").await.unwrap();
        assert_eq!(kv.get_string("a").await.unwrap().as_deref(), Some("1"));
        assert_eq!(kv.incr_by("a", 41, None).await.unwrap(), 42);
        assert_eq!(kv.incr("n").await.unwrap(), 1);
        kv.set("text", "x").await.unwrap();
        assert!(kv.incr("text").await.is_err());

        // A lock, until it expires.
        let short = Some(Duration::from_millis(30));
        assert!(kv.set_if_absent("lock", "me", short).await.unwrap());
        assert!(!kv.set_if_absent("lock", "you", short).await.unwrap());
        assert!(kv.ttl("lock").await.unwrap().unwrap() <= Duration::from_millis(30));
        assert_eq!(kv.ttl("a").await.unwrap(), None);
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(!kv.exists("lock").await.unwrap());
        assert!(kv.set_if_absent("lock", "you", None).await.unwrap());

        // A counter made with a window keeps it.
        let minute = Some(Duration::from_secs(60));
        assert_eq!(kv.incr_by("rate", 1, minute).await.unwrap(), 1);
        assert_eq!(
            kv.incr_by("rate", 1, Some(Duration::from_secs(600)))
                .await
                .unwrap(),
            2
        );
        assert!(kv.ttl("rate").await.unwrap().unwrap() <= Duration::from_secs(60));
        assert!(kv.persist("rate").await.unwrap());
        assert_eq!(kv.ttl("rate").await.unwrap(), None);
        assert!(kv.expire("rate", Duration::from_secs(5)).await.unwrap());
        assert!(!kv.expire("nope", Duration::from_secs(5)).await.unwrap());

        // JSON, and a cache in one call.
        kv.set_json("user", &serde_json::json!({ "name": "Ada" }), None)
            .await
            .unwrap();
        assert_eq!(
            kv.get_json::<serde_json::Value>("user")
                .await
                .unwrap()
                .unwrap()["name"],
            "Ada"
        );
        let mut computed = 0;
        for _ in 0..3 {
            let report: Vec<u32> = kv
                .remember("report", Duration::from_secs(60), || {
                    computed += 1;
                    async { Ok(vec![1, 2, 3]) }
                })
                .await
                .unwrap();
            assert_eq!(report, [1, 2, 3]);
        }
        assert_eq!(computed, 1);

        assert_eq!(kv.keys("r").await.unwrap(), ["rate", "report"]);
        assert!(kv.delete("a").await.unwrap() && !kv.delete("a").await.unwrap());
        assert_eq!(kv.clear().await.unwrap(), 6);
        assert!(kv.keys("").await.unwrap().is_empty());
        // Another store of its own shares nothing.
        kv.set("mine", "1").await.unwrap();
        assert_eq!(Kv::in_memory().get("mine").await.unwrap(), None);
    }
}
