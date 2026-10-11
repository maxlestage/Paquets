# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- License: Paquets is proprietary software by Maxime Nathan Lestage, all rights reserved
  (see `LICENSE`); it is free to use as a dependency, commercially too, but not to modify
  or redistribute. The MIT/Apache-2.0 dual license is withdrawn. Projects created by
  `paquets new` stay yours, under the license you choose.
- Credits: Maxime Nathan Lestage is named as the author in the crate metadata, the README
  and the website's footer, which now links the six projects of the ecosystem.
- `paquets::VERSION`, the version of the crate.

### Added
- `app.pwa(Pwa::new(...))`: installable apps (PWA) with a web app manifest and an offline
  service worker (active's `pwa` module), linked by every page; the default icons are the
  Paquets box. Projects created by `paquets new` are installable, and so is the website.

## [0.1.0] - 2026-10-10

### Added
- `App`: a vitesse application with securite's shield, a request log, the database of
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
- Website built with Paquets itself (active pages, WebAssembly islands, static export
  for GitHub Pages), documentation in English, French and Spanish,
  Heroku deployment from a phone, GitHub Pages, crates.io release workflow.
