# Introduction

Paquets 📦 is **the simplest full-stack framework for Rust**. One crate, one import and one `App` give you a fast web server, security in front of every request, a database, REST APIs, web pages with complete SEO tags, and interactive parts that run in the browser as WebAssembly. This page explains what Paquets is, what it does for you, and what it doesn't do.

## What is Paquets?

Paquets (French for "packages") is a Rust library for building websites and web APIs. It assembles four packages written by the same author, each of which can also be used on its own:

| Package | What it does in Paquets |
|---|---|
| [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/) | The HTTP server, with an API in the style of Express.js: `app.get(...)`, `req.param(...)`, middleware, routers, WebSocket. |
| [Securite](https://securite-2db4c1994be7.herokuapp.com/) | The security shield in front of every request: firewall, rate limit, CSRF protection, security headers and more. |
| [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/) | The database: type-safe models and queries, in memory, SQLite, PostgreSQL, MySQL, Redis or MongoDB. |
| [Active](https://active-c1883a966650.herokuapp.com/) | The front end: views written as Rust functions, server rendering with SEO tags, and WebAssembly islands. |

You don't need to learn four libraries to start. Paquets connects them, chooses good defaults, and exposes one small API. When you need more, every package is one import away (see [The four packages](packages.md)).

## A complete application

Here is a complete program. It serves a home page, a greeting route and a REST API backed by a database:

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async {
        Page::new("Home")
            .description("My site, in Rust.")
            .body(html::main().child(h1().text("Hello 👋")))
    });

    app.get("/hello/:name", |req: Request| async move {
        format!("Hello {}!", req.param("name").unwrap_or("you"))
    });

    app.api::<Todo>("/api/todos");

    app.run()
}
```

Run it and you get:

- `http://localhost:3000/`: a styled HTML page with its title, description, Open Graph and Twitter tags, in light or dark mode;
- `http://localhost:3000/hello/Ada`: the text `Hello Ada!`;
- `http://localhost:3000/api/todos`: a JSON API to list, read, create, replace, update and delete to-dos (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`), stored in a database table that was created at startup.

> [!NOTE]
> In a file that contains `fn main`, the `<main>` HTML tag is written `html::main()`, because the name `main` already belongs to your program's function. Everywhere else, `main()` works. See [Pages](pages.md#the-main-tag-in-mainrs).

## What you get without configuring anything

`App::new()` is ready for production. Nothing in the table below needs a single line of configuration:

| Feature | Default behaviour | Change it with |
|---|---|---|
| Server | Port from the `PORT` variable (Heroku), `3000` otherwise. `Ctrl+C` or `SIGTERM` stops it gracefully. | `app.port(8080)` |
| Security | Securite's recommended shield on every request: request ids, protocol checks, 2 MiB body limit, rate limit of 20 requests per second per IP, bot blocker, firewall, CSRF protection, security headers, hardened cookies. | `app.security(...)` |
| Database | From the `DATABASE_URL` variable; in memory when it is not set. Tables of your models are created at startup. | `app.database(...)` |
| Pages | Server-rendered HTML with SEO tags, a built-in theme and a Content-Security-Policy that allows exactly what the page contains. | `Page` methods |
| Static files | The `public/` folder is served at the root of the site. | `app.static_dir(...)` |
| Islands | The browser bundle in `pkg/` (built by `paquets build`) is found at startup and loaded only by pages that contain islands. | — |
| Logs | One line per request: `GET /hello/Ada 200 0.339 ms`. | `app.quiet()` |

## Why Paquets?

- **Simplicity.** One dependency, one import (`use paquets::prelude::*;`), one `App`. A handler is an `async` function that takes a request and returns a response. Errors are values, and `?` turns them into the right HTTP status.
- **Secure by default.** Attacks such as SQL injection in the URL, cross-site request forgery, clickjacking or brute force are blocked before your code runs. You have to opt out, not opt in.
- **Fast.** Vitesse handles about 291,000 requests per second for `GET /` on 2 CPU cores, more than actix-web and axum on the same machine, and 2.78 million per second with pipelining. Securite's whole pipeline costs about 6 µs per request. The smallest island bundle (a counter) weighs 36.8 kB compressed. See [Performance](performance.md).
- **Type-safe from the database to the page.** A misspelled column, a comparison between a number and a string, or a missing field in a response is a compile error, not a bug in production.
- **Easy to deploy.** A project created with `paquets new` contains a `Dockerfile`, the files for a one-tap Heroku deployment and GitHub workflows. You can deploy, redeploy and read your logs [from a phone](deploy-heroku-mobile.md).

## Who is it for?

- **Beginners in Rust** who want to build a real website. Most handlers are a few lines long, and this documentation explains the Rust-specific parts as they come up.
- **Web developers** coming from Express, Rails, Laravel or Django who want one framework that covers the whole stack, without assembling a dozen crates.
- **Rust developers** who want a small, explicit framework with secure defaults, and who like to read the code they depend on.

## What Paquets doesn't do (yet)

Paquets is young (version 0.1). Be aware of these limits before you choose it:

- **HTTPS, HTTP/2 and HTTP/3 are options.** On a platform such as Heroku, its router provides HTTPS in front of the app. To serve it from the app itself, turn on the `tls` feature (and `http2`, `http3`): see [Configuration](configuration.md#https-in-the-server).
- **No automatic schema changes.** Tables are created when they are missing, but adding a field to a model does not change an existing table: write a migration (see [Database](database.md#changing-a-table-later)).
- **Islands need two extra tools**: the `wasm32-unknown-unknown` target and `wasm-bindgen-cli`. `paquets dev` installs them for you the first time. Pages without islands need nothing.
- **No built-in user accounts.** The shield protects every request and Vitesse keeps sessions (feature `session`), but storing users and checking their passwords is up to you. See [Security](security.md#accounts-and-sessions).
- **Not on crates.io yet.** Until the four packages and Paquets are published, projects depend on the GitHub repository. See [Installation](installation.md) and [Publish on crates.io](publish-crates-io.md).

## How this documentation is organised

- **Getting started**: [Installation](installation.md), a [Quick start](quick-start.md) tutorial, and the [Project structure](project-structure.md).
- **Essentials**: [Routing](routing.md), [Requests and responses](requests-responses.md), [Pages](pages.md), [Database](database.md), [REST API](rest-api.md) and [Security](security.md).
- **Going further**: [Islands](islands.md), [WebSocket](websocket.md), [Testing](testing.md), [Errors](errors.md), [Configuration](configuration.md) and [Performance](performance.md).
- **Deploy**: [Heroku from your phone](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Publish on crates.io](publish-crates-io.md) and [Automation](automation.md).
- **Reference**: the [Command line tool](cli.md), the [Cheat sheet](cheatsheet.md), [The four packages](packages.md) and the [FAQ](faq.md).

## Next steps

- [Installation](installation.md): install Rust and the `paquets` command.
- [Quick start](quick-start.md): build your first app in ten minutes.
