# The four packages

Paquets is built on four packages by the same author: vitesse, securite, memoire-orm and active. Each one is a complete library that you can also use on its own. Paquets connects them, chooses the defaults, and re-exports them, so that one dependency and one import are enough. This page shows what each package brings, how Paquets uses it, and how to reach the parts that the prelude doesn't include.

## Overview

| Package | Role | Re-exported as | Repository | Documentation |
|---|---|---|---|---|
| **vitesse** | HTTP server in the style of Express.js: routes, middleware, routers, WebSocket, HTTP/3, test client | `paquets::vitesse` | [maxlestage/Vitesse](https://github.com/maxlestage/Vitesse) | [docs](https://github.com/maxlestage/Vitesse/blob/master/docs/en/README.md) |
| **securite** | security middleware: WAF, rate limit, CSRF, CORS, security headers and CSP, bot blocker, authentication… | `paquets::securite` | [maxlestage/Securite](https://github.com/maxlestage/Securite) | [docs](https://github.com/maxlestage/Securite/blob/master/docs/en/introduction.md) |
| **memoire-orm** | type-safe async ORM, SQL built at compile time: memory, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB | `paquets::orm` | [maxlestage/Memoire](https://github.com/maxlestage/Memoire) | [docs](https://github.com/maxlestage/Memoire/blob/master/docs/en/introduction.md) |
| **active** | front end: components as Rust functions, `use_state`, server rendering with complete SEO tags, WebAssembly islands | `paquets::active` | [maxlestage/Active](https://github.com/maxlestage/Active) | [docs](https://github.com/maxlestage/Active/blob/master/site/docs/en/introduction.md) |

Paquets also re-exports `paquets::serde` and `paquets::serde_json` (and `json!` in the prelude), so that `#[model]` and `#[data]` work without adding serde to your `Cargo.toml`, and `paquets::test`, the test client types.

## vitesse: the server

[vitesse](https://github.com/maxlestage/Vitesse) (French for "speed") is the HTTP server, with its own HTTP/1.1 engine on top of tokio. It handles about 291,000 requests per second on 2 cores for `GET /`, more than actix-web and axum on the same machine, and 2.78 million with pipelining.

**In Paquets:** `App` contains a `vitesse::App` and gives access to all its methods (it dereferences to it): `get`, `post`, `ws`, `middleware`, `mount`, `state`, `fallback`, `on_error`, `body_limit`, `static_dir`, `workers`… Handlers receive a `vitesse::Request` and return anything that implements `vitesse::IntoResponse`. The test client of `app.test()` is vitesse's `TestClient`.

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

- `paquets::vitesse::tokio`: the tokio runtime vitesse runs on (`tokio::spawn`, `tokio::sync`, `tokio::time`…), without adding tokio to your dependencies;
- `paquets::vitesse::Error`: the error type of `on_error`, and of `with_source` (see [Errors](errors.md));
- `paquets::vitesse::ws::Upgrade`: WebSocket handshakes with subprotocols and size limits;
- HTTP/3 with the `http3` feature (see [vitesse's HTTP/3 guide](https://github.com/maxlestage/Vitesse/blob/master/docs/en/http3.md)).

## securite: the shield

[securite](https://github.com/maxlestage/Securite) is a set of 19 security guards, assembled into a `Shield`. Its recommended pipeline costs about 6 µs per request. It is written for the tower ecosystem (axum, hyper, tonic) and contains no `unsafe` code.

**In Paquets:** `App::new()` puts `paquets::shield().build()` in front of every request: securite's recommended preset, with the client IP read from `X-Forwarded-For` on Heroku. Paquets bridges vitesse's requests to securite's guards: the shield sees the request line and the headers, adds its headers to every response, and hands its findings to your handlers (the client IP, the authenticated user, the request id). Pages compute their own Content-Security-Policy with securite's hashing functions.

**To use it:**

```rust
use paquets::securite::prelude::*;

app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
```

`paquets::securite::prelude::*` brings every guard (`RateLimit`, `Waf`, `Csrf`, `Cors`, `Auth`, `IpFilter`, `HostFilter`, `SecurityHeaders`…), `Shield`, `ShieldBuilder`, `ClientIpSource`, `ErrorFormat`, `ClientIp` and `Principal`. `paquets::securite::headers::ContentSecurityPolicy` builds policies. See [Security](security.md).

## memoire-orm: the database

[memoire-orm](https://github.com/maxlestage/Memoire) maps Rust structs to tables and writes queries in Rust, checked by the compiler. The SQL of primary-key operations is built at compile time, and the same code runs on six backends.

**In Paquets:**

- `#[model]` adds `#[derive(Model)]` (and serde) to your struct, and points the generated code to `paquets::orm`, so your project doesn't need its own `memoire-orm` dependency;
- `Db` is one handle for every backend, chosen by `DATABASE_URL`, and implements memoire's `Executor` and `Database` traits: every model method and query accepts `req.db()`;
- `app.api::<M>()` and `app.model::<M>()` create tables at startup; database errors become `404`, `409` or `400` through `?`.

**In the prelude:** the traits `Model`, `Executor`, `Database` and `Transaction`, which give models and the database their methods.

**Beyond the prelude**, through `paquets::orm`: `Migrator` and `Migration`, `Dialect`, the `params!` macro for raw SQL, `Page` (the result of `paginate`), `Json<T>` columns, and the backends themselves (`SqliteDb`, `PgPool`, `MySqlPool`, `RedisDb`, `MongoDb`, `MemoryDb`), to open a database with your own settings and pass it to `app.database(Db::from(...))`. See [Database](database.md).

## active: the front end

[active](https://github.com/maxlestage/Active) builds views with one function per HTML tag, renders them on the server with complete SEO tags, and turns marked parts of a page into WebAssembly islands. It has no macros, no template language and no dependency on the server side; its counter bundle weighs 36.8 kB compressed.

**In Paquets:** `Page` wraps active's `Document`: it adds the theme, the Content-Security-Policy and the loading of the islands' bundle (found in `pkg/` at startup). Islands are active's islands, hydrated in the browser by `paquets::hydrate` (feature `client`).

**In the prelude:** every HTML tag function (`div()`, `p()`, `a()`…) and the `html` module (for `html::main()`), `Element`, `Node`, `Seo`, `State`, `use_state`, `effect`, `island`, `el`, `text`, `text_dyn`, `fragment`, `fragment_dyn` and `keyed`.

**Beyond the prelude**, through `paquets::active`: `Document` to render a page without the theme and security policy of `Page`, for example to export static files. With the `client` feature (browser side): `paquets::hydrate`, `paquets::mount_to` and `paquets::mount` for apps rendered entirely in the browser, `paquets::navigate`, `paquets::location`, `paquets::query` and `paquets::query_param` for routing without reloading. See [Islands](islands.md) and [active's documentation](https://github.com/maxlestage/Active/blob/master/site/docs/en/introduction.md).

## How the names map

| In Paquets | In the package |
|---|---|
| `App` | `vitesse::App`, plus the shield, the database, the REST API, static files and islands |
| `app.run()` | `vitesse::App::run(port)`, with the port from `PORT` |
| `app.security(shield)` | a `securite::Shield` |
| `paquets::shield()` | `securite::ShieldBuilder::recommended()` with Heroku's client IP |
| `#[model]` | `#[derive(memoire_orm::Model, Serialize, Deserialize, Clone, Debug)]` with `#[memoire(crate = "::paquets::orm")]` |
| `#[data]` | `#[derive(Serialize, Deserialize, Clone, Debug)]` |
| `Db`, `req.db()` | any memoire backend, behind one type |
| `Page` | `active::Document`, plus the theme, the CSP and the bundle |
| `paquets::Error`, `Result<T>` | `vitesse::Error`, with the right status for memoire's errors |

## Using a package on its own

Each package is published separately and works without Paquets:

- **vitesse** for a server without a database or pages, or with your own choices;
- **securite** in front of an axum, hyper or tonic service: `.layer(Shield::recommended())`;
- **memoire-orm** in any Rust program that stores data, a command line tool or a worker;
- **active** for a static site, a client-side app, or pages served by another framework (its documentation has an axum guide).

Their READMEs explain how to start. All four use the 2024 edition. Paquets, vitesse, securite and memoire-orm are licensed under MIT or Apache-2.0, active under MIT.

## Versions

Paquets 0.1 depends on vitesse 0.1, securite 0.1, memoire-orm 0.1 and active 0.1. Until they are on crates.io, Cargo takes them from their GitHub repositories; see [Publish on crates.io](publish-crates-io.md) for the publication order.

## Next steps

- [Introduction](introduction.md): what Paquets adds on top of the four packages.
- [Cheat sheet](cheatsheet.md): the whole API on one page.
- [FAQ](faq.md): common questions.
