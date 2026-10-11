# Requests and responses

Every handler receives a `Request` and returns a response. This page shows how to read everything a request carries (parameters, query string, headers, cookies, JSON and form bodies) and every kind of response you can return (text, JSON, pages, statuses, redirects, cookies, files).

## The request

A handler receives the request by value: `|req: Request| async move { ... }` or `async fn handler(req: Request)`. Nearly every method takes `&self`, so you can read a parameter, then the body, then a header, in any order.

| Method | Returns | Example |
|---|---|---|
| `req.method()` | `&Method` | `GET` |
| `req.path()` | `&str` | `/users/42` |
| `req.param("id")` | `Option<&str>` | the `:id` segment of the route |
| `req.param_as::<T>("id")` | `Result<T, _>` | a typed parameter, `400` if invalid |
| `req.query("q")` | `Option<Cow<str>>` | `?q=rust` |
| `req.query_as::<T>()` | `Result<T, _>` | the whole query string in a struct, `400` if invalid |
| `req.header("user-agent")` | `Option<&str>` | a header (name is case-insensitive) |
| `req.cookie("session")` | `Option<&str>` | a cookie |
| `req.json::<T>().await` | `Result<T, _>` | the body as JSON, `400` if invalid |
| `req.form::<T>().await` | `Result<T, _>` | an HTML form, `400` if invalid |
| `req.text().await` | `Result<String, _>` | the body as text |
| `req.bytes().await` | `Result<Bytes, _>` | the raw body |
| `req.db()` | `&'static Db` | the database (see [Database](database.md)) |
| `req.state::<T>()` | `&T` | shared state (see [Configuration](configuration.md#shared-state)) |

Every method that can fail returns an error that `?` turns into the right status, so a handler only describes the success path.

## Query strings

For one or two values, `req.query(name)` is enough. It returns the decoded value, or `None`:

```rust
app.get("/search", |req: Request| async move {
    let q = req.query("q").unwrap_or_default();
    let page: u32 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    format!("searching {q:?}, page {page}")
});
```

`GET /search?q=caf%C3%A9&page=2` answers `searching "café", page 2`.

For more values, describe the query string with a `#[data]` struct and read it in one call with `req.query_as()`. Use `Option` for optional values and `#[serde(default)]` for values with a default:

```rust
#[data]
struct Search {
    q: String,
    page: Option<u32>,
    #[serde(default)]
    exact: bool,
}

async fn search(req: Request) -> Result<String> {
    let s: Search = req.query_as()?; // 400 if `q` is missing or `page` is not a number
    Ok(format!("{} (page {}, exact: {})", s.q, s.page.unwrap_or(1), s.exact))
}
```

`#[data]` derives `Clone`, `Debug`, `Serialize` and `Deserialize` for the struct, with the `serde` crate re-exported by Paquets: no other dependency is needed.

## Headers and cookies

```rust
app.get("/whoami", |req: Request| async move {
    let agent = req.header("user-agent").unwrap_or("unknown");
    let visits = req.cookie("visits").unwrap_or("0");
    format!("You use {agent} and came {visits} times before.")
});
```

- `req.header(name)` returns the first value of a header as text. `req.headers()` returns all of them.
- `req.cookie(name)` reads a cookie sent by the browser, as is: it is neither decoded nor signed.

## The client's IP address

The security shield finds the real address of the client, even behind Heroku's router, and attaches it to the request. Read it with:

```rust
use paquets::securite::ClientIp;

async fn ip(req: Request) -> String {
    match req.extensions().get::<ClientIp>() {
        Some(ClientIp(ip)) => format!("Your IP: {ip}"),
        None => "unknown".to_string(),
    }
}
```

`req.ip()` also exists, but it returns the address of the direct connection: behind a proxy or Heroku's router, that is the proxy's address. Prefer `ClientIp`. See [Security](security.md#the-client-ip-behind-a-proxy).

## JSON bodies

Describe the expected body with a `#[data]` struct (or use a `#[model]`), and read it with `req.json()`:

```rust
#[data]
struct NewUser {
    name: String,
    email: String,
    newsletter: Option<bool>,
}

async fn create_user(req: Request) -> Result<(StatusCode, Json<paquets::serde_json::Value>)> {
    let user: NewUser = req.json().await?;
    if !user.email.contains('@') {
        return Err(Error::unprocessable("the email address is not valid"));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "name": user.name, "newsletter": user.newsletter.unwrap_or(false) })),
    ))
}
```

If the body is not valid JSON, if a field is missing or has the wrong type, `req.json()` fails and the client gets a `400` that explains why. Sending `{"name":"Ada"}` to this handler gives:

```json
{"error":"invalid JSON: missing field `email` at line 1 column 14"}
```

Unknown fields are ignored. For JSON without a fixed shape, ask for a `serde_json::Value` (re-exported as `paquets::serde_json`, and `json!` is in the prelude).

## Forms

HTML forms (`application/x-www-form-urlencoded`) work the same way, with `req.form()`:

```rust
#[data]
struct Login {
    email: String,
    password: String,
    // An unchecked checkbox is not sent at all: default to `false`.
    #[serde(default)]
    remember: bool,
}

async fn login(req: Request) -> Result<Redirect> {
    let form: Login = req.form().await?;
    if form.email.is_empty() || form.password.is_empty() {
        return Err(Error::bad_request("email and password are required"));
    }
    // ... check the password, open a session ...
    Ok(Redirect::see_other("/dashboard"))
}
```

File uploads (`multipart/form-data`) are not parsed for you: read the raw body with `req.take_body()` and give it to a multipart crate. See [vitesse's guide to requests](https://maxlestage.github.io/Vitesse/en/docs/requests/).

## Body size

Two limits protect the server from huge bodies:

| Limit | Default | Checked by | Response |
|---|---|---|---|
| The shield's body limit | 2 MiB | the `Content-Length` header, before your code runs | `413` |
| The read limit | 2 MiB | `req.json()`, `req.form()`, `req.text()` and `req.bytes()` | `413 {"error":"Payload Too Large"}` |

So, by default, a handler reads at most 2 MiB. To accept bigger bodies, raise both limits:

```rust
use paquets::securite::prelude::*;

let mut app = App::new();
app.body_limit(10 * 1024 * 1024); // read up to 10 MiB
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

`paquets::shield()` is the default shield, ready to be changed: see [Security](security.md#changing-the-shield).

## Responses

A handler doesn't write to a response object: it **returns** its response. Anything that implements `IntoResponse` works:

| Return | Status | Content-Type |
|---|---|---|
| `&'static str`, `String` | `200` | `text/plain; charset=utf-8` |
| `Json(value)` (any `Serialize` value) | `200` | `application/json` |
| `json!({ ... })` | `200` | `application/json` |
| `Page` | `200`, or the one set with `.status(...)` | `text/html; charset=utf-8` |
| `Html(body)` | `200` | `text/html; charset=utf-8` |
| `StatusCode::NO_CONTENT` | that status | the reason phrase as text |
| `(status, body)` | `status` | the one of `body` |
| `Redirect::to("/")` | `302` | none, with a `Location` header |
| `Option<T>` | `T`, or `404` for `None` | |
| `Result<T>` | `T`, or the error's status | |
| `Response` | whatever you built | |

```rust
app.get("/text", |_| async { "Hello!" });
app.get("/user", |_| async { Json(json!({ "id": 1, "name": "Ada" })) });
app.get("/stats", |_| async { json!({ "users": 42, "online": true }) });
app.post("/jobs", |_| async { (StatusCode::ACCEPTED, "queued") });
app.post("/users", |_| async { (201, Json(json!({ "id": 2 }))) });
app.delete("/cache", |_| async { StatusCode::NO_CONTENT });
app.get("/old-page", |_| async { Redirect::permanent("/new-page") });
```

### Pages or Html?

Return a [`Page`](pages.md) for HTML: it is a complete document with SEO tags, the theme, and a Content-Security-Policy that allows the page's own styles and scripts. `Html(...)` sends a string of HTML as is, with the shield's default policy, which blocks inline `<style>` and `<script>` tags it doesn't know about. Use `Html` only for fragments or HTML that loads its resources from files.

### Redirects

| Helper | Status | Use it |
|---|---|---|
| `Redirect::to(url)` | `302 Found` | a temporary redirect |
| `Redirect::see_other(url)` | `303 See Other` | after a form `POST` |
| `Redirect::temporary(url)` | `307 Temporary Redirect` | keeps the method and the body |
| `Redirect::permanent(url)` | `301 Moved Permanently` | a page that moved for good |

### Headers and cookies: the res builder

When you need to set several things at once, build a `Response`. The `res` module provides entry points in the style of Express, and every method can be chained:

```rust
app.get("/custom", |_| async {
    res::status(202)
        .header("x-version", "1.0")
        .cookie(Cookie::new("seen", "1"))
        .json(json!({ "ok": true }))
});

app.post("/logout", |_| async { res::redirect("/").clear_cookie("session") });
```

| Method | Effect |
|---|---|
| `res::status(code)` / `.status(code)` | the status |
| `res::json(v)` / `.json(v)`, `.text(s)`, `.html(s)` | the body, with its `Content-Type` |
| `.header(name, value)` | sets a header |
| `.append_header(name, value)` | adds one more value to a header |
| `.cookie(Cookie::new(name, value))` | sets a cookie (`.max_age(...)`, `.path(...)`…) |
| `.clear_cookie(name)` | deletes a cookie |
| `res::redirect(url)` | a `302` redirect |

The shield's cookie guard adds `Secure`, `HttpOnly` and `SameSite=Lax` to every cookie that doesn't set them, so cookies are safe even if you forget. Browsers accept `Secure` cookies on `http://localhost` while you develop.

### Files

```rust
app.get("/terms", |_| async { res::file("legal/terms.pdf").await });
app.get("/invoice", |_| async { res::download("files/invoice-42.pdf", "invoice.pdf").await });
```

`res::file` guesses the `Content-Type` from the extension, streams large files and answers `404` if the file doesn't exist. `res::download` also asks the browser to save the file under the given name. For a whole folder, use `public/` or `app.static_dir` (see [Routing](routing.md#static-files)).

### Your own response types

Implement `IntoResponse` to return your own types directly:

```rust
struct Csv(String);

impl IntoResponse for Csv {
    fn into_response(self) -> Response {
        res::send(self.0).content_type("text/csv; charset=utf-8")
    }
}

app.get("/export.csv", |_| async { Csv("id,name\n1,Ada\n".into()) });
```

## Learn more

- [vitesse: Reading requests](https://maxlestage.github.io/Vitesse/en/docs/requests/) and [Sending responses](https://maxlestage.github.io/Vitesse/en/docs/responses/): every method, streaming bodies, Server-Sent Events.
- [Errors](errors.md): what `?` does with each kind of error.

## Next steps

- [Pages](pages.md): HTML pages with SEO tags and a theme.
- [Database](database.md): read and write data from your handlers.
