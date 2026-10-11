# Performance

Paquets is fast without tuning: its server, Vitesse, is faster than actix-web and axum on the same machine, its security shield costs a few microseconds per request, and pages ship no code to the browser unless they contain islands. This page gives the numbers, explains how the server uses your CPU, and lists what keeps your own app fast.

## The numbers

### The server (Vitesse)

Requests per second on a 4 vCPU VM, the server pinned to 2 cores and the load generator (wrk) to the other 2, 128 keep-alive connections. In brackets: the server's CPU time per request (lower is better).

| Scenario | axum 0.8 | actix-web 4 | **Vitesse** |
|---|---:|---:|---:|
| `GET /` (text) | 168,165 (11.7 µs) | 274,195 (7.1 µs) | **291,415 (6.1 µs)** |
| `GET /json` | 165,988 (11.9 µs) | 248,987 (7.9 µs) | **313,438 (5.9 µs)** |
| `GET /json` sent by a browser (12 headers) | 131,710 (15.0 µs) | 179,505 (10.9 µs) | **290,478 (6.8 µs)** |
| `GET /users/:id` (parameter + JSON) | 151,303 (13.1 µs) | 209,173 (9.4 µs) | **304,323 (6.2 µs)** |
| `POST /echo` (reads and returns JSON) | 105,570 (18.8 µs) | 170,129 (11.7 µs) | **253,941 (7.6 µs)** |
| `GET /` pipelined ×16 | 213,678 (9.3 µs) | 1,272,510 (1.6 µs) | **2,781,541 (0.64 µs)** |

The methodology, the other frameworks measured (Express, Drogon) and the way to reproduce these numbers are in [Vitesse's performance guide](https://maxlestage.github.io/Vitesse/en/docs/performance/).

### The shield (Securite)

The whole recommended pipeline (request id, hygiene, method filter, body limit, rate limit, bot blocker, firewall, CSRF, security headers, cookie hardening) costs about **6 µs** per request.

### Pages and islands (Active)

| | Server rendering of a product page | WebAssembly + loader for a counter (compressed) | Interactive at 100 ms of latency |
|---|---:|---:|---:|
| **active** | **5.6 µs** | **36.8 kB** | **224 ms** |
| Leptos 0.8 + leptos_meta | 13.4 µs | 60.9 kB | 227 ms |
| Sycamore 0.9 | 33.0 µs | 45.8 kB | 227 ms |
| Dioxus 0.7 | 24.2 µs | 170.4 kB | 339 ms |

The smallest bundle of all, and server rendering 2.4 times faster than the other framework that merges SEO tags from components (Leptos with leptos_meta). A template engine without SEO merging, such as maud, renders faster (1.4 µs), but sends nothing to the browser. Details in [Active's benchmarks](https://maxlestage.github.io/Active/en/docs/benchmarks/).

### What this means for your app

These benchmarks measure each package alone, on trivial handlers. In a real Paquets app, a request also goes through the shield and the request log, and most handlers query a database. In practice, **the database and your own code are where the time goes**: the framework's share is a few microseconds. The tips below therefore focus on them.

## How the server uses your CPU

- With the in-memory database or SQLite, `app.run()` starts **one thread per CPU core**, each with its own event loop and its own listening socket. The kernel spreads connections across threads, and a request never changes thread. This is Vitesse's fastest mode.
- With a database server (PostgreSQL…), the app runs on tokio's multi-threaded runtime, shared with the database connections, and idle threads take work from busy ones.

In both modes, a handler that blocks its thread (a long computation, a blocking file read, `std::thread::sleep`) delays every other request of that thread. Keep handlers asynchronous, and move heavy computations off the event loop:

```rust
use paquets::vitesse::tokio::task;

async fn report(_req: Request) -> Result<String> {
    // A long computation: run it on a thread made for blocking work.
    let total = task::spawn_blocking(|| (1..=50_000_000u64).sum::<u64>()).await?;
    Ok(format!("total: {total}"))
}
```

Password hashing, image processing and large compressions belong in `spawn_blocking`.

## Tips

### Build in release mode

A debug build (`cargo run`, `paquets dev`) can be ten times slower or more. Measure and deploy release builds only: `paquets build`, or `cargo build --release`. The `Dockerfile` of a generated project builds in release mode, with link-time optimisation.

### Database

- **Add indexes** on the columns you filter or sort by: `#[memoire(index)]` (unique columns are indexed already).
- **Read only what you show**: `.limit(...)` or `.paginate(...)` on lists, `.columns(...)` when you need one or two columns, `.count(...)` or `.exists(...)` instead of loading rows to count them.
- **Avoid one query per row** (the "N+1" problem). Load related rows in one query with `is_in`:

```rust
async fn posts_with_authors(req: Request) -> Result<Json<Vec<(String, String)>>> {
    let db = req.db();
    let posts = Post::query().order_by(Post::ID.desc()).limit(20).all(db).await?;
    // One query for every author, instead of one per post.
    let ids: Vec<i64> = posts.iter().map(|p| p.author_id).collect();
    let authors = Author::query().filter(Author::ID.is_in(ids)).all(db).await?;
    let rows = posts
        .into_iter()
        .map(|post| {
            let name = authors.iter().find(|a| a.id == post.author_id).map_or("?", |a| a.name.as_str());
            (post.title, name.to_string())
        })
        .collect();
    Ok(Json(rows))
}
```

- **Group writes** in a transaction, or use `insert_many`, `update_many` and `delete_many`.
- Memoire builds the SQL of primary-key operations (`get`, `insert`, `update`, `delete`) at compile time: they cost no SQL generation at run time.

### Pages

- A page without islands sends no JavaScript at all. Add islands only where the page must react in the browser, and keep their props small: they are part of the HTML.
- The built-in theme is about 3.5 kB, inline in each page. For a large stylesheet shared by every page, put it in `public/` and link it with `.stylesheet(...)`: browsers cache it.

### Static files

Files of `public/` are sent with `ETag` and `Last-Modified`, and browsers revalidate them (`304 Not Modified`). For files whose name changes with their content (`app.3f9c2b.css`), allow a long cache with a dedicated folder:

```rust
use std::time::Duration;
use paquets::vitesse::ServeDir;

app.serve_dir("/assets", ServeDir::new("assets").max_age(Duration::from_secs(31_536_000)));
```

### Compression

Text compresses very well (HTML, JSON, CSS, WebAssembly), and Heroku's router doesn't compress. With the `compress` feature, the app does it itself:

```rust
app.middleware(middleware::compress());
```

Responses of 1 KiB or more that are text, JSON, SVG or WebAssembly are sent in Brotli to the browsers that accept it, in gzip to the others; images and archives pass through. The encoding runs on a blocking thread. A CDN (Cloudflare, Fastly…) in front of the app does the same and caches the result.

### Logs

One log line per request costs little, but writing to the terminal is not free under heavy load. `app.quiet()` turns the request log off, for benchmarks or when your platform already logs requests.

## Measuring

Build in release mode, then load the app with a tool such as [oha](https://crates.io/crates/oha) or wrk:

```sh
paquets build
./target/release/my-app &
oha -z 10s -c 64 http://127.0.0.1:3000/
```

> [!IMPORTANT]
> All the requests of a load test come from one IP address, so the shield's rate limit (20 requests per second per IP, bursts of 100) answers `429` to most of them, and you measure the rate limiter. For a benchmark on your own machine, remove the rate limit and the request log:
>
> ```rust
> app.security(paquets::shield().without("rate_limit").build());
> app.quiet();
> ```
>
> Never deploy that configuration: the rate limit is what protects your app from floods.

Measure what your users do: the pages and API routes that matter, with realistic data in the database.

## Next steps

- [Configuration](configuration.md): workers, threads and the runtime.
- [Database](database.md): queries, indexes and transactions.
- [Deploy with Docker](deploy-docker.md): an optimised production image.
