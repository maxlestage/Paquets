# Errores

En Paquets, los errores son valores normales. Un handler devuelve `Result<T>`, el operador `?` lo detiene en el primer error, y el error se convierte en una respuesta HTTP con el estado correcto. Esta página explica qué hace `?` con cada tipo de error, cómo crear tus propios errores y cómo cambiar el aspecto de las respuestas de error.

## Result y el operador ?

`paquets::Result<T>` (en el prelude como `Result<T>`) es `Result<T, paquets::Error>`. Haz que tus handlers lo devuelvan y usa `?` con todo lo que pueda fallar:

```rust
async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si `id` no es un número
    let todo = Todo::get(req.db(), id).await?; // 404 si no existe esa fila
    Ok(Json(todo))
}
```

`?` convierte cada error en un `paquets::Error` con un estado:

| Error | Estado | Cuerpo enviado al cliente |
|---|---|---|
| parámetro no válido (`req.param_as`) | `400` | ``{"error":"invalid parameter 'id': 'abc'"}`` |
| query string no válida (`req.query_as`) | `400` | `{"error":"invalid query string: ..."}` |
| JSON o formulario no válido (`req.json`, `req.form`) | `400` | `{"error":"invalid JSON: ..."}` |
| cuerpo demasiado grande | `413` | `{"error":"Payload Too Large"}` |
| no existe la fila (`get`, `one`, `update`) | `404` | `{"error":"record not found"}` |
| valor duplicado en una columna `unique`, otra restricción, conflicto de transacción | `409` | ``{"error":"unique constraint violated on `user.email`"}`` |
| `NULL` en un campo que no es `Option`, valor no válido procedente de la base de datos | `400` | el mensaje de la base de datos |
| cualquier otro error (E/S, análisis, conexión perdida…) | `500` | `{"error":"Internal Server Error"}` |

En un `500`, los detalles **nunca** se envían al cliente (podrían revelar cómo funciona tu servidor), pero se escriben en la salida de errores del servidor:

```text
[vitesse] error 500: invalid digit found in string
```

En Heroku, esta línea aparece en los logs de la aplicación.

## Crear errores

| Constructor | Estado |
|---|---|
| `Error::bad_request(msg)` | `400 Bad Request` |
| `Error::unauthorized(msg)` | `401 Unauthorized` |
| `Error::forbidden(msg)` | `403 Forbidden` |
| `Error::not_found(msg)` | `404 Not Found` |
| `Error::conflict(msg)` | `409 Conflict` |
| `Error::unprocessable(msg)` | `422 Unprocessable Content` |
| `Error::internal(msg)` | `500 Internal Server Error`: el mensaje también se envía, así que mantenlo genérico |
| `Error::new(status, msg)` | cualquier estado: `Error::new(418, "Soy una tetera")` |

El mensaje se envía al cliente como `{"error": "..."}`. Lee las partes de un error con `err.status()` y `err.message()`.

```rust
#[data]
struct NewTodo {
    title: String,
}

async fn create(req: Request) -> Result<(StatusCode, Json<Todo>)> {
    let new: NewTodo = req.json().await?;
    if new.title.trim().is_empty() {
        return Err(Error::unprocessable("el título no puede estar vacío"));
    }
    let mut todo = Todo { id: 0, title: new.title, done: false };
    todo.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}
```

### A partir de un Option

Convierte un `None` en un error con `ok_or_else`:

```rust
async fn show_member(req: Request) -> Result<Json<Member>> {
    let id: i64 = req.param_as("id")?;
    let member = Member::find(req.db(), id)
        .await?
        .ok_or_else(|| Error::not_found(format!("miembro {id} no encontrado")))?;
    Ok(Json(member))
}
```

Un handler también puede devolver directamente un `Option<T>`: `None` se convierte en `404 {"error":"Not Found"}`.

### En las closures

En una closure, Rust no puede adivinar el tipo de error de `?`. Escríbelo en el `Ok` final:

```rust
app.get("/double/:n", |req: Request| async move {
    let n: i64 = req.param_as("n")?;
    Ok::<_, Error>(format!("{}", n * 2))
});
```

## Elegir tu propio mensaje para un error de la base de datos

`?` da a los errores de la base de datos un mensaje genérico. Para dar uno mejor, comprueba el error con un `match` antes del `?`. Los errores de memoire tienen `is_not_found()`, `is_unique_violation()` e `is_conflict()`:

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
        Err(e) if e.is_unique_violation() => Err(Error::conflict("este correo ya está registrado")),
        Err(e) => Err(e.into()),
    }
}
```

## Tus propios tipos de error

Los errores de negocio a menudo merecen su propio tipo. Conviértelo al tipo de error de vitesse, y `?` hace el resto:

```rust
#[derive(Debug)]
enum ShopError {
    OutOfStock(i64),
    InvalidQuantity,
}

impl From<ShopError> for paquets::vitesse::Error {
    fn from(err: ShopError) -> Self {
        match err {
            ShopError::OutOfStock(id) => Self::conflict(format!("el artículo {id} está agotado")),
            ShopError::InvalidQuantity => Self::unprocessable("cantidad no válida"),
        }
    }
}

// Tu lógica de negocio.
fn reserve(id: i64, quantity: i64) -> Result<(), ShopError> {
    if quantity < 1 {
        return Err(ShopError::InvalidQuantity);
    }
    Err(ShopError::OutOfStock(id))
}

async fn order(req: Request) -> Result<&'static str> {
    let id: i64 = req.param_as("id")?;
    reserve(id, 1)?; // ShopError → 409 o 422
    Ok("reservado")
}
```

`Result<T>` usa `paquets::Error` por defecto, y `Result<T, ShopError>` funciona para tus propias funciones.

### Conservar la causa en los logs

Para enviar un mensaje claro al cliente y conservar el error original en los logs, adjúntalo con `with_source`:

```rust
async fn settings(_req: Request) -> Result<String> {
    let text = std::fs::read_to_string("settings.toml")
        .map_err(|e| Error::internal("ajustes no disponibles").with_source(e))?;
    Ok(text)
}
```

El cliente recibe `500 {"error":"ajustes no disponibles"}`, y los logs reciben la causa: `[vitesse] error 500: No such file or directory (os error 2)`.

## Cambiar el formato de todos los errores

Por defecto, los errores se envían como `{"error": "mensaje"}`. `app.on_error` reemplaza ese formato para toda la aplicación:

```rust
app.on_error(|err: Error| {
    res::status(err.status()).json(json!({
        "error": { "status": err.status().as_u16(), "message": err.message() }
    }))
});
```

```json
{"error":{"message":"miembro 9 no encontrado","status":404}}
```

- La función recibe un `paquets::Error` (`Error` en el prelude): `err.status()`, `err.message()`.
- Se llama para cada respuesta construida a partir de un error: los errores de tus handlers, el 404 por defecto, los 405, los cuerpos no válidos, los 413 y los pánicos.
- Fija tú el estado (`res::status(err.status())`): si no, la página de error se envía con `200 OK`.
- No se llama para las peticiones bloqueadas por el escudo, que tienen su propio formato (consulta [Seguridad](security.md#cómo-es-una-petición-bloqueada)), ni para las respuestas que construyes tú mismo, como `(404, "no está aquí")`.

### Páginas de error para los navegadores

`on_error` no ve la petición. Para enviar páginas de error HTML a los navegadores y JSON a todo lo demás, usa un middleware que inspeccione la respuesta:

```rust
app.middleware(|req: Request, next: Next| async move {
    let wants_html = req.header("accept").is_some_and(|a| a.contains("text/html"));
    let mut res = next.run(req).await;
    if wants_html {
        if let Some(err) = res.take_error() {
            return Page::new("Error")
                .status(err.status().as_u16())
                .body(h1().text(err.message().to_string()))
                .body(a().href("/").text("Volver a la página de inicio"))
                .into_response();
        }
    }
    res
});
```

Solo para el caso de «página no encontrada», `app.fallback` es más sencillo: consulta [Enrutamiento](routing.md#página-404-personalizada).

## Pánicos

Un pánico en un handler (un `unwrap()` sobre `None`, un índice fuera de rango…) no detiene el servidor: el cliente recibe `500 {"error":"Internal Server Error"}`, el mensaje del pánico se imprime en la salida de errores y las demás peticiones siguen su curso. Aun así, es preferible usar `?` y errores explícitos: un pánico es un bug, no una forma de responder.

## Errores al arrancar

Algunos errores detienen la aplicación antes de que sirva nada, con un mensaje que explica por qué:

| Error | Qué ocurre |
|---|---|
| ruta no válida o duplicada | pánico al declarar la ruta: `duplicate route: GET /api/todos is already defined` |
| `DATABASE_URL` incorrecta, base de datos inaccesible | `run()` devuelve un error que empieza por `database:` |
| un hook `on_start` falla | `run()` muestra `📦 Paquets could not start: startup: …` con la causa y devuelve el error |
| el puerto ya está ocupado | `run()` devuelve un error «address in use» |

Como `main` devuelve `std::io::Result<()>`, el programa muestra el error y termina con un código distinto de cero. En Heroku, el dyno se cae entonces (`H10` en los logs): lee la línea justo encima.

## Próximos pasos

- [Pruebas](testing.md): comprueba tus errores en las pruebas.
- [Seguridad](security.md): las respuestas del escudo.
- [La guía de errores de vitesse](https://maxlestage.github.io/Vitesse/es/docs/errors/): más sobre los errores, `on_error` y los pánicos.
