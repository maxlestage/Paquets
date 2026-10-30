//! A Memoire storage kept in the browser's IndexedDB (feature `browser-storage`).
//!
//! `localStorage` holds a few megabytes of strings and blocks the page on every
//! write. IndexedDB holds far more, and writes in the background.
//! [`IndexedDbStorage`] puts it behind Memoire's [`KeyValueStorage`], so a
//! [`Store`](crate::orm::web::Store) or a whole
//! [`LocalStorageDb`](crate::orm::web::LocalStorageDb) — models, queries,
//! transactions — can live in IndexedDB:
//!
//! ```no_run
//! # async fn demo() -> paquets::orm::Result<()> {
//! use paquets::IndexedDbStorage;
//! use paquets::orm::web::LocalStorageDb;
//!
//! let storage = IndexedDbStorage::open("my-app").await?;
//! let db = LocalStorageDb::open_with("notes", storage)?;
//! # Ok(()) }
//! ```
//!
//! The whole store is read once, when it is opened; after that every read is
//! served from memory and every write is applied in memory at once, then
//! written to IndexedDB in the background (writes reach IndexedDB in the order
//! they were made). Other tabs are not watched: a change made in another tab is
//! seen at the next opening.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use memoire_orm::Result;
use memoire_orm::web::KeyValueStorage;

/// The IndexedDB object store that holds the entries, in every database.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const STORE: &str = "entries";

/// A Memoire [`KeyValueStorage`] kept in an IndexedDB database of the browser.
///
/// Cheap to clone: the clones share the same entries.
#[derive(Clone, Debug)]
pub struct IndexedDbStorage {
    name: String,
    entries: Arc<Mutex<HashMap<String, String>>>,
}

impl IndexedDbStorage {
    /// Opens (or creates) the IndexedDB database `name` and reads its entries.
    ///
    /// Fails outside the browser, or when the browser refuses IndexedDB
    /// (some private modes): use [`MemoryStorage`](crate::orm::web::MemoryStorage)
    /// or [`LocalStorage`](crate::orm::web::LocalStorage) then.
    pub async fn open(name: &str) -> Result<Self> {
        let entries = browser::open(name).await?;
        Ok(IndexedDbStorage {
            name: name.to_owned(),
            entries: Arc::new(Mutex::new(entries)),
        })
    }

    /// The name of the IndexedDB database.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn entries(&self) -> MutexGuard<'_, HashMap<String, String>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl KeyValueStorage for IndexedDbStorage {
    fn scope(&self) -> String {
        format!("indexeddb:{}", self.name)
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.entries().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        self.entries().insert(key.to_owned(), value.to_owned());
        browser::write(&self.name, key, Some(value))
    }

    fn remove(&self, key: &str) -> Result<()> {
        self.entries().remove(key);
        browser::write(&self.name, key, None)
    }

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self.entries().keys().cloned().collect())
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use memoire_orm::{Error, Result};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

    use super::STORE;

    thread_local! {
        /// The open databases, by name: a JavaScript handle stays on the
        /// page's thread, while the storage itself must be `Send + Sync`.
        static OPEN: RefCell<HashMap<String, IdbDatabase>> = RefCell::new(HashMap::new());
    }

    fn js_error(what: &str, e: JsValue) -> Error {
        let detail = e
            .dyn_ref::<js_sys::Error>()
            .map(|e| String::from(e.message()))
            .or_else(|| e.as_string())
            .unwrap_or_else(|| format!("{e:?}"));
        Error::Storage(format!("IndexedDB, {what}: {detail}"))
    }

    /// Waits for a request to succeed, and gives its result.
    async fn done(request: &IdbRequest) -> Result<JsValue> {
        let mut handlers = None;
        let promise =
            js_sys::Promise::new(&mut |resolve, reject| handlers = Some((resolve, reject)));
        let (resolve, reject) = handlers.expect("the executor runs at once");
        let ok = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let _ = resolve.call0(&JsValue::NULL);
        });
        let ko = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let _ = reject.call0(&JsValue::NULL);
        });
        request.set_onsuccess(Some(ok.as_ref().unchecked_ref()));
        request.set_onerror(Some(ko.as_ref().unchecked_ref()));
        let outcome = JsFuture::from(promise).await;
        request.set_onsuccess(None);
        request.set_onerror(None);
        match outcome {
            Ok(_) => request.result().map_err(|e| js_error("result", e)),
            Err(_) => Err(js_error(
                "request",
                request
                    .error()
                    .ok()
                    .flatten()
                    .map(JsValue::from)
                    .unwrap_or(JsValue::NULL),
            )),
        }
    }

    pub(super) async fn open(name: &str) -> Result<HashMap<String, String>> {
        let factory = web_sys::window()
            .ok_or_else(|| Error::Storage("no `window`: not running in a page".into()))?
            .indexed_db()
            .map_err(|e| js_error("open", e))?
            .ok_or_else(|| Error::Storage("IndexedDB is disabled by the browser".into()))?;
        let request: IdbOpenDbRequest = factory
            .open_with_u32(name, 1)
            .map_err(|e| js_error("open", e))?;
        // A new database gets its object store before it opens.
        let upgrade = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
            let db = e
                .target()
                .and_then(|t| t.dyn_into::<IdbOpenDbRequest>().ok())
                .and_then(|r| r.result().ok())
                .and_then(|r| r.dyn_into::<IdbDatabase>().ok());
            if let Some(db) = db
                && !db.object_store_names().contains(STORE)
            {
                let _ = db.create_object_store(STORE);
            }
        });
        request.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
        let db: IdbDatabase = done(&request)
            .await?
            .dyn_into()
            .map_err(|e| js_error("open", e))?;
        request.set_onupgradeneeded(None);

        // Keys and values come back in the same (key) order.
        let store = db
            .transaction_with_str(STORE)
            .and_then(|t| t.object_store(STORE))
            .map_err(|e| js_error("read", e))?;
        let keys = done(&store.get_all_keys().map_err(|e| js_error("read", e))?).await?;
        let values = done(&store.get_all().map_err(|e| js_error("read", e))?).await?;
        let keys = js_sys::Array::from(&keys);
        let values = js_sys::Array::from(&values);
        let mut entries = HashMap::with_capacity(keys.length() as usize);
        for (key, value) in keys.iter().zip(values.iter()) {
            if let (Some(key), Some(value)) = (key.as_string(), value.as_string()) {
                entries.insert(key, value);
            }
        }
        OPEN.with(|open| open.borrow_mut().insert(name.to_owned(), db));
        Ok(entries)
    }

    /// Writes (or with `None` deletes) one entry, in the background: the
    /// requests of one store are applied in the order they were made.
    pub(super) fn write(name: &str, key: &str, value: Option<&str>) -> Result<()> {
        OPEN.with(|open| {
            let open = open.borrow();
            let db = open
                .get(name)
                .ok_or_else(|| Error::Storage(format!("IndexedDB `{name}` is not open")))?;
            let store = db
                .transaction_with_str_and_mode(STORE, IdbTransactionMode::Readwrite)
                .and_then(|t| t.object_store(STORE))
                .map_err(|e| js_error("write", e))?;
            let key = JsValue::from_str(key);
            match value {
                Some(value) => store.put_with_key(&JsValue::from_str(value), &key),
                None => store.delete(&key),
            }
            .map(|_| ())
            .map_err(|e| js_error("write", e))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod browser {
    use std::collections::HashMap;

    use memoire_orm::{Error, Result};

    pub(super) async fn open(_name: &str) -> Result<HashMap<String, String>> {
        Err(Error::Storage(
            "IndexedDB only exists in the browser".into(),
        ))
    }

    pub(super) fn write(_name: &str, _key: &str, _value: Option<&str>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_the_browser_it_does_not_open() {
        let err = memoire_orm::block_on(IndexedDbStorage::open("app")).unwrap_err();
        assert!(err.to_string().contains("IndexedDB"), "{err}");
    }

    #[test]
    fn reads_and_writes_go_through_memory() {
        let storage = IndexedDbStorage {
            name: "app".into(),
            entries: Arc::default(),
        };
        assert_eq!(storage.scope(), "indexeddb:app");
        storage.set("a", "1").unwrap();
        let clone = storage.clone();
        assert_eq!(clone.get("a").unwrap().as_deref(), Some("1"));
        assert_eq!(storage.keys().unwrap(), ["a"]);
        storage.remove("a").unwrap();
        assert_eq!(clone.get("a").unwrap(), None);
    }
}
