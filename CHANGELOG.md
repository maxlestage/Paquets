# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.3] - 2026-11-13

### Added
- Nuage DB, the database of Memoire written entirely in Rust, is the default database of new
  projects. `app.nuage("data")` keeps the data in the `data/` folder in development (Nuage
  DB's local mode: Memoire's engine, the one Nuage DB runs statements with, in
  `data/app.memoire`), and `NUAGE_URL` and `NUAGE_TOKEN` point the same code to a Nuage DB
  server in production. `NUAGE_BRANCH` runs the app on a branch of the server, made from the
  main database at the first start if it is missing: previews and tests on a copy of real data.
- The `nuage` feature, on by default: the client of a Nuage DB server (`memoire-orm/nuage`),
  `Db::Nuage` and `DbTransaction::Nuage`, `Db::nuage(target, token)`, `From<NuageDb> for Db`,
  and `paquets::orm::NuageDb`, also in islands (`features = ["client", "nuage"]`).
- `Db::connect` and `DATABASE_URL` accept a Nuage DB server (`https://…`, `http://…`, with the
  token of `NUAGE_TOKEN`) and the local mode (`nuage:data`).
- `paquets new`: `app.nuage("data")` in `src/main.rs`, `/data` in `.gitignore` and
  `.dockerignore`, a `data/` folder the app can write in the `Dockerfile`, `NUAGE_URL` and
  `NUAGE_TOKEN` in `app.json`, and the README of the project explains the server.
- Documentation: a Nuage DB page (development and production, running and deploying a
  server, connecting an app, branches for previews and tests, `/admin` and `/app`, moving an
  existing project), in English, French and Spanish; the database, installation, quick start,
  configuration, deployment, command line, generators, FAQ and cheat sheet pages follow.
- Website: a "Nuage DB" section on the home page, and Nuage DB in the ecosystem of the footer.

### Changed
- `NUAGE_URL` wins over `DATABASE_URL`, which wins over the folder of `app.nuage(...)`.
  Existing apps, which set neither `NUAGE_URL` nor `app.nuage(...)`, keep their database:
  `DATABASE_URL`, in memory without it. The startup line says where Nuage DB is
  (`database: nuage, local (data)`, `database: nuage (https://…)`), and an app on Heroku
  without `NUAGE_URL` warns that its `data/` folder is erased at each restart.
- With a Nuage DB server, the app runs on a multi-threaded tokio runtime, as with PostgreSQL.
- memoire-orm 0.2.1 (it was 0.2.0), the version with the Nuage DB client.
- Rust 1.95 or newer (it was 1.89): Memoire now uses `parking_lot` 0.13, which needs it (Active,
  Securite and pratique need 1.90). The MSRV job of the CI checks 1.95.
- Active, Vitesse, Securite, pratique and Memoire on their latest commits. Through
  `paquets::vitesse`, the `templates` feature uses minijinja 3 (it was 2):
  `paquets::vitesse::templates::minijinja` is minijinja 3, so an environment given to
  `App::templates` comes from minijinja 3. Through `paquets::orm`, the `encryption` feature
  uses `aes-gcm` 0.11 and `argon2` 0.6 (files encrypted by earlier versions still open), and
  `rusqlite` 0.41.
- `paquets-macros` is built with `syn` 3 (it was 2): `#[model]` and `#[data]` generate the same
  code and the same error messages.
- Website: `wasm-bindgen` 0.2.130 (it was 0.2.129), with the same `wasm-bindgen-cli` in the
  CI, the `Dockerfile` and `heroku-build.sh` (read from `site/Cargo.lock`) and in the docs.
  The browser bundle is no longer stripped by Cargo, as `wasm-bindgen` 0.2.130 reads its
  `target_features` section; `wasm-bindgen` drops the names and producers sections itself
  (`--remove-name-section --remove-producers-section`), and the bundle is slightly smaller.
- Every other dependency brought to its latest compatible version.

## [0.3.2] - 2026-11-09

### Added
- Active 0.3: what its 3D lacked. Through `paquets::scene`: textures (`Texture::from_url`,
  `from_rgba8`, `from_fn`, with repeat, offset, rotation, filters, mipmaps and anisotropic
  filtering) as `map` on every material, plus `emissive_map`, `normal_map` (no tangents
  needed) and `roughness_metalness_map` on `MeshStandardMaterial`, and `alpha_test` for
  cut-outs; `SpotLight` (cone and penumbra); shadows of directional lights and spots
  (`cast_shadow`, `receive_shadow`, `LightShadow` with map size, biases, soft edges and an
  `extent` for large scenes); `Sprite`, an image that faces the camera. Up to 16 point lights
  instead of 8, the nearest to the camera kept when there are more.
- Through `paquets::ecs`: `Image` assets and `AssetServer::load`, `base_color_texture`,
  `emissive_texture`, `normal_map_texture` and `metallic_roughness_texture` on
  `StandardMaterial`, `AlphaMode::Mask`, `SpotLight`, `shadows_enabled` now drawn,
  `NotShadowCaster`, `NotShadowReceiver`, `DirectionalLightShadowMap`,
  `PointLightShadowMap`, and 2D: `Camera2d`, `Sprite` (with `Anchor`, flips, sizes and sprite
  sheet rectangles), `Mesh2d` and `ColorMaterial`.

### Changed
- `active` 0.3 (it was 0.2): its scene types gained fields (materials, `Draw`, `Object3D`)
  and variants (`Light::Spot`, `ObjectKind::Sprite`). Code building them with
  `..Default::default()` or their builders is unchanged; `Draw::new` builds a mesh to draw.

## [0.3.1] - 2026-11-08

### Changed
- Active's physics runs on its new engine (`paquets::physics`, and the physics of
  `paquets::ecs` and `paquets::scene`): eight times faster on Active's 3D demo (0.22 ms per
  frame instead of 1.85), with the same API; the laws of mechanics Active checks still hold.
  The physics is Active's `physics` feature, on by default.
- Rust 1.89 or newer (it was 1.88): the physics engine is written with `nalgebra` 0.35, which
  needs it. The MSRV job of the CI checks 1.89.
- Vitesse's engine reads the clock for the `date` header once per batch of events instead of
  once per connection (2% less CPU per request), and its benchmark was measured again:
  3.29 million pipelined requests per second on two cores, twice actix-web.
- Every dependency brought to its latest compatible version.

## [0.3.0] - 2026-11-07

### Added
- 3D, games and physics, Active's three engines: `paquets::scene` (a scene of meshes,
  lights and cameras drawn with WebGL 2, `OrbitControls`, `animation_loop`), `paquets::ecs`
  (the game engine: entities, components, systems, an `App` with plugins and schedules, 3D in
  physical light units, `app.run_in(canvas)` in an island) and `paquets::physics` (every 3D
  object has a mass from its volume, forces, gravity, friction, rolling, bounces and
  collisions, checked against the formulas of mechanics; built into `paquets::ecs` with
  `Mass`, `LinearVelocity`, `ExternalForce`, `CollisionStarted`… and into
  `paquets::scene::Physics`). Guide: Islands, "3D in an island".
- Vitesse measured again: 3.29 million pipelined requests per second on two cores, twice
  actix-web; +36% with a real browser request.
- Faster page changes: `Page::view_transitions()` (the View Transitions API, as in Astro and
  Next), `Page::prefetch(Prefetch::Hover)` and `Page::prerender(...)` (speculation rules, with
  a fallback for the other browsers); `Prefetch` is in the prelude. Their inline code is added
  to the page's Content-Security-Policy by its hash, and nothing else.
- Documented: `db.transaction(...)`, Memoire's transaction that commits, rolls back on an
  error and runs again on a conflict (like Prisma's `$transaction`), on `req.db()`; and
  `db.push::<M>()`, the call behind the tables brought up to date at startup.
- Tables follow the models: at startup, `app.api` / `app.model` now bring each table up to date
  with Memoire's `push` (like `drizzle-kit push`): a field added to a struct becomes a column,
  new indexes are created, nothing is ever dropped. A new field existing rows could not fill
  (not an `Option`, no `default`) stops the start with a clear message, before any change.
- `paquets::pratique` and `Idempotency` in the prelude: `app.middleware(Idempotency::new())`
  makes a `POST` sent again with the same `Idempotency-Key` get the first answer again instead
  of running twice (payments, orders).
- `use_query` / `use_mutation` in the prelude: Active's query cache shared by key, with
  background revalidation, invalidation by prefix and optimistic updates.
- `IndexedDbStorage` (feature `browser-storage`): a Memoire `KeyValueStorage` kept in the
  browser's IndexedDB, so a `Store` or a whole `LocalStorageDb` — models, queries,
  transactions — can live in IndexedDB instead of `localStorage`: far more room, and writes
  that do not block the page. Read once when opened, then served from memory; writes reach
  IndexedDB in the background, in order.

### Fixed
- The first page rendered with `view_transitions()`, `prefetch(...)` or `prerender(...)`
  no longer fails with "this state was disposed" (a 500) when it holds islands with state:
  the hashes of these options are now found after the page is rendered.
- With SQLite or the in-memory database, the tasks spawned by `app.on_start` hooks
  (`tokio::spawn`: a purge every minute, a cache refreshed in the background…) were cancelled
  as soon as the server started, because the runtime of the startup was dropped: it now keeps
  running on a thread of its own (`paquets-tasks`), for as long as the server.

### Changed
- `DATABASE_URL=sqlite://data/app.db` creates the folder of the file when it is missing, like
  `memory://data/app.memoire`.

## [0.2.0] - 2026-10-15

### Changed
- `Cargo.lock` (and the site's) on the latest commits of Active, Vitesse, Securite and Memoire.
- On the latest versions of the four packages, all reachable through Paquets:
  Vitesse (request validation with 422 details, OpenAPI 3.1 and Swagger UI,
  lifecycle hooks, `app.param`, route groups and plugins), Securite (Argon2id
  password hashing, login throttling, gates and policies, signed URLs, TOTP
  two-factor codes, trusted proxies), Memoire (timestamps, soft deletes,
  lifecycle hooks, `save`, `first_or_create`, chunks and cursor pagination, query
  log) and active (watch, provide/inject, two-way binding, route patterns,
  resources and suspense, error boundaries, island hydration strategies,
  navigation guards, transitions, forms with validation, portals).
- Links: the websites of Paquets, Active, Vitesse, Securite and Memoire are linked at their
  Heroku addresses (`*.herokuapp.com`) instead of GitHub Pages, in the READMEs, the docs, the
  site's footer, `Cargo.toml`, `app.json`, the rustdoc logo and the `paquets new` template; the
  old `/#/docs` links point to real pages (`/<lang>/docs/`).
- Credits: FerroSwift and Solidify are no longer listed in the ecosystem (READMEs, site footer,
  `PUBLISHING.md`): the footer presents five projects.
- Heroku: the website is built with the emk/rust buildpack on the heroku-26 stack by default
  (`RustConfig` sets the Rust version and skips the buildpack's own build, `heroku-build.sh`
  builds the server and the islands, `Procfile`; `app.json` chooses them for the Deploy button).
  The Deploy to Heroku workflow takes an optional `HEROKU_BUILD` variable:
  `buildpack` (default) or `docker`, which keeps the Docker build (`heroku.yml`, `Dockerfile`,
  "container" stack); it switches the app's stack when needed. Apps created by `paquets new`
  are unchanged.
- Website: no longer published on GitHub Pages (the `pages.yml` workflow is removed): it is
  served by Heroku only, and GitHub only hosts the code. The READMEs, the automation docs and
  `site/README.md` no longer describe a Pages deployment; the static export keeps working and
  is still checked by CI.
- Website: the version shown at the top of the home page (and in its terminals) is
  read from `Cargo.toml`, so it follows every release (it said `v0.1` after 0.1.1).

### Added
- Authentication in one line, like Laravel Breeze: `app.auth(Auth::new())` (`paquets::auth`)
  adds sign-up, login, logout, forgotten passwords and TOTP two-factor codes, with their
  pages, on Securite's Argon2id `PasswordHasher`, `LoginThrottle`, signed cookies and links
  (`Signer`) and `Totp`; the `User` model (table `users`), `req.user()` (`AuthExt`, in the
  prelude), the `login_required` and `admin_required` middlewares, a pluggable mailer for
  reset links, administrators by e-mail address, key rotation (`Auth::previous_secret`).
  Two-factor secrets are encrypted at rest (Securite's `Encrypter`, XChaCha20-Poly1305, a key
  derived from `APP_KEY` with HMAC-SHA256, bound to the user's row; secrets stored in clear are
  still read and encrypted at the next two-factor login); `Debug` of `User` redacts the
  password hash and the secret; a missing `APP_KEY` is reported at startup in every build.
- An admin panel for models, like Django's admin: `app.admin(Admin::new().model::<Todo>())`
  (`paquets::admin`, `Admin` in the prelude): dashboard, lists with search, sort, boolean
  filters and pages, forms to create and edit rows typed by column, deletion with a
  confirmation; secrets hidden; access for administrators, a rule (`allow`) or a Securite
  `Gate`.
- API documentation in one line: `app.api_docs("My API")` serves the OpenAPI 3.1 document at
  `/openapi.json` and Swagger UI at `/docs` (with its own CSP); the routes of `app.api::<M>()`
  are now documented with the model's JSON Schema.
- `app.health("/health")` (database check, `503` when down) and `app.metrics("/metrics")`
  (Prometheus counters and histogram, local or `METRICS_TOKEN`); `paquets new` projects have
  the health check and the API documentation.
- `app.migrate(Migrator)` (versioned migrations at startup), `app.seed(…)`, and `app.routes()`
  with `RouteInfo`.
- Command line: generators `paquets make:model`, `make:api`, `make:page`, `make:migration`,
  `make:test`, `make:auth` and `make:admin`, which write idiomatic code and wire it into
  `src/main.rs`; `paquets routes`, `paquets db:migrate`, `paquets db:seed` (the app answers
  `PAQUETS_COMMAND` instead of serving); `paquets dev --watch`, which rebuilds and restarts the
  app at every change while the open pages reload by themselves (Server-Sent Events).
- CI: the template job also runs every generator on a third project and checks it.
- Documentation, in English, French and Spanish: Authentication, Admin panel, Generators and
  Operations pages; the command line page lists the new commands.
- Releases without a form: once CI is green on `master`, a new `version` in `Cargo.toml`
  is tagged and released on GitHub with its changelog section, then published on crates.io
  when the four packages are there.

## [0.1.1] - 2026-10-12

### Changed
- From the latest Vitesse: an app listening on port 0 with HTTP/3 picks another pair of
  ports when the UDP twin of the chosen TCP port is unavailable (reserved on Windows),
  instead of failing to start.
- Website: the share image is lighter (281 kB: WhatsApp drops previews over about
  300 kB), and every card gives its size and description.
- The shield of every app is Securite's own Vitesse middleware (Securite now protects
  Vitesse applications natively): the bridge between tower and Vitesse that Paquets
  carried is gone, with its `tower-layer`, `tower-service` and `http` dependencies.
- License: Paquets is proprietary software by Maxime Nathan Lestage, all rights reserved
  (see `LICENSE`); it is free to use as a dependency, commercially too, but not to modify
  or redistribute. The MIT/Apache-2.0 dual license is withdrawn. Projects created by
  `paquets new` stay yours, under the license you choose.
- Credits: Maxime Nathan Lestage is named as the author in the crate metadata, the README
  and the website's footer, which now links the seven projects of the ecosystem.
- `paquets::VERSION`, the version of the crate.
- Built on the latest Vitesse, Securite, memoire-orm and Active. `Db` passes Memoire's
  `insert_rows` on to its backend, so `insert_many` stores rows in the in-memory engine
  without copying them; the service worker of installable apps is versioned with
  Active's `pwa::version`.
- Credits: Solidify joins the projects linked by the README, the license and the
  website's footer.

### Added
- Vitesse's options as Cargo features, all in `full`: `tls` (HTTPS in the server, with
  `App::tls`; the startup line then says `https://`), `http2`, `session` (signed cookies
  and sessions), `templates` (minijinja), `multipart` (file uploads), `compress` (Brotli
  or gzip responses) and `tower` (tower layers as middlewares).
- `DATABASE_URL=memory://data/app.memoire`: the in-memory engine kept in a file and read
  back at startup; the `encryption` feature encrypts such files with a password.
- The `browser-storage` feature: Memoire's `web` module in islands, a Pinia-style
  `Store` and `LocalStorageDb`, both kept in `localStorage`.
- In the prelude: Active's `memo`, `untrack`, `on_cleanup`, `indexed` and `switch`, and
  Memoire's `LoadRelated` (`users.load_children::<Post>(db)`).
- Documentation, in English, French and Spanish: related rows, full-text search and
  indexes over several columns, the in-memory database kept in a file, HTTPS in the
  server, sessions, file uploads, compression, browser storage in islands, and Pratique,
  Securite's everyday middlewares, with a Paquets app.
- `Pwa::new(...).start_url("/{lang}/")`: one manifest per language, so an installed app
  opens in the language of the page it was installed from.
- `app.pwa(Pwa::new(...))`: installable apps (PWA) with a web app manifest and an offline
  service worker (Active's `pwa` module), linked by every page; the default icons are the
  Paquets box. Projects created by `paquets new` are installable, and so is the website.

## [0.1.0] - 2026-10-10

### Added
- `App`: a Vitesse application with Securite's shield, a request log, the database of
  `DATABASE_URL`, static files from `public/` and the islands bundle from `pkg/`; the port
  from `PORT`. `app.run()`, `app.serve().await`, `app.test().await`.
- `app.api::<M>(path)`: a REST API for a model (list with `limit`/`offset` and
  `X-Total-Count`, show, create, replace, partial update, delete), tables created at startup;
  `app.model::<M>()`, `app.on_start(...)`, `app.database(...)`, `app.security(...)`,
  `app.port(...)`, `app.quiet()`.
- `Db`: one handle for every memoire-orm backend (memory, SQLite, PostgreSQL, MySQL/MariaDB,
  Redis, MongoDB), chosen by URL, with transactions (`DbTransaction`). `req.db()`.
- `Page`: server-rendered pages with merged SEO tags, a light/dark theme, and a
  Content-Security-Policy with the hashes of the page's inline styles and scripts; islands
  load the browser bundle automatically.
- `Error`: `?` on any error, with the right status for database errors (404, 409, 400).
- `#[model]` and `#[data]` (`paquets-macros`): models and JSON payloads without other
  dependencies.
- The `paquets` command: `new` (complete project with Dockerfile, Heroku files and GitHub
  Actions that merge green branches into master), `dev`, `build`, `doctor`.
- Website built with Paquets itself (Active pages, WebAssembly islands, static export
  for GitHub Pages), documentation in English, French and Spanish,
  Heroku deployment from a phone, GitHub Pages, crates.io release workflow.
