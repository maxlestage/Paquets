# Testing

`app.test().await` turns your app into a test client: requests go straight to the app, through the shield, the middleware and your handlers, without opening a port. The database is in memory and empty for every test, so tests are fast, independent and can run in parallel. This page shows how to write them.

## Setup

Tests are `async` functions marked `#[tokio::test]`. They need tokio as a development dependency, which projects created with `paquets new` already have:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Your app itself doesn't need tokio: `App::run()` manages its own runtime.

## Make the app testable

Tests must build the same app as `main`, without starting the server. Put the app in a function:

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Home").body(h1().text("Hello")) });
    app.api::<Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

## Your first test

Add a `tests` module at the end of `src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_page() {
        let client = app().test().await;

        let res = client.get("/").await;

        assert_eq!(res.status(), 200);
        assert!(res.text().contains("<h1>Hello</h1>"));
    }
}
```

Run it:

```sh
cargo test                 # every test
cargo test home_page       # the tests whose name contains "home_page"
cargo test -- --nocapture  # also show what the tests print
```

`app().test().await` prepares the app exactly as `run()` would: it opens the database, creates the tables, runs the `on_start` hooks, and plugs in the shield and the static files. Only the request log and the network are left out.

## Sending requests

Start a request from the client, chain what you need, and send it with `.await`:

| Method | Request |
|---|---|
| `client.get(uri)` | `GET` |
| `client.post(uri)` | `POST` |
| `client.put(uri)` | `PUT` |
| `client.patch(uri)` | `PATCH` |
| `client.delete(uri)` | `DELETE` |
| `client.request(Method::OPTIONS, uri)` | any other method |

| Then | Effect |
|---|---|
| `.header(name, value)` | adds a header |
| `.json(&value)` | a JSON body, with `content-type: application/json` |
| `.form(&value)` | a form body, such as `&[("email", "ada@example.com")]` |
| `.body(data)` | a raw body |

```rust
let res = client
    .post("/api/todos")
    .json(&json!({ "title": "Write tests", "done": false }))
    .await;

let res = client.get("/search?q=caf%C3%A9").await; // the query string is part of the URI, encoded
let res = client.post("/login").form(&[("email", "ada@example.com"), ("password", "secret")]).await;
```

## Reading responses

| Method | Returns |
|---|---|
| `res.status()` | the status, comparable with a number: `assert_eq!(res.status(), 404)` |
| `res.header(name)` | one header, as `Option<&str>` |
| `res.headers()` | every header |
| `res.text()` | the body as a `String` |
| `res.json::<T>()` | the body decoded from JSON (it panics, showing the body, if it isn't valid) |
| `res.bytes()` | the raw body |

```rust
#[tokio::test]
async fn creates_a_todo() {
    let client = app().test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Write tests", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    let todo: Todo = res.json(); // typed
    assert_eq!(todo.title, "Write tests");

    let body: paquets::serde_json::Value = client.get("/api/todos/1").await.json(); // untyped
    assert_eq!(body["done"], false);
}
```

## The database in tests

Every call to `app().test()` gets a new, empty, in-memory database, whatever `DATABASE_URL` says. Tests never see each other's data, nor yours.

To start from known data, add an `on_start` hook in the test:

```rust
#[tokio::test]
async fn lists_are_paginated() {
    let mut app = app();
    app.on_start(|db| async move {
        for title in ["One", "Two", "Three"] {
            Todo { id: 0, title: title.into(), done: false }.insert(db).await?;
        }
        Ok(())
    });
    let client = app.test().await;

    let res = client.get("/api/todos?limit=2").await;
    assert_eq!(res.header("x-total-count"), Some("3"));
    assert_eq!(res.json::<Vec<Todo>>().len(), 2);
}
```

To test against a real SQL engine (for raw SQL or SQL-only migrations), choose it in the test. An in-memory SQLite database is still created fresh for each test:

```rust
let mut app = app();
app.database("sqlite::memory:");
let client = app.test().await;
```

## Testing pages

A page is HTML: check the status, the content type and the text you expect.

```rust
#[tokio::test]
async fn the_home_page_has_its_seo_tags() {
    let client = app().test().await;
    let res = client.get("/").await;

    assert_eq!(res.status(), 200);
    assert!(res.header("content-type").unwrap().starts_with("text/html"));
    let html = res.text();
    assert!(html.contains("<title>Home</title>"));
    assert!(html.contains("<h1>Hello</h1>"));
}
```

## Testing security

The shield runs in tests too, so you can check that attacks are refused:

```rust
#[tokio::test]
async fn attacks_are_blocked() {
    let client = app().test().await;

    // SQL injection in the query string: blocked by the firewall.
    let res = client.get("/?q=1%27%20OR%20%271%27=%271").await;
    assert_eq!(res.status(), 403);

    // A known attack tool: blocked by the bot blocker.
    let res = client.get("/").header("user-agent", "sqlmap/1.7").await;
    assert_eq!(res.status(), 403);

    // A request sent from another website: blocked by the CSRF guard.
    let res = client
        .post("/api/todos")
        .header("sec-fetch-site", "cross-site")
        .json(&json!({ "title": "x", "done": false }))
        .await;
    assert_eq!(res.status(), 403);
}
```

> [!WARNING]
> The rate limit runs in tests as well: all the requests of a test come from the same client, so after 100 requests in quick succession the next ones get `429 Too Many Requests`. For a test that sends many requests, remove the rate limit in that test:
>
> ```rust
> let mut app = app();
> app.security(paquets::shield().without("rate_limit").build());
> let client = app.test().await;
> ```

## Testing errors and middleware

Errors are ordinary responses: check the status and the message.

```rust
#[tokio::test]
async fn bad_input_is_refused() {
    let client = app().test().await;

    let res = client.post("/api/todos").body("{oops").await; // not JSON
    assert_eq!(res.status(), 400);

    assert_eq!(client.get("/api/todos/abc").await.status(), 400); // not a number
    assert_eq!(client.get("/api/todos/999").await.status(), 404); // no such row
}
```

For a middleware, check both paths: the request it lets through and the one it stops (for example, with and without the `Authorization` header).

## Tests in the tests folder

The `tests` module of `src/main.rs` is the simplest place for tests. If you prefer integration tests in a `tests/` folder, they can only use your library (`src/lib.rs`), not `src/main.rs`. Move `app()` to the library, and keep it out of the browser build when the project has islands:

```rust
// src/lib.rs
#[cfg(not(target_arch = "wasm32"))]
pub fn app() -> paquets::App {
    let mut app = paquets::App::new();
    // ... routes ...
    app
}
```

```rust
// src/main.rs
fn main() -> std::io::Result<()> {
    my_app::app().run()
}
```

```rust
// tests/api.rs
use paquets::prelude::*;

#[tokio::test]
async fn home_page() {
    let client = my_app::app().test().await;
    assert_eq!(client.get("/").await.status(), 200);
}
```

The models used by `app()` then move to the library too, behind the same `#[cfg(not(target_arch = "wasm32"))]`.

## What the test client doesn't cover

- **The network**: headers written by the HTTP engine (`content-length`, `date`) are not in test responses.
- **WebSocket conversations**: the test client only sees the handshake. See [WebSocket](websocket.md#testing) to test on a real port.
- **The browser side of islands**: test the server HTML of an island with the test client, and its logic by calling your Rust functions directly.

## In continuous integration

The `ci.yml` workflow of a generated project runs `cargo test` on every push, together with `cargo fmt --check` and Clippy, and merges the branch into `master` only when everything is green. See [Automation](automation.md).

## Next steps

- [Errors](errors.md): what each error looks like, to test it.
- [Automation](automation.md): tests on every push.
- [Vitesse's testing guide](https://vitesse-f16135654c3d.herokuapp.com/en/docs/testing/): more examples with the same test client.
