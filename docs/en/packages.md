# The four packages

Paquets is built on four packages by the same author: Vitesse, Securite, memoire-orm and Active. Each one is a complete library that you can also use on its own. Paquets connects them, chooses the defaults, and re-exports them, so that one dependency and one import are enough. This page shows what each package brings, how Paquets uses it, and how to reach the parts that the prelude doesn't include.

## Overview

| Package | Role | Re-exported as | Repository | Documentation |
|---|---|---|---|---|
| **Vitesse** | HTTP server in the style of Express.js: routes, middleware, routers, WebSocket, HTTPS, HTTP/2 and HTTP/3, sessions, templates, test client | `paquets::vitesse` | [maxlestage/Vitesse](https://maxlestage.github.io/Vitesse/) | [docs](https://maxlestage.github.io/Vitesse/en/docs/README/) |
| **Securite** | security middleware: WAF, rate limit, CSRF, CORS, security headers and CSP, bot blocker, authentication… | `paquets::securite` | [maxlestage/Securite](https://maxlestage.github.io/Securite/) | [docs](https://maxlestage.github.io/Securite/en/docs/introduction/) |
| **memoire-orm** | type-safe async ORM, SQL built at compile time: memory, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB | `paquets::orm` | [maxlestage/Memoire](https://maxlestage.github.io/Memoire/) | [docs](https://maxlestage.github.io/Memoire/en/docs/introduction/) |
| **active** | front end: components as Rust functions, `use_state`, server rendering with complete SEO tags, WebAssembly islands | `paquets::active` | [maxlestage/Active](https://maxlestage.github.io/Active/) | [docs](https://maxlestage.github.io/Active/en/docs/introduction/) |

Paquets also re-exports `paquets::serde` and `paquets::serde_json` (and `json!` in the prelude), so that `#[model]` and `#[data]` work without adding serde to your `Cargo.toml`, and `paquets::test`, the test client types.

## vitesse: the server

[Vitesse](https://maxlestage.github.io/Vitesse/) (French for "speed") is the HTTP server, with its own HTTP/1.1 engine on top of tokio. It handles about 291,000 requests per second on 2 cores for `GET /`, more than actix-web and axum on the same machine, and 2.78 million with pipelining.

**In Paquets:** `App` contains a `vitesse::App` and gives access to all its methods (it dereferences to it): `get`, `post`, `ws`, `middleware`, `mount`, `state`, `fallback`, `on_error`, `body_limit`, `static_dir`, `workers`… Handlers receive a `vitesse::Request` and return anything that implements `vitesse::IntoResponse`. The test client of `app.test()` is Vitesse's `TestClient`.

**In the prelude:** `Request`, `Response`, `IntoResponse`, `Json`, `Html`, `Redirect`, `Cookie`, `StatusCode`, `Method`, `Router`, `Next`, `HandlerExt`, and the modules `res`, `middleware` and `ws`.

**Beyond the prelude**, through `paquets::vitesse`:

```rust
use std::time::Duration;

let mut app = App::new();
// vitesse's built-in middleware: CORS, timeout, logger, helmet.
app.middleware(middleware::timeout(Duration::from_secs(10)));
// A folder with its own cache settings.
app.serve_dir("/assets", paquets::vitesse::ServeDir::new("assets").max_age(Duration::from_secs(3600)));
```

- `paquets::vitesse::tokio`: the tokio runtime Vitesse runs on (`tokio::spawn`, `tokio::sync`, `tokio::time`…), without adding tokio to your dependencies;
- `paquets::vitesse::Error`: Vitesse's own error type, wrapped by `paquets::Error` (`err.into_inner()` gives it back);
- `paquets::vitesse::ws::Upgrade`: WebSocket handshakes with subprotocols and size limits;
- `paquets::vitesse::sse`: Server-Sent Events, a stream of events to the browser;
- the middlewares `request_id`, `basic_auth`, `bearer_auth`, `concurrency_limit` and `rate_limit`, next to `cors`, `timeout`, `logger` and `helmet`.

Vitesse's options are Cargo features of Paquets, with the same names:

| Feature | Brings | Vitesse's guide |
|---|---|---|
| `tls`, `http2`, `http3` | HTTPS in the server (`app.tls(...)`), HTTP/2, HTTP/3 over QUIC | [HTTPS](https://maxlestage.github.io/Vitesse/en/docs/https/), [HTTP/3](https://maxlestage.github.io/Vitesse/en/docs/http3/) |
| `session` | signed cookies (`app.cookie_secret(...)`) and sessions (`middleware::session(...)`, `req.session()`) | [Sessions](https://maxlestage.github.io/Vitesse/en/docs/sessions/) |
| `templates` | minijinja templates: `app.views("views")`, `req.render("page.html", json!({...}))` | [Templates](https://maxlestage.github.io/Vitesse/en/docs/templates/) |
| `multipart` | file uploads: `req.multipart()` | [Requests](https://maxlestage.github.io/Vitesse/en/docs/requests/) |
| `compress` | Brotli or gzip responses (`middleware::compress()`), compressed request bodies (`middleware::decompress()`) | [Middleware](https://maxlestage.github.io/Vitesse/en/docs/middleware/) |
| `tower` | tower layers as middlewares: `middleware::layer(...)` | [Middleware](https://maxlestage.github.io/Vitesse/en/docs/middleware/) |

Pages rendered with `req.render` get the shield's Content-Security-Policy, which refuses inline scripts and styles: put them in `public/`, or build the page with `Page`, which allows its own (see [Security](security.md#security-headers-and-the-content-security-policy)).

## securite: the shield

[Securite](https://maxlestage.github.io/Securite/) is a set of 19 security guards, assembled into a `Shield`. Its recommended pipeline costs about 6 µs per request. It is written for Vitesse (and for the tower ecosystem: hyper, tonic) and contains no `unsafe` code.

**In Paquets:** `App::new()` puts `paquets::shield().build()` in front of every request: Securite's recommended preset, with the client IP read from `X-Forwarded-For` on Heroku. Paquets bridges Vitesse's requests to Securite's guards: the shield sees the request line and the headers, adds its headers to every response, and hands its findings to your handlers (the client IP, the authenticated user, the request id). Pages compute their own Content-Security-Policy with Securite's hashing functions.

**To use it:**

```rust
use paquets::securite::prelude::*;

app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
```

`paquets::securite::prelude::*` brings every guard (`RateLimit`, `Waf`, `Csrf`, `Cors`, `Auth`, `IpFilter`, `HostFilter`, `SecurityHeaders`…), `Shield`, `ShieldBuilder`, `ClientIpSource`, `ErrorFormat`, `ClientIp` and `Principal`. `paquets::securite::headers::ContentSecurityPolicy` builds policies. See [Security](security.md).

**Its sibling, Pratique.** Securite's repository also holds `pratique`, everyday Vitesse middlewares that have nothing to do with security: `ETag` and `304`, `Cache-Control`, a response cache, access logs, Prometheus metrics, health checks, maintenance mode, redirects, trailing slashes, the visitor's language. They work with a Paquets app as they are, since `App` is a Vitesse app:

```toml
[dependencies]
pratique = { git = "https://github.com/maxlestage/Securite" }
```

```rust
use pratique::prelude::*;

app.middleware(Health::new().version(env!("CARGO_PKG_VERSION"))); // GET /health
app.middleware(ETag::new());
```

See the [Pratique documentation](https://maxlestage.github.io/Securite/en/docs/pratique/).

## memoire-orm: the database

[memoire-orm](https://maxlestage.github.io/Memoire/) maps Rust structs to tables and writes queries in Rust, checked by the compiler. The SQL of primary-key operations is built at compile time, and the same code runs on six backends.

**In Paquets:**

- `#[model]` adds `#[derive(Model)]` (and serde) to your struct, and points the generated code to `paquets::orm`, so your project doesn't need its own `memoire-orm` dependency;
- `Db` is one handle for every backend, chosen by `DATABASE_URL`, and implements Memoire's `Executor` and `Database` traits: every model method and query accepts `req.db()`;
- `app.api::<M>()` and `app.model::<M>()` create tables at startup; database errors become `404`, `409` or `400` through `?`.

**In the prelude:** the traits `Model`, `Executor`, `Database` and `Transaction`, which give models and the database their methods, and `LoadRelated`, which loads the related rows of a list in one query per level (`users.load_children::<Post>(db)`).

**What Memoire brings lately**, all available through `req.db()`: relations read from the foreign keys ([Related rows](database.md#related-rows)), full-text search with `Column::matches`, indexes over several columns, `find_many` and `update_only`, an in-memory database kept in a file (`DATABASE_URL=memory://app.memoire`, encrypted with the `encryption` feature), and browser storage for islands (`browser-storage` feature, see [Islands](islands.md#keeping-data-in-the-browser)).

**Beyond the prelude**, through `paquets::orm`: `Migrator` and `Migration`, `Dialect`, the `params!` macro for raw SQL, `Page` (the result of `paginate`), `Json<T>` columns, `Observed` (a handle that announces every write made through it), `StorageOptions`, and the backends themselves (`SqliteDb`, `PgPool`, `MySqlPool`, `RedisDb`, `MongoDb`, `MemoryDb`), to open a database with your own settings and pass it to `app.database(Db::from(...))`. See [Database](database.md).

## active: the front end

[Active](https://maxlestage.github.io/Active/) builds views with one function per HTML tag, renders them on the server with complete SEO tags, and turns marked parts of a page into WebAssembly islands. It has no macros, no template language and no dependency on the server side; its counter bundle weighs 36.8 kB compressed.

**In Paquets:** `Page` wraps Active's `Document`: it adds the theme, the Content-Security-Policy and the loading of the islands' bundle (found in `pkg/` at startup). Islands are active's islands, hydrated in the browser by `paquets::hydrate` (feature `client`).

**In the prelude:** every HTML tag function (`div()`, `p()`, `a()`…) and the `html` module (for `html::main()`), `Element`, `Node`, `Seo`, `State`, `use_state`, `effect`, `memo` (a derived state that notifies only when its value changes), `untrack`, `on_cleanup`, `island`, `el`, `text`, `text_dyn`, `fragment`, `fragment_dyn`, `keyed`, `indexed` (a list whose rows follow their position) and `switch` (a view that keeps its page while the page's parameters change).

**Beyond the prelude**, through `paquets::active`: `Document` to render a page without the theme and security policy of `Page`, for example to export static files. With the `client` feature (browser side): `paquets::hydrate`, `paquets::mount_to` and `paquets::mount` for apps rendered entirely in the browser, `paquets::navigate`, `paquets::location`, `paquets::query` and `paquets::query_param` for routing without reloading. See [Islands](islands.md) and [Active's documentation](https://maxlestage.github.io/Active/en/docs/introduction/).

## How the names map

| In Paquets | In the package |
|---|---|
| `App` | `vitesse::App`, plus the shield, the database, the REST API, static files and islands |
| `app.run()` | `vitesse::App::run(port)`, with the port from `PORT` |
| `app.security(shield)` | a `securite::Shield` |
| `paquets::shield()` | `securite::ShieldBuilder::recommended()` with Heroku's client IP |
| `#[model]` | `#[derive(memoire_orm::Model, Serialize, Deserialize, Clone, Debug)]` with `#[memoire(crate = "::paquets::orm")]` |
| `#[data]` | `#[derive(Serialize, Deserialize, Clone, Debug)]` |
| `Db`, `req.db()` | any Memoire backend, behind one type |
| `Page` | `active::Document`, plus the theme, the CSP and the bundle |
| `paquets::Error`, `Result<T>` | `vitesse::Error`, with the right status for Memoire's errors |

## Using a package on its own

Each package is published separately and works without Paquets:

- **Vitesse** for a server without a database or pages, or with your own choices;
- **Securite** in a Vitesse app (`app.middleware(Shield::recommended())`) or in front of a hyper or tonic service (`.layer(Shield::recommended())`);
- **memoire-orm** in any Rust program that stores data, a command line tool or a worker;
- **active** for a static site, a client-side app, or pages served by another framework (its documentation has a Vitesse guide).

Their READMEs explain how to start. All four use the 2024 edition. Like Paquets, the four packages are proprietary software by Maxime Nathan Lestage, free to use as dependencies (see [Which license?](faq.md#which-license)).

## Versions

Paquets 0.1 depends on Vitesse 0.1, Securite 0.1, memoire-orm 0.1 and Active 0.1. Until they are on crates.io, Cargo takes them from their GitHub repositories; see [Publish on crates.io](publish-crates-io.md) for the publication order.

## Next steps

- [Introduction](introduction.md): what Paquets adds on top of the four packages.
- [Cheat sheet](cheatsheet.md): the whole API on one page.
- [FAQ](faq.md): common questions.
