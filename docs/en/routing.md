# Routing

Routing decides which code answers a request, based on its method (`GET`, `POST`…) and its path (`/users/42`). Paquets uses the router of [vitesse](https://maxlestage.github.io/Vitesse/), with the vocabulary of Express.js: `app.get(path, handler)`, parameters written `:id`, middleware with `next`. This page covers routes, parameters, routers, middleware, the 404 page and static files.

## Your first routes

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async { "Home" });
    app.get("/about", |_| async { Page::new("About").body(h1().text("About us")) });
    app.post("/contact", |_| async { (201, "Message received") });

    app.run()
}
```

Each call takes a **path** and a **handler**. A handler is an `async` function or closure that receives the request and returns something that becomes a response: text, JSON, a page, a status, a redirect… (see [Requests and responses](requests-responses.md)).

## Three ways to write a handler

```rust
// 1. A closure that doesn't need the request: `|_|`.
app.get("/", |_| async { "Home" });

// 2. A closure that uses the request: give its type, and write `async move`.
app.get("/hello/:name", |req: Request| async move {
    format!("Hello {}!", req.param("name").unwrap_or("you"))
});

// 3. A named function, best as soon as the code is more than a few lines.
async fn list_users(_req: Request) -> Json<Vec<&'static str>> {
    Json(vec!["ada", "grace"])
}
app.get("/users", list_users);
```

`async move` lets the closure's future own the request, which is what allows it to read the request after an `.await`. Forgetting `move` gives a borrow error from the compiler.

## HTTP methods

| Method | Matches |
|---|---|
| `app.get(path, handler)` | `GET`, and `HEAD` too |
| `app.post(path, handler)` | `POST` |
| `app.put(path, handler)` | `PUT` |
| `app.patch(path, handler)` | `PATCH` |
| `app.delete(path, handler)` | `DELETE` |
| `app.head(path, handler)` | `HEAD` |
| `app.options(path, handler)` | `OPTIONS` |
| `app.all(path, handler)` | every method |
| `app.route(method, path, handler)` | the `Method` you pass, including non-standard ones |
| `app.ws(path, handler)` | a WebSocket connection (see [WebSocket](websocket.md)) |
| `app.api::<Model>(path)` | six REST routes for a model (see [REST API](rest-api.md)) |

The routing methods return `&mut` the app, so you can chain them:

```rust
app.get("/articles", list_articles)
    .post("/articles", create_article)
    .get("/articles/:id", show_article)
    .delete("/articles/:id", delete_article);
```

## Path parameters

A segment that starts with `:` captures that part of the path. `req.param(name)` returns it as an `Option<&str>`:

```rust
app.get("/users/:id/posts/:post", |req: Request| async move {
    let user = req.param("id").unwrap_or_default();
    let post = req.param("post").unwrap_or_default();
    format!("post {post} of user {user}")
});
```

`GET /users/42/posts/7` answers `post 7 of user 42`. Values are percent-decoded (`/users/Fran%C3%A7ois` gives `François`), and a parameter always covers a whole segment: it never matches an empty segment or a `/`.

### Typed parameters

`req.param_as::<T>(name)` converts a parameter to a number, a `bool`, or any type that implements `FromStr`. When the conversion fails, it returns an error that becomes a `400 Bad Request`, so `?` is all you need:

```rust
async fn show_user(req: Request) -> Result<String> {
    let id: i64 = req.param_as("id")?; // 400 if `id` is not a number
    Ok(format!("user #{id}"))
}

app.get("/users/:id", show_user);
```

```text
GET /users/42   → 200 user #42
GET /users/abc  → 400 {"error":"invalid parameter 'id': 'abc'"}
```

## Wildcards

A `*name` segment captures the rest of the path, slashes included. It must be the last segment:

```rust
app.get("/files/*path", |req: Request| async move {
    format!("you asked for {}", req.param("path").unwrap_or_default())
});
```

`/files/2024/report.pdf` gives `path = "2024/report.pdf"`, and `/files` gives an empty string.

## How a route is chosen

- **The most specific route wins**, whatever the order of declaration: a static segment beats a parameter, which beats a wildcard. With `/users/new` and `/users/:id`, a request to `/users/new` goes to the first one.
- **A trailing slash is ignored**: `/about/` matches `/about`.
- **Paths are case-sensitive**: `/About` doesn't match `/about`.
- **The query string is ignored**: `/search?q=rust` matches `/search` (read it with `req.query("q")`, see [Requests and responses](requests-responses.md#query-strings)).

Mistakes in route definitions are caught **when the app starts**, with a clear message: a path that doesn't start with `/`, a wildcard that isn't last, or the same method declared twice on the same path. For example, `app.api::<Todo>("/api/todos")` followed by your own `app.get("/api/todos", ...)` panics with `duplicate route: GET /api/todos is already defined`.

Some HTTP behaviour comes for free:

- a `HEAD` request uses the `GET` route and gets the headers without the body;
- a path that exists, but not for this method, gets `405 Method Not Allowed` with an `Allow` header listing the methods that work;
- an `OPTIONS` request gets `204 No Content` with the same `Allow` header.

## Routers: grouping routes

A `Router` groups routes under a prefix, like `express.Router()`:

```rust
let mut admin = Router::new();
admin.get("/", |_| async { "Dashboard" });
admin.get("/users", |req: Request| async move {
    Ok::<_, Error>(Json(User::all(req.db()).await?))
});

app.mount("/admin", admin); // GET /admin and GET /admin/users
```

Handlers in a router have the same request as anywhere else: `req.db()`, `req.state()` and the security shield all work. A router can have its own middleware (see below), which makes it the natural way to protect a section of the site.

## Middleware

A middleware runs around the handlers. It receives the request and `next`, the rest of the chain, and returns a response. It can act before the handler, after it, or answer by itself:

```rust
use std::time::Instant;

app.middleware(|req: Request, next: Next| async move {
    let start = Instant::now();
    let res = next.run(req).await; // the rest of the chain, then the handler
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    res.header("server-timing", format!("app;dur={ms:.2}"))
});
```

To stop a request, return a response without calling `next.run`:

```rust
async fn require_admin(req: Request, next: Next) -> Response {
    if req.header("x-admin-key") != Some("let-me-in") {
        return Error::forbidden("admins only").into_response();
    }
    next.run(req).await
}
```

There are three places to put a middleware:

| Where | Code | Runs for |
|---|---|---|
| The whole app | `app.middleware(m)` | every request, including 404s, whatever the order of declaration |
| A router | `router.middleware(m)` | every request under the router's prefix |
| One route | `app.get("/stats", stats.with(m))` | that route only |

```rust
let mut admin = Router::new();
admin.middleware(require_admin);
admin.get("/", |_| async { "Dashboard" });
app.mount("/admin", admin);

// One route only. `.with` comes from the `HandlerExt` trait, in the prelude.
app.get("/stats", stats.with(require_admin));
```

To call `.with` on a closure, wrap the closure in parentheses: `(|_| async { "ok" }).with(require_admin)`.

### The order of the whole chain

For every request, Paquets runs these layers, in this order:

1. the request log (one line per request; `app.quiet()` turns it off);
2. the security shield (see [Security](security.md)): a blocked request stops here, before any of your code;
3. your global middleware, in the order you added them;
4. the static files of `public/`;
5. routing, then the router's middleware, the route's middleware (`.with`) and the handler.

The response travels back through the same layers in reverse order, so the shield adds its security headers to every response, yours included.

## Custom 404 page

When no route matches, Paquets answers `404` with `{"error":"Cannot GET /path"}`. `app.fallback` replaces that answer, for example with a page:

```rust
app.fallback(|req: Request| async move {
    Page::new("Page not found")
        .status(404)
        .body(h1().text("Page not found"))
        .body(p().text(format!("There is nothing at {}.", req.path())))
        .body(a().href("/").text("Back to the home page"))
});
```

Don't forget `.status(404)`: a `Page` is sent with `200 OK` by default. The fallback is not called for a `405` (the path exists, with another method). To change every error response at once, see [Errors](errors.md#changing-the-format-of-every-error).

## Static files

Files in the `public/` folder are served at the root of the site, with no code. To serve another folder under a prefix, use `static_dir`:

```rust
app.static_dir("/downloads", "files"); // files/report.pdf → /downloads/report.pdf
```

Paths are relative to the folder the server is launched from. Static files get the right `Content-Type`, `ETag` and `Last-Modified` headers, `304 Not Modified` and range requests (video). Hidden files (`.env`, `.git/`) and `..` paths are refused. More options in [vitesse's static files guide](https://maxlestage.github.io/Vitesse/en/docs/static-files/).

## Learn more

vitesse's documentation covers routing in more depth: [routing](https://maxlestage.github.io/Vitesse/en/docs/routing/), [middleware](https://maxlestage.github.io/Vitesse/en/docs/middleware/) (including the built-in `logger`, `cors` and `timeout`) and [routers](https://maxlestage.github.io/Vitesse/en/docs/routers/). Everything there works in Paquets: `App` gives access to every method of vitesse's app.

## Next steps

- [Requests and responses](requests-responses.md): read parameters, query strings and bodies, and build any response.
- [Pages](pages.md): return complete HTML pages.
- [Security](security.md): what the shield does before your middleware runs.
