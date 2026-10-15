# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
