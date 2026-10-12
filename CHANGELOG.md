# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-10-12

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
