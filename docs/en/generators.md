# Generators

`paquets make:*` writes the code you would write by hand — a model, its REST API, a page, a migration, a test, authentication, an admin panel — and wires it into `src/main.rs`, like Laravel's Artisan or Rails' generators. The generated code is ordinary, idiomatic Rust: read it, change it, it is yours.

## The generators

| Command | Writes | Wires into `fn app()` |
|---|---|---|
| `paquets make:model Post title:string body:text` | `src/models/post.rs`, `src/models.rs` | `app.model::<models::Post>();` |
| `paquets make:api Post title:string` | the model, with a test of its API | `app.api::<models::Post>("/api/posts");` |
| `paquets make:page about` | `src/pages/about.rs` (with a test), `src/pages.rs` | `app.get("/about", pages::about::about);` |
| `paquets make:migration add_slug` | `src/migrations/m<timestamp>_add_slug.rs`, `src/migrations.rs` | `app.migrate(migrations::all());` |
| `paquets make:test checkout` | `src/feature_tests/checkout.rs` | the module, compiled by `cargo test` only |
| `paquets make:auth` | `Cargo.toml` (Argon2 optimized in debug builds) | `app.auth(paquets::auth::Auth::new());` |
| `paquets make:admin Post Comment` | — | `app.admin(Admin::new().model::<models::Post>()…);` |

`make:model` also takes `--api` (same as `make:api`) and `--admin` (adds the model to the [admin panel](admin.md)). Every generator declares the modules it needs (`mod models;`…) and runs `rustfmt` on the result, so a project made by `paquets new` still passes `cargo fmt --check` and `cargo clippy -- -D warnings`: CI checks exactly that.

Run them in the project folder. They never overwrite a file: generating the same model twice is an error.

## Models and fields

```sh
paquets make:api Post title:string body:text published:bool "views:i64?" slug:string:unique --admin
```

writes `src/models/post.rs`:

```rust
//! The `Post` model: a table of the database (`posts`), also sent and
//! received as JSON.

use paquets::prelude::*;

/// Post.
#[model(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub views: Option<i64>,
    #[memoire(unique)]
    pub slug: String,
}
```

and a test that creates a row through `POST /api/posts` and lists it. Fields are `name:type`, with an `id: i64` key added first:

| Type | Rust |
|---|---|
| `string`, `text` | `String` |
| `i64`, `int`, `integer` | `i64` |
| `i32` | `i32` |
| `f64`, `float`, `number` | `f64` |
| `bool`, `boolean` | `bool` |

A `?` after the type makes the field optional (`Option<T>`, quote it in your shell: `"views:i64?"`), and `:unique` adds a unique constraint. Names are written in `snake_case` (`BlogPost` and `blog-post` both give `blog_post`), the type in `PascalCase`, the table and the API path in the plural (`posts`, `categories`).

## Pages, migrations and tests

`paquets make:page pricing` writes a page rendered on the server, routed at `/pricing`, with a test that requests it:

```rust
/// `GET /pricing`, rendered on the server with its SEO tags.
pub async fn pricing(_req: Request) -> Page {
    Page::new("Pricing")
        .description("Pricing.")
        .body(
            html::main()
                .class("stack")
                .child(h1().text("Pricing"))
                .child(p().text("Edit src/pages/pricing.rs.")),
        )
}
```

`paquets make:migration add_slug_to_posts` adds a versioned migration (its version is the UTC time) to the list of `src/migrations.rs`; describe its changes with memoire's `Migration` (`.create_table::<M>()`, `.sql(…)`, `.sql_for(dialect, …)`). See [Operations](operations.md#migrations-and-seeds).

`paquets make:test checkout` writes a feature test that goes through the whole app, in memory:

```rust
#[tokio::test]
async fn checkout() {
    let client = crate::app().test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
}
```

Tests and pages use the `fn app() -> App` that `paquets new` writes in `src/main.rs`; without it, the generators tell you which line to add yourself.

## Authentication and admin

`paquets make:auth` turns on [authentication](authentication.md): sign-up, login, password resets and two-factor codes. `paquets make:admin Post Comment` adds models to the [admin panel](admin.md) (and creates it if needed).

## Next steps

- [Authentication](authentication.md): the pages and the protection of routes.
- [Admin panel](admin.md): managing your models from the browser.
- [Operations](operations.md): API documentation, the route list, migrations, health and metrics, live reload.
