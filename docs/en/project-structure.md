# Project structure

`paquets new my-app` creates a complete project: a server with pages, a model and its REST API, an island, tests, a production `Dockerfile`, the files for Heroku and two GitHub workflows. This page goes through every file, explains where the program looks for files at runtime, and suggests how to organise the code as it grows.

## The files

```text
my-app/
├── Cargo.toml                  the package: dependencies, two targets, build profiles
├── src/
│   ├── main.rs                 the server: pages, a model, its REST API, tests
│   └── lib.rs                  components shared with the browser, and the islands
├── public/
│   └── favicon.svg             static files, served at the root of the site
├── Dockerfile                  the production image (islands + server)
├── heroku.yml                  tells Heroku to build the Dockerfile
├── app.json                    describes the app for the "Deploy to Heroku" button
├── .github/
│   └── workflows/
│       ├── ci.yml              tests every push, then merges the branch into master
│       └── heroku.yml          deploys master to Heroku
├── README.md                   how to run, test and deploy the project
├── .gitignore                  ignores target/, pkg/ and *.db files
└── .dockerignore               keeps target/, pkg/, .git and *.db out of Docker builds
```

`paquets new` also runs `git init` with a `master` branch, unless the folder is already inside a Git repository.

These appear later, and are never committed (they are in `.gitignore`):

| Folder or file | Created by | Contains |
|---|---|---|
| `target/` | Cargo | everything Cargo compiles |
| `pkg/` | `paquets dev`, `paquets build` | the browser bundle of the islands: `my_app.js` and `my_app_bg.wasm` |
| `*.db` | SQLite | your local database, when `DATABASE_URL=sqlite://...` |

## src/main.rs: the server

The server is one file to start with. It contains, from top to bottom:

1. `use paquets::prelude::*;`, which brings everything an app needs, then `use my_app::counter;`, the island defined in `src/lib.rs`;
2. a `Todo` model (`#[model]`): a database table that also travels as JSON;
3. a `NewTodo` struct (`#[data]`): the data of the "new todo" form;
4. `home`, the handler of the home page: it reads the to-dos from the database and returns a `Page` with a header, a hero section, a card with the counter island, the list of to-dos and a form;
5. `add`, the handler of the form: it saves the to-do and redirects to `/`;
6. `app()`, which creates the `App` and declares the routes: `GET /`, `POST /todos` and the REST API at `/api/todos`;
7. `main()`, which runs `app()`;
8. a `tests` module that checks the home page and the API.

Separating `app()` from `main()` is what makes the app testable: tests call `app().test().await` and get the exact same routes, without opening a port. See [Testing](testing.md).

## src/lib.rs: shared with the browser

`src/lib.rs` is compiled twice:

- **for the server**, as a normal library that `src/main.rs` uses (`use my_app::counter;`), so the server can render the islands to HTML;
- **for the browser**, to WebAssembly, by `paquets dev` and `paquets build`, so the browser can make those islands interactive.

It contains the `counter` island and the browser entry point:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[cfg(target_arch = "wasm32")]` means "only in the browser build". Put the components and islands that the browser needs in this file (or in modules of the library), and keep server-only code (handlers, database queries) in `src/main.rs` and its modules. See [Islands](islands.md).

## Cargo.toml: one crate, two targets

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
publish = false

# One crate, two targets: the server (src/main.rs) and, compiled to WebAssembly by
# `paquets build`, the islands of src/lib.rs.
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = "0.1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { version = "0.1", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true

# The browser bundle: as small as possible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

| Section | Why |
|---|---|
| `crate-type = ["cdylib", "rlib"]` | `cdylib` produces the `.wasm` file for the browser, `rlib` lets `src/main.rs` use the library. |
| `cfg(not(target_arch = "wasm32"))` dependencies | The server side: Paquets with its default features (SQLite, PostgreSQL). |
| `cfg(target_arch = "wasm32")` dependencies | The browser side: Paquets without the server and databases, with the `client` feature, plus `wasm-bindgen`. |
| `[dev-dependencies]` | tokio, for `#[tokio::test]` in the tests. Your app itself doesn't need it. |
| `[profile.release]` | Optimisations for the production server: link-time optimisation, smaller binary. |
| `[profile.wasm-release]` | Used by `paquets build` for the islands: optimised for size. |

The `paquets` line has both a `version` and a `git` source. While you develop, Cargo uses the GitHub repository. Once Paquets is on crates.io you can simplify it to `paquets = "0.1"` (and keep `default-features = false, features = ["client"]` on the wasm32 line).

## public/: static files

Every file in `public/` is served at the root of the site: `public/favicon.svg` is `/favicon.svg`, `public/css/site.css` is `/css/site.css`. Put images, fonts, stylesheets, `robots.txt` and the like there. Files come with the right `Content-Type`, `ETag` and `Last-Modified` headers, and browsers get `304 Not Modified` when they already have the file.

> [!NOTE]
> A file in `public/` wins over a route with the same path, because static files are checked before routing. In particular, a `public/index.html` file would replace your `GET /` route.

## The deployment files

| File | Role | Read more |
|---|---|---|
| `Dockerfile` | Builds the islands and the server in release mode, then copies the program, `pkg/` and `public/` into a small Debian image that runs as a non-root user. | [Docker](deploy-docker.md) |
| `heroku.yml` | Tells Heroku to build the image from the `Dockerfile` (Heroku's "container" stack). | [Heroku](deploy-heroku-mobile.md) |
| `app.json` | Describes the app for the "Deploy to Heroku" button: name, description, container stack. | [Heroku](deploy-heroku-mobile.md#1-the-fastest-way-the-deploy-to-heroku-button) |
| `.github/workflows/ci.yml` | On every push: formatting, Clippy (server and WebAssembly), tests. When everything is green, the branch is merged into `master` automatically (except branches named `wip/...`). | [Automation](automation.md) |
| `.github/workflows/heroku.yml` | Deploys `master` to Heroku through the Heroku Platform API, configured from your phone with a secret and a variable. | [Heroku](deploy-heroku-mobile.md#4-automatic-deployments-with-github-actions) |

## The minimal project

`paquets new my-app --minimal` creates the same project without islands:

- no `src/lib.rs`: the whole app is in `src/main.rs`;
- a plain `Cargo.toml` with `paquets` under `[dependencies]` and no WebAssembly profile;
- a `Dockerfile` that only builds the server.

It needs no WebAssembly tool. You can add islands later: see [Islands](islands.md#setting-up-a-project-by-hand).

## Where the program looks for files

When it starts, the server looks for two folders **in the current working directory** (the folder you launch it from, which is the project root with `paquets dev` and `cargo run`):

| Folder | If it exists |
|---|---|
| `public/` | served at the root of the site |
| `pkg/` | served at `/pkg/`; if it contains a bundle (`<name>.js` and `<name>_bg.wasm`), pages with islands load it |

The SQLite file of `DATABASE_URL=sqlite://app.db` is also relative to the working directory. In the Docker image, the working directory is `/app`, where the `Dockerfile` copies `public/` and `pkg/`. If you run the release binary by hand, launch it from the project root (or copy `public/` and `pkg/` next to where you launch it).

## Growing the project

One file is perfect to start. When `src/main.rs` gets long, split it into modules. A common layout:

```text
src/
├── main.rs        mod declarations, fn app() with the routes, fn main()
├── models.rs      the #[model] and #[data] structs
├── pages.rs       the handlers that return pages, and the shared layout
├── api.rs         custom JSON routes, or a Router mounted at /api
└── lib.rs         the islands (browser)
```

In `src/main.rs`, declare the modules and keep the routes in one place:

```rust
mod models;
mod pages;

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/", pages::home);
    app.api::<models::Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Each module starts with `use paquets::prelude::*;`. Items used by other modules need `pub`:

```rust
// src/models.rs
use paquets::prelude::*;

#[model]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

```rust
// src/pages.rs
use paquets::prelude::*;

use crate::models::Todo;

/// The layout shared by every page.
pub fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .body(header().child(a().href("/").text("📦 My app")))
        .body(main().child(content))
        .body(footer().text("Made with Paquets"))
}

pub async fn home(req: Request) -> Result<Page> {
    let todos = Todo::all(req.db()).await?;
    Ok(layout(
        "Home",
        ul().children(todos.into_iter().map(|todo| li().text(todo.title))),
    ))
}
```

In `pages.rs` there is no `fn main`, so the `<main>` tag is simply `main()`.

## Next steps

- [Routing](routing.md): routes, parameters, routers and middleware.
- [Pages](pages.md): views, layouts, the theme and SEO.
- [Command line tool](cli.md): what `paquets dev` and `paquets build` do.
