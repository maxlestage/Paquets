//! # Paquets 📦
//!
//! **The simplest full-stack web framework for Rust.** One crate, one import, and
//! your app has a fast server, security, a database, a REST API, SEO-ready pages and
//! interactive WebAssembly islands.
//!
//! ```no_run
//! use paquets::prelude::*;
//!
//! #[model]
//! struct Todo {
//!     id: i64,
//!     title: String,
//!     done: bool,
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     let mut app = App::new();
//!
//!     app.get("/", |_| async {
//!         Page::new("Home").body(h1().text("Hello 👋"))
//!     });
//!     app.api::<Todo>("/api/todos"); // GET, POST, PUT, PATCH, DELETE
//!
//!     app.run() // http://localhost:3000, or $PORT on Heroku
//! }
//! ```
//!
//! ## Built on four packages
//!
//! Paquets assembles four crates, each of which can also be used on its own:
//!
//! | Package | Role in Paquets | Re-exported as |
//! |---|---|---|
//! | [vitesse](https://vitesse-f16135654c3d.herokuapp.com/) | HTTP server in the style of Express: routes, middlewares, WebSocket | [`vitesse`], [`App`] dereferences to `vitesse::App` |
//! | [securite](https://securite-2db4c1994be7.herokuapp.com/) | security in front of every request: WAF, rate limit, CSRF, headers… | [`securite`], [`App::security`] |
//! | [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/) | database: memory, SQLite, PostgreSQL, MySQL, Redis, MongoDB | [`orm`], [`Db`], [`macro@model`] |
//! | [active](https://active-c1883a966650.herokuapp.com/) | pages rendered on the server, SEO tags, WebAssembly islands | [`active`], [`Page`] |
//!
//! ## In handlers
//!
//! A handler is an `async` function (or closure) that takes a [`Request`](vitesse::Request) and
//! returns anything that becomes a response: text, [`Json`](vitesse::Json), a [`Page`], a status,
//! a `Result`… Use `?` on any error: database errors become `404`, `409` or `400`,
//! the others `500` (see [`Error`]).
//!
//! ```
//! use paquets::prelude::*;
//!
//! #[model]
//! struct Todo {
//!     id: i64,
//!     title: String,
//!     done: bool,
//! }
//!
//! async fn todos_page(req: Request) -> Result<Page> {
//!     let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
//!     Ok(Page::new("My todos").body(
//!         main()
//!             .child(h1().text("My todos"))
//!             .child(ul().children(todos.into_iter().map(|t| li().text(t.title)))),
//!     ))
//! }
//! ```
//!
//! ## Guide
//!
//! The complete documentation, in English, French and Spanish, is on the
//! [Paquets website](https://paquets-90dfc06b6fea.herokuapp.com/) and in the
//! [`docs/` folder](https://paquets-90dfc06b6fea.herokuapp.com/en/docs/) of the
//! repository: getting started, every concept, deployment to Heroku from a phone,
//! publishing on crates.io.
//!
//! ## Cargo features
//!
//! | Feature | Default | Adds |
//! |---|---|---|
//! | `sqlite` | ✓ | SQLite (`DATABASE_URL=sqlite://app.db`) |
//! | `postgres` | ✓ | PostgreSQL with TLS (Heroku Postgres) |
//! | `mysql`, `redis`, `mongodb` | | the other databases |
//! | `chrono`, `uuid`, `json` | | more column types |
//! | `encryption` | | in-memory databases stored in an encrypted file |
//! | `tls`, `http2`, `http3` | | HTTPS ([`App::tls`]), HTTP/2, HTTP/3 over QUIC |
//! | `session`, `templates`, `multipart` | | signed cookies and sessions, minijinja templates, file uploads |
//! | `compress`, `tower` | | Brotli or gzip responses, tower layers as middlewares |
//! | `full` | | everything above |
//! | `client` | | browser side (`wasm32`): hydrate islands, client-side apps |
//! | `browser-storage` | | `orm::web`: a state store and a database in `localStorage`, for islands |

#![doc(
    html_logo_url = "https://paquets-90dfc06b6fea.herokuapp.com/assets/logo.svg",
    html_favicon_url = "https://paquets-90dfc06b6fea.herokuapp.com/assets/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod api;
#[cfg(not(target_arch = "wasm32"))]
mod app;
#[cfg(not(target_arch = "wasm32"))]
mod db;
#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod page;
#[cfg(not(target_arch = "wasm32"))]
mod pwa;
#[cfg(not(target_arch = "wasm32"))]
mod shield;

#[cfg(not(target_arch = "wasm32"))]
pub use api::Resource;
#[cfg(not(target_arch = "wasm32"))]
pub use app::{App, DbSource, RequestExt};
#[cfg(not(target_arch = "wasm32"))]
pub use db::{Db, DbTransaction};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{Error, Result};
#[cfg(not(target_arch = "wasm32"))]
pub use page::{Page, THEME};
#[cfg(not(target_arch = "wasm32"))]
pub use shield::{default_shield, shield};

/// The web app manifest of an installable app: see [`App::pwa`].
#[cfg(not(target_arch = "wasm32"))]
pub use active::pwa::Manifest as Pwa;

/// The version of this crate, e.g. `"0.1.1"`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Front end: views, state, SEO tags, islands ([active](https://active-c1883a966650.herokuapp.com/)).
pub use active;
/// Database: models, queries, transactions, migrations ([memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/)).
pub use memoire_orm as orm;
/// Security middlewares ([securite](https://securite-2db4c1994be7.herokuapp.com/)).
#[cfg(not(target_arch = "wasm32"))]
pub use securite;
pub use serde;
pub use serde_json;
/// HTTP server ([vitesse](https://vitesse-f16135654c3d.herokuapp.com/)).
#[cfg(not(target_arch = "wasm32"))]
pub use vitesse;
/// Testing an application without opening a port: see [`App::test`].
#[cfg(not(target_arch = "wasm32"))]
pub use vitesse::test;

pub use paquets_macros::{data, model};

/// Browser side: hydrates the islands rendered by the server.
///
/// ```ignore
/// #[wasm_bindgen::prelude::wasm_bindgen(start)]
/// pub fn start() {
///     paquets::hydrate(&[("counter", counter)]);
/// }
/// ```
#[cfg(feature = "client")]
pub use active::{hydrate, location, mount, mount_to, navigate, query, query_param, update_head};

/// Everything an application needs: `use paquets::prelude::*;`
pub mod prelude {
    // Every HTML tag (`div()`, `h1()`…), and the module itself: in a `main.rs`,
    // `main` is the program's function, so the tag is written `html::main()`.
    pub use active::html::{self, *};
    pub use active::{
        Element, Event, Node, Seo, State, effect, el, fragment, fragment_dyn, indexed, island,
        keyed, memo, on_cleanup, switch, text, text_dyn, untrack, use_state,
    };
    pub use memoire_orm::{Database, Executor, LoadRelated, Model, Transaction};
    pub use serde_json::json;

    pub use crate::{data, model};

    #[cfg(feature = "client")]
    pub use crate::{hydrate, location, mount_to, navigate, query_param};

    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::{App, Db, Error, Page, Pwa, RequestExt, Result};
    #[cfg(not(target_arch = "wasm32"))]
    pub use vitesse::{
        Cookie, HandlerExt, Html, IntoResponse, Json, Method, Next, Redirect, Request, Response,
        Router, StatusCode, middleware, res, ws,
    };
}

/// Compiles the code examples of the README.
#[cfg(all(doctest, not(target_arch = "wasm32")))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
