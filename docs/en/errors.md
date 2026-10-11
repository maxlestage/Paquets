# Errors

In Paquets, errors are ordinary values. A handler returns `Result<T>`, the `?` operator stops it at the first error, and the error becomes an HTTP response with the right status. This page explains what `?` does with each kind of error, how to create your own, and how to change what error responses look like.

## Result and the ? operator

`paquets::Result<T>` (in the prelude as `Result<T>`) is `Result<T, paquets::Error>`. Make your handlers return it, and use `?` on anything that can fail:

```rust
async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 if `id` is not a number
    let todo = Todo::get(req.db(), id).await?; // 404 if there is no such row
    Ok(Json(todo))
}
```

`?` converts every error into a `paquets::Error` with a status:

| Error | Status | Body sent to the client |
|---|---|---|
| invalid parameter (`req.param_as`) | `400` | ``{"error":"invalid parameter 'id': 'abc'"}`` |
| invalid query string (`req.query_as`) | `400` | `{"error":"invalid query string: ..."}` |
| invalid JSON or form (`req.json`, `req.form`) | `400` | `{"error":"invalid JSON: ..."}` |
| body too large | `413` | `{"error":"Payload Too Large"}` |
| no such row (`get`, `one`, `update`) | `404` | `{"error":"record not found"}` |
| duplicate value in a `unique` column, other constraint, transaction conflict | `409` | ``{"error":"unique constraint violated on `user.email`"}`` |
| `NULL` in a non-`Option` field, invalid value from the database | `400` | the database's message |
| any other error (I/O, parsing, connection lost…) | `500` | `{"error":"Internal Server Error"}` |

For a `500`, the details are **never** sent to the client (they could reveal how your server works), but they are written to the server's error output:

```text
[vitesse] error 500: invalid digit found in string
```

On Heroku, this line is in the app's logs.

## Creating errors

| Constructor | Status |
|---|---|
| `Error::bad_request(msg)` | `400 Bad Request` |
| `Error::unauthorized(msg)` | `401 Unauthorized` |
| `Error::forbidden(msg)` | `403 Forbidden` |
| `Error::not_found(msg)` | `404 Not Found` |
| `Error::conflict(msg)` | `409 Conflict` |
| `Error::unprocessable(msg)` | `422 Unprocessable Content` |
| `Error::internal(msg)` | `500 Internal Server Error`: the message is sent too, so keep it generic |
| `Error::new(status, msg)` | any status: `Error::new(418, "I'm a teapot")` |

The message is sent to the client as `{"error": "..."}`. Read an error's parts with `err.status()` and `err.message()`.

```rust
#[data]
struct NewTodo {
    title: String,
}

async fn create(req: Request) -> Result<(StatusCode, Json<Todo>)> {
    let new: NewTodo = req.json().await?;
    if new.title.trim().is_empty() {
        return Err(Error::unprocessable("the title cannot be empty"));
    }
    let mut todo = Todo { id: 0, title: new.title, done: false };
    todo.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}
```

### From an Option

Turn a `None` into an error with `ok_or_else`:

```rust
async fn show_member(req: Request) -> Result<Json<Member>> {
    let id: i64 = req.param_as("id")?;
    let member = Member::find(req.db(), id)
        .await?
        .ok_or_else(|| Error::not_found(format!("member {id} not found")))?;
    Ok(Json(member))
}
```

A handler can also return an `Option<T>` directly: `None` becomes `404 {"error":"Not Found"}`.

### In closures

In a closure, Rust can't guess the error type of `?`. Write it on the final `Ok`:

```rust
app.get("/double/:n", |req: Request| async move {
    let n: i64 = req.param_as("n")?;
    Ok::<_, Error>(format!("{}", n * 2))
});
```

## Choosing your own message for a database error

`?` gives database errors a generic message. To give a better one, match on the error before `?`. memoire's errors have `is_not_found()`, `is_unique_violation()` and `is_conflict()`:

```rust
#[model]
struct Member {
    id: i64,
    #[memoire(unique)]
    email: String,
}

#[data]
struct SignUp {
    email: String,
}

async fn sign_up(req: Request) -> Result<(StatusCode, Json<Member>)> {
    let form: SignUp = req.json().await?;
    let mut member = Member { id: 0, email: form.email };
    match member.insert(req.db()).await {
        Ok(_) => Ok((StatusCode::CREATED, Json(member))),
        Err(e) if e.is_unique_violation() => Err(Error::conflict("this email is already registered")),
        Err(e) => Err(e.into()),
    }
}
```

## Your own error types

Business errors often deserve their own type. Convert it to vitesse's error type, and `?` does the rest:

```rust
#[derive(Debug)]
enum ShopError {
    OutOfStock(i64),
    InvalidQuantity,
}

impl From<ShopError> for paquets::vitesse::Error {
    fn from(err: ShopError) -> Self {
        match err {
            ShopError::OutOfStock(id) => Self::conflict(format!("item {id} is out of stock")),
            ShopError::InvalidQuantity => Self::unprocessable("invalid quantity"),
        }
    }
}

// Your business logic.
fn reserve(id: i64, quantity: i64) -> Result<(), ShopError> {
    if quantity < 1 {
        return Err(ShopError::InvalidQuantity);
    }
    Err(ShopError::OutOfStock(id))
}

async fn order(req: Request) -> Result<&'static str> {
    let id: i64 = req.param_as("id")?;
    reserve(id, 1)?; // ShopError → 409 or 422
    Ok("reserved")
}
```

`Result<T>` uses `paquets::Error` by default, and `Result<T, ShopError>` works for your own functions.

### Keeping the cause in the logs

To send a clear message to the client and keep the original error in the logs, attach it with `with_source`:

```rust
async fn settings(_req: Request) -> Result<String> {
    let text = std::fs::read_to_string("settings.toml")
        .map_err(|e| Error::internal("settings unavailable").with_source(e))?;
    Ok(text)
}
```

The client receives `500 {"error":"settings unavailable"}`, and the logs get the cause: `[vitesse] error 500: No such file or directory (os error 2)`.

## Changing the format of every error

By default, errors are sent as `{"error": "message"}`. `app.on_error` replaces that format for the whole app:

```rust
app.on_error(|err: Error| {
    res::status(err.status()).json(json!({
        "error": { "status": err.status().as_u16(), "message": err.message() }
    }))
});
```

```json
{"error":{"message":"member 9 not found","status":404}}
```

- The function receives a `paquets::Error` (`Error` in the prelude): `err.status()`, `err.message()`.
- It is called for every response made from an error: your handlers' errors, the default 404, 405s, invalid bodies, 413s, panics.
- Set the status yourself (`res::status(err.status())`): otherwise the error page is sent with `200 OK`.
- It is not called for requests blocked by the shield, which have their own format (see [Security](security.md#what-a-blocked-request-looks-like)), nor for responses you build yourself such as `(404, "not here")`.

### Error pages for browsers

`on_error` doesn't see the request. To send HTML error pages to browsers and JSON to everything else, use a middleware that inspects the response:

```rust
app.middleware(|req: Request, next: Next| async move {
    let wants_html = req.header("accept").is_some_and(|a| a.contains("text/html"));
    let mut res = next.run(req).await;
    if wants_html {
        if let Some(err) = res.take_error() {
            return Page::new("Error")
                .status(err.status().as_u16())
                .body(h1().text(err.message().to_string()))
                .body(a().href("/").text("Back to the home page"))
                .into_response();
        }
    }
    res
});
```

For the "page not found" case only, `app.fallback` is simpler: see [Routing](routing.md#custom-404-page).

## Panics

A panic in a handler (an `unwrap()` on `None`, an index out of bounds…) doesn't stop the server: the client gets `500 {"error":"Internal Server Error"}`, the panic message is printed on the error output, and the other requests carry on. Still, prefer `?` and explicit errors: a panic is a bug, not a way to answer.

## Errors at startup

Some mistakes stop the app before it serves anything, with a message that says why:

| Mistake | What happens |
|---|---|
| invalid or duplicate route | panic when the route is declared: `duplicate route: GET /api/todos is already defined` |
| wrong `DATABASE_URL`, database unreachable | `run()` returns an error starting with `database:` |
| an `on_start` hook fails | `run()` prints `📦 Paquets could not start: startup: …` with the cause, and returns the error |
| the port is already taken | `run()` returns an "address in use" error |

Because `main` returns `std::io::Result<()>`, the program prints the error and exits with a non-zero code. On Heroku, the dyno then crashes (`H10` in the logs): read the line just above.

## Next steps

- [Testing](testing.md): check your errors in tests.
- [Security](security.md): the responses of the shield.
- [vitesse's error guide](https://maxlestage.github.io/Vitesse/en/docs/errors/): more about errors, `on_error` and panics.
