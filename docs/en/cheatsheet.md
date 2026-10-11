# Cheat sheet

The whole of Paquets on one page. Every example assumes `use paquets::prelude::*;`. Follow the links for details.

## Command line

| Task | Command |
|---|---|
| Install the tool | `cargo install --git https://github.com/maxlestage/Paquets paquets --locked` |
| Create a project | `paquets new my-app` (`--minimal`: no islands) |
| Run while developing | `paquets dev` |
| Build for production | `paquets build` |
| Check the tools | `paquets doctor` |
| Run the tests | `cargo test` |

See [Command line tool](cli.md).

## The app

| Task | Code |
|---|---|
| Create | `let mut app = App::new();` |
| Run (`PORT` or 3000) | `app.run()` from `fn main() -> std::io::Result<()>` |
| Run inside `#[tokio::main]` | `app.serve().await` |
| Choose the port | `app.port(8080);` |
| Turn the request log off | `app.quiet();` |
| Installable app, offline (PWA) | `app.pwa(Pwa::new("My app").theme_color("#6d4aff"));` |
| Choose the database | `app.database("sqlite://app.db");` or `app.database(Db::memory());` |
| Change the shield | `app.security(paquets::shield().guard(...).build());` |
| Turn the shield off | `app.security(None);` |
| Code at startup | `app.on_start(\|db\| async move { ...; Ok(()) });` |
| Shared state | `app.state(value);` then `req.state::<T>()` |
| Read limit for bodies | `app.body_limit(10 * 1024 * 1024);` |
| Test client | `let client = app.test().await;` |

See [Configuration](configuration.md).

## Routes

| Task | Code |
|---|---|
| Methods | `app.get`, `.post`, `.put`, `.patch`, `.delete`, `.all`, `.head`, `.options` |
| Closure handler | `app.get("/", \|_\| async { "Home" });` |
| Closure using the request | `app.get("/hi/:name", \|req: Request\| async move { ... });` |
| Function handler | `async fn home(req: Request) -> Result<Page> { ... }` then `app.get("/", home);` |
| Parameter | `/users/:id` → `req.param("id")` (`Option<&str>`) |
| Typed parameter | `let id: i64 = req.param_as("id")?;` (400 if invalid) |
| Wildcard | `/files/*path` → `req.param("path")` |
| Router | `let mut api = Router::new(); api.get(...); app.mount("/api", api);` |
| Global middleware | `app.middleware(\|req: Request, next: Next\| async move { next.run(req).await });` |
| Middleware on one route | `app.get("/admin", handler.with(auth));` |
| Middleware on a router | `router.middleware(auth);` |
| Custom 404 | `app.fallback(\|_\| async { Page::new("Not found").status(404) });` |
| Static folder | `public/` at `/` automatically; `app.static_dir("/files", "files");` |
| WebSocket | `app.ws("/chat", \|req, mut socket\| async move { ... });` |

See [Routing](routing.md).

## Requests

| Read | Code |
|---|---|
| Method, path | `req.method()`, `req.path()` |
| Query value | `req.query("q")` (`Option<Cow<str>>`) |
| Whole query string | `let s: Search = req.query_as()?;` |
| Header, cookie | `req.header("user-agent")`, `req.cookie("session")` |
| JSON body | `let body: NewTodo = req.json().await?;` |
| Form body | `let form: Login = req.form().await?;` |
| Text, bytes | `req.text().await?`, `req.bytes().await?` |
| Client IP (behind proxies) | `req.extensions().get::<paquets::securite::ClientIp>()` |
| Database | `req.db()` |
| State | `req.state::<Settings>()` |

See [Requests and responses](requests-responses.md).

## Responses

| Return | Gives |
|---|---|
| `"text"`, `String` | `200`, plain text |
| `Json(value)`, `json!({...})` | `200`, JSON |
| `Page::new("Title").body(...)` | `200`, an HTML page |
| `StatusCode::NO_CONTENT` | that status |
| `(201, Json(value))` | a status and a body |
| `Redirect::to("/")`, `Redirect::see_other("/")`, `Redirect::permanent("/")` | `302`, `303`, `301` |
| `Result<T>` | `T`, or the error's status |
| `Option<T>` | `T`, or `404` |
| `res::status(202).header("x-a", "1").cookie(Cookie::new("k", "v")).json(...)` | a custom response |
| `res::file("report.pdf").await` | a file |

## Pages and views

| Task | Code |
|---|---|
| Page | `Page::new("Title").description("…").body(view)` |
| Language | `.lang("fr")` |
| SEO tags | `.seo(Seo::new().canonical("…").image("…"))` |
| CSS | `.style("h1{color:red}")`, `.stylesheet("/site.css")` |
| Extra head tag | `.head(r#"<link rel="icon" href="/favicon.svg"/>"#)` |
| No theme | `.plain()` |
| Status | `.status(404)` |
| Element | `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`… |
| `<main>` in `main.rs` | `html::main()` |
| Text (escaped) | `.text("…")` |
| Attributes | `.class("card")`, `.id("x")`, `.href("/")`, `.attr("type", "email")` |
| Children | `.child(view)`, `.children(iter.map(...))` |
| Condition | `.child(flag.then(\|\| p().text("…")))` |
| Island | `island("counter", "0", counter)` |

Theme classes: `.hero`, `.card`, `.grid`, `.row`, `.stack`, `.center`, `.muted`, `.button`, `.ghost`. Theme variables: `--accent`, `--bg`, `--fg`, `--muted`, `--line`, `--card`, `--radius`, `--font`. See [Pages](pages.md).

## Models

| Task | Code |
|---|---|
| Table + JSON | `#[model] struct Todo { id: i64, title: String, done: bool }` |
| Table name | `#[model(table = "todos")]` (default: `todo`) |
| JSON only | `#[data] struct Login { email: String, password: String }` |
| Unique, index | `#[memoire(unique)]`, `#[memoire(index)]` |
| Nullable | `bio: Option<String>` |
| Optional in JSON | `#[serde(default)] done: bool` |
| Create the table only | `app.model::<Todo>();` |

## Queries

| Task | Code |
|---|---|
| All, one, maybe one | `Todo::all(db)`, `Todo::get(db, id)` (404), `Todo::find(db, id)` (`Option`) |
| Count | `Todo::count(db)` |
| Insert | `let mut t = Todo { id: 0, ... }; t.insert(db).await?;` (id written back) |
| Update, delete | `t.update(db)`, `t.delete(db)`, `Todo::delete_by_key(db, id)` |
| Filter | `Todo::query().filter(Todo::DONE.eq(false)).all(db)` |
| Combine | `a & b`, `a \| b`, `!a` |
| Compare | `.eq`, `.ne`, `.gt`, `.ge`, `.lt`, `.le`, `.is_in([..])`, `.between(a, b)`, `.is_null()` |
| Text | `.contains("x")`, `.icontains("x")`, `.starts_with("x")`, `.like("%x")` |
| Order, limit | `.order_by(Todo::ID.desc()).limit(20).offset(40)` |
| Page | `.paginate(db, page, 20)` → `items`, `total` |
| First, exists | `.first(db)`, `.exists(db)` |
| Update many | `Todo::update_many().set(Todo::DONE, true).filter(...).execute(db)` |
| Delete many | `Todo::delete_many().filter(...).execute(db)` |
| Transaction | `let tx = db.begin().await?; ...(&tx)...; tx.commit().await?;` |
| Raw SQL | `db.raw::<(i64, String)>("SELECT …", &paquets::orm::params![…])` |

`db` is `req.db()` in a handler, or the argument of `on_start`. See [Database](database.md).

## Databases

| `DATABASE_URL` | Database |
|---|---|
| not set, `memory` | in memory |
| `sqlite://app.db` | SQLite file |
| `postgres://…` | PostgreSQL (Heroku Postgres) |
| `mysql://…`, `mariadb://…` | MySQL, MariaDB (feature `mysql`) |
| `redis://…` | Redis (feature `redis`) |
| `mongodb://…` | MongoDB (feature `mongodb`) |

## REST API

`app.api::<Todo>("/api/todos");`

| Request | Answer |
|---|---|
| `GET /api/todos?limit=100&offset=0` | `200`, array, `X-Total-Count` |
| `GET /api/todos/:id` | `200` or `404` |
| `POST /api/todos` | `201` with the generated `id` |
| `PUT /api/todos/:id` | `200`, whole row replaced |
| `PATCH /api/todos/:id` | `200`, given fields changed |
| `DELETE /api/todos/:id` | `204` or `404` |

See [REST API](rest-api.md).

## Errors

| Task | Code |
|---|---|
| Return type | `Result<T>` (= `Result<T, paquets::Error>`) |
| Propagate | `?` |
| Create | `Error::bad_request(…)`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `unprocessable`, `internal`, `Error::new(418, "…")` |
| In a closure | `Ok::<_, Error>(value)` |
| From `None` | `.ok_or_else(\|\| Error::not_found("…"))?` |
| Custom format | `app.on_error(\|err\| res::status(err.status()).json(json!({ "message": err.message() })));` |

| `?` on | Status |
|---|---|
| bad parameter, query, JSON, form | `400` |
| row not found | `404` |
| unique value taken | `409` |
| anything else | `500` (details logged) |

See [Errors](errors.md).

## Security

`use paquets::securite::prelude::*;`

| Task | Code |
|---|---|
| Rate limit | `paquets::shield().guard(RateLimit::per_minute(600)).build()` |
| Bigger bodies | `.guard(BodyLimit::new(10 << 20))` and `app.body_limit(10 << 20)` |
| Remove a guard | `.without("bot_blocker")` |
| Cross-origin front end | `.guard(Cors::new().allow_origin(o)).guard(Csrf::new().trusted_origin(o))` |
| CSRF exemption | `.guard(Csrf::new().exempt("/auth/callback"))` |
| Client IP behind a proxy | `.client_ip(ClientIpSource::heroku())`, `::fly()`, `::cloudflare()` |
| Basic auth on a section | `.guard(Auth::basic("Admin").user("admin", "…").protect(["/admin"]))` |
| Log blocked requests | `.on_event(\|e\| eprintln!("{} blocked {}", e.guard, e.path))` |
| Presets | `ShieldBuilder::strict()`, `ShieldBuilder::api()` |

See [Security](security.md).

## Islands

| Task | Code |
|---|---|
| Island (in `src/lib.rs`) | `pub fn counter(props: &str) -> Node { ... }` |
| State | `let n = use_state(0);` `n.get()`, `n.set(v)`, `n.update(\|n\| *n += 1)`, `n.with(\|v\| …)` |
| Events | `.on_click(\|e\| …)`, `.on_input(\|e\| …)`, `.on("keydown", \|e\| …)`; `e.value()`, `e.key()` |
| Dynamic | `.text_dyn(\|\| …)`, `.attr_dyn(name, \|\| …)`, `.class_if(class, \|\| …)`, `.bool_attr(name, \|\| …)` |
| Register | `#[wasm_bindgen(start)] pub fn start() { paquets::hydrate(&[("counter", counter)]); }` |
| Place | `island("counter", "0", counter)` |
| Build | `paquets dev`, `paquets build` → `pkg/` |

See [Islands](islands.md).

## Tests

| Task | Code |
|---|---|
| Client | `let client = app().test().await;` |
| Requests | `client.get("/")`, `.post(...)`, `.put`, `.patch`, `.delete`, then `.await` |
| Body, header | `.json(&json!({...}))`, `.form(&[("k", "v")])`, `.header("k", "v")` |
| Response | `res.status()`, `res.text()`, `res.json::<T>()`, `res.header("k")` |
| Many requests | `app.security(paquets::shield().without("rate_limit").build());` |

See [Testing](testing.md).

## Environment variables

| Variable | Effect |
|---|---|
| `PORT` | the port (default `3000`) |
| `DATABASE_URL` | the database (default: memory) |
| `DYNO` | set by Heroku: client IP read from `X-Forwarded-For` |

## Deployment

| Task | Where |
|---|---|
| One-tap Heroku | `https://heroku.com/deploy?template=https://github.com/<you>/<repo>` |
| Automatic Heroku deploys | GitHub secret `HEROKU_API_KEY`, variable `HEROKU_APP_NAME` (optional `HEROKU_REGION`) |
| Database on Heroku | Resources → Heroku Postgres |
| Redeploy by hand | Actions → Deploy to Heroku → Run workflow |
| Docker | `docker build -t my-app . && docker run -p 8080:8080 my-app` |
| Skip the automatic merge | name the branch `wip/...` |
| Publish on crates.io | secret `CARGO_REGISTRY_TOKEN`, then Releases → Draft a new release → `v0.1.0` |

See [Heroku](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Automation](automation.md) and [Publish on crates.io](publish-crates-io.md).
