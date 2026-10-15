# Paquets documentation

Everything you need to build full-stack web apps with Paquets 📦, the simplest full-stack framework for Rust: from your first page to a deployment made from your phone. These pages are also on the website: https://paquets-90dfc06b6fea.herokuapp.com/

## Getting started

- [Introduction](introduction.md): what Paquets is, what it does for you, and its limits.
- [Installation](installation.md): Rust, the `paquets` command and the optional WebAssembly tools.
- [Quick start](quick-start.md): build a small app with pages, a form, a database and a test, step by step.
- [Project structure](project-structure.md): every file created by `paquets new`, and how to organise a growing project.

## Essentials

- [Routing](routing.md): routes, parameters, routers, middleware, the 404 page and static files.
- [Requests and responses](requests-responses.md): query strings, headers, JSON and forms; text, JSON, redirects, cookies and files.
- [Pages](pages.md): views, layouts, the built-in theme, SEO tags and the Content-Security-Policy.
- [Database](database.md): models, choosing the database, queries, transactions, migrations and raw SQL.
- [REST API](rest-api.md): a complete JSON API for a model in one line, and how to protect or extend it.
- [Security](security.md): the shield in front of every request, how to change it, and authentication.

## Going further

- [Islands](islands.md): interactive parts of a page, written in Rust and run in the browser as WebAssembly.
- [WebSocket](websocket.md): two-way connections, from an echo server to a chat room.
- [Testing](testing.md): test the whole app in memory with `app.test()`, without opening a port.
- [Errors](errors.md): `Result`, the `?` operator, the status of each error, and custom error formats.
- [Configuration](configuration.md): environment variables, app settings, shared state and how the server runs.
- [Performance](performance.md): the benchmarks, how the server uses your CPU, and tips to stay fast.
- [Authentication](authentication.md): sign-up, login, password resets and two-factor codes in one line.
- [Admin panel](admin.md): list, search, create, edit and delete the rows of your models in the browser.
- [Generators](generators.md): `paquets make:model`, `make:api`, `make:page`… write the code and wire it.
- [Operations](operations.md): OpenAPI and Swagger UI, the route list, migrations and seeds, health checks, metrics, live reload.

## Deploy

- [Deploy to Heroku from your phone](deploy-heroku-mobile.md): the Deploy button, a database, automatic deployments, logs and domains, all without a computer.
- [Deploy with Docker](deploy-docker.md): the production image, volumes, docker compose and other platforms.
- [Publish on crates.io](publish-crates-io.md): the publication order of the packages, and releases from a phone.
- [Automation](automation.md): tests on every push, the automatic merge into `master`, deployments, and how to turn them off.

## Reference

- [Command line tool](cli.md): `paquets new`, `dev`, `build`, `doctor`, `routes`, `db:*` and the generators.
- [Cheat sheet](cheatsheet.md): the whole API on one page.
- [The four packages](packages.md): Vitesse, Securite, memoire-orm and Active, and how Paquets uses them.
- [FAQ](faq.md): common questions and their answers.

## More resources

- API reference generated from the source code: https://docs.rs/paquets
- Source code, issues and examples: https://paquets-90dfc06b6fea.herokuapp.com/
- The four packages: [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/), [Securite](https://securite-2db4c1994be7.herokuapp.com/), [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/), [Active](https://active-c1883a966650.herokuapp.com/)
- Also available in [French](../fr/README.md) and [Spanish](../es/README.md).
