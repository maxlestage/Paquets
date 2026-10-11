# Enrutamiento

El enrutamiento decide qué código responde a una petición, según su método (`GET`, `POST`…) y su ruta (`/users/42`). Paquets usa el enrutador de [Vitesse](https://maxlestage.github.io/Vitesse/), con el vocabulario de Express.js: `app.get(path, handler)`, parámetros escritos `:id`, middleware con `next`. Esta página cubre las rutas, los parámetros, los routers, el middleware, la página 404 y los archivos estáticos.

## Tus primeras rutas

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async { "Inicio" });
    app.get("/about", |_| async { Page::new("Acerca de").body(h1().text("Quiénes somos")) });
    app.post("/contact", |_| async { (201, "Mensaje recibido") });

    app.run()
}
```

Cada llamada recibe una **ruta** y un **handler**. Un handler es una función o closure `async` que recibe la petición y devuelve algo que se convierte en una respuesta: texto, JSON, una página, un estado, una redirección… (consulta [Peticiones y respuestas](requests-responses.md)).

## Tres formas de escribir un handler

```rust
// 1. Una closure que no necesita la petición: `|_|`.
app.get("/", |_| async { "Inicio" });

// 2. Una closure que usa la petición: indica su tipo y escribe `async move`.
app.get("/hello/:name", |req: Request| async move {
    format!("¡Hola, {}!", req.param("name").unwrap_or("tú"))
});

// 3. Una función con nombre, lo mejor en cuanto el código ocupa más de unas pocas líneas.
async fn list_users(_req: Request) -> Json<Vec<&'static str>> {
    Json(vec!["ada", "grace"])
}
app.get("/users", list_users);
```

`async move` hace que el future de la closure sea dueño de la petición, y eso es lo que le permite leer la petición después de un `.await`. Si olvidas `move`, el compilador da un error de préstamo.

## Métodos HTTP

| Método | Acepta |
|---|---|
| `app.get(path, handler)` | `GET`, y también `HEAD` |
| `app.post(path, handler)` | `POST` |
| `app.put(path, handler)` | `PUT` |
| `app.patch(path, handler)` | `PATCH` |
| `app.delete(path, handler)` | `DELETE` |
| `app.head(path, handler)` | `HEAD` |
| `app.options(path, handler)` | `OPTIONS` |
| `app.all(path, handler)` | todos los métodos |
| `app.route(method, path, handler)` | el `Method` que pases, incluidos los no estándar |
| `app.ws(path, handler)` | una conexión WebSocket (consulta [WebSocket](websocket.md)) |
| `app.api::<Model>(path)` | seis rutas REST para un modelo (consulta [API REST](rest-api.md)) |

Los métodos de enrutamiento devuelven `&mut` a la aplicación, así que puedes encadenarlos:

```rust
app.get("/articles", list_articles)
    .post("/articles", create_article)
    .get("/articles/:id", show_article)
    .delete("/articles/:id", delete_article);
```

## Parámetros de ruta

Un segmento que empieza por `:` captura esa parte de la ruta. `req.param(name)` la devuelve como un `Option<&str>`:

```rust
app.get("/users/:id/posts/:post", |req: Request| async move {
    let user = req.param("id").unwrap_or_default();
    let post = req.param("post").unwrap_or_default();
    format!("artículo {post} del usuario {user}")
});
```

`GET /users/42/posts/7` responde `artículo 7 del usuario 42`. Los valores llegan ya decodificados (`/users/Fran%C3%A7ois` da `François`), y un parámetro siempre cubre un segmento entero: nunca coincide con un segmento vacío ni con una `/`.

### Parámetros tipados

`req.param_as::<T>(name)` convierte un parámetro en un número, un `bool` o cualquier tipo que implemente `FromStr`. Cuando la conversión falla, devuelve un error que se convierte en un `400 Bad Request`, así que `?` es todo lo que necesitas:

```rust
async fn show_user(req: Request) -> Result<String> {
    let id: i64 = req.param_as("id")?; // 400 si `id` no es un número
    Ok(format!("usuario #{id}"))
}

app.get("/users/:id", show_user);
```

```text
GET /users/42   → 200 usuario #42
GET /users/abc  → 400 {"error":"invalid parameter 'id': 'abc'"}
```

## Comodines

Un segmento `*name` captura el resto de la ruta, barras incluidas. Debe ser el último segmento:

```rust
app.get("/files/*path", |req: Request| async move {
    format!("has pedido {}", req.param("path").unwrap_or_default())
});
```

`/files/2024/report.pdf` da `path = "2024/report.pdf"`, y `/files` da una cadena vacía.

## Cómo se elige una ruta

- **Gana la ruta más específica**, sea cual sea el orden de declaración: un segmento estático gana a un parámetro, que gana a un comodín. Con `/users/new` y `/users/:id`, una petición a `/users/new` va a la primera.
- **La barra final se ignora**: `/about/` coincide con `/about`.
- **Las rutas distinguen mayúsculas y minúsculas**: `/About` no coincide con `/about`.
- **La query string se ignora**: `/search?q=rust` coincide con `/search` (léela con `req.query("q")`, consulta [Peticiones y respuestas](requests-responses.md#query-strings)).

Los errores en la definición de las rutas se detectan **al arrancar la aplicación**, con un mensaje claro: una ruta que no empieza por `/`, un comodín que no está al final o el mismo método declarado dos veces en la misma ruta. Por ejemplo, `app.api::<Todo>("/api/todos")` seguido de tu propio `app.get("/api/todos", ...)` provoca un pánico con `duplicate route: GET /api/todos is already defined`.

Parte del comportamiento HTTP viene incluido:

- una petición `HEAD` usa la ruta `GET` y recibe las cabeceras sin el cuerpo;
- una ruta que existe, pero no para ese método, recibe `405 Method Not Allowed` con una cabecera `Allow` que enumera los métodos que funcionan;
- una petición `OPTIONS` recibe `204 No Content` con la misma cabecera `Allow`.

## Routers: agrupar rutas

Un `Router` agrupa rutas bajo un prefijo, como `express.Router()`:

```rust
let mut admin = Router::new();
admin.get("/", |_| async { "Panel" });
admin.get("/users", |req: Request| async move {
    Ok::<_, Error>(Json(User::all(req.db()).await?))
});

app.mount("/admin", admin); // GET /admin y GET /admin/users
```

Los handlers de un router reciben la misma petición que en cualquier otro sitio: `req.db()`, `req.state()` y el escudo de seguridad funcionan igual. Un router puede tener su propio middleware (ver más abajo), lo que lo convierte en la forma natural de proteger una sección del sitio.

## Middleware

Un middleware se ejecuta alrededor de los handlers. Recibe la petición y `next`, el resto de la cadena, y devuelve una respuesta. Puede actuar antes del handler, después de él o responder por sí mismo:

```rust
use std::time::Instant;

app.middleware(|req: Request, next: Next| async move {
    let start = Instant::now();
    let res = next.run(req).await; // el resto de la cadena y luego el handler
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    res.header("server-timing", format!("app;dur={ms:.2}"))
});
```

Para detener una petición, devuelve una respuesta sin llamar a `next.run`:

```rust
async fn require_admin(req: Request, next: Next) -> Response {
    if req.header("x-admin-key") != Some("let-me-in") {
        return Error::forbidden("solo para administradores").into_response();
    }
    next.run(req).await
}
```

Hay tres sitios donde poner un middleware:

| Dónde | Código | Se ejecuta para |
|---|---|---|
| Toda la aplicación | `app.middleware(m)` | cada petición, incluidos los 404, sea cual sea el orden de declaración |
| Un router | `router.middleware(m)` | cada petición bajo el prefijo del router |
| Una ruta | `app.get("/stats", stats.with(m))` | solo esa ruta |

```rust
let mut admin = Router::new();
admin.middleware(require_admin);
admin.get("/", |_| async { "Panel" });
app.mount("/admin", admin);

// Una sola ruta. `.with` viene del trait `HandlerExt`, que está en el prelude.
app.get("/stats", stats.with(require_admin));
```

Para llamar a `.with` sobre una closure, envuelve la closure entre paréntesis: `(|_| async { "ok" }).with(require_admin)`.

### El orden de toda la cadena

Para cada petición, Paquets ejecuta estas capas, en este orden:

1. el log de peticiones (una línea por petición; `app.quiet()` lo desactiva);
2. el escudo de seguridad (consulta [Seguridad](security.md)): una petición bloqueada se detiene aquí, antes de cualquier código tuyo;
3. tus middlewares globales, en el orden en que los añadiste;
4. los archivos estáticos de `public/`;
5. el enrutamiento y, después, el middleware del router, el middleware de la ruta (`.with`) y el handler.

La respuesta vuelve a través de las mismas capas en orden inverso, así que el escudo añade sus cabeceras de seguridad a todas las respuestas, también a las tuyas.

## Página 404 personalizada

Cuando ninguna ruta coincide, Paquets responde `404` con `{"error":"Cannot GET /path"}`. `app.fallback` reemplaza esa respuesta, por ejemplo por una página:

```rust
app.fallback(|req: Request| async move {
    Page::new("Página no encontrada")
        .status(404)
        .body(h1().text("Página no encontrada"))
        .body(p().text(format!("No hay nada en {}.", req.path())))
        .body(a().href("/").text("Volver a la página de inicio"))
});
```

No olvides `.status(404)`: una `Page` se envía con `200 OK` por defecto. El fallback no se llama para un `405` (la ruta existe, con otro método). Para cambiar todas las respuestas de error a la vez, consulta [Errores](errors.md#cambiar-el-formato-de-todos-los-errores).

## Archivos estáticos

Los archivos de la carpeta `public/` se sirven en la raíz del sitio, sin código. Para servir otra carpeta bajo un prefijo, usa `static_dir`:

```rust
app.static_dir("/downloads", "files"); // files/report.pdf → /downloads/report.pdf
```

Las rutas son relativas a la carpeta desde la que se lanza el servidor. Los archivos estáticos reciben las cabeceras `Content-Type`, `ETag` y `Last-Modified` correctas, `304 Not Modified` y peticiones parciales (vídeo). Los archivos ocultos (`.env`, `.git/`) y las rutas con `..` se rechazan. Más opciones en la [guía de archivos estáticos de Vitesse](https://maxlestage.github.io/Vitesse/es/docs/static-files/).

## Más información

La documentación de Vitesse trata el enrutamiento con más profundidad: [enrutamiento](https://maxlestage.github.io/Vitesse/es/docs/routing/), [middleware](https://maxlestage.github.io/Vitesse/es/docs/middleware/) (incluidos los middlewares integrados `logger`, `cors` y `timeout`) y [routers](https://maxlestage.github.io/Vitesse/es/docs/routers/). Todo lo que hay allí funciona en Paquets: `App` da acceso a todos los métodos de la aplicación de Vitesse.

## Próximos pasos

- [Peticiones y respuestas](requests-responses.md): leer parámetros, query strings y cuerpos, y construir cualquier respuesta.
- [Páginas](pages.md): devolver páginas HTML completas.
- [Seguridad](security.md): lo que hace el escudo antes de que se ejecute tu middleware.
