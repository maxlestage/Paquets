# Peticiones y respuestas

Cada handler recibe una `Request` y devuelve una respuesta. Esta página muestra cómo leer todo lo que lleva una petición (parámetros, query string, cabeceras, cookies, cuerpos JSON y de formulario) y todos los tipos de respuesta que puedes devolver (texto, JSON, páginas, estados, redirecciones, cookies, archivos).

## La petición

Un handler recibe la petición por valor: `|req: Request| async move { ... }` o `async fn handler(req: Request)`. Casi todos los métodos reciben `&self`, así que puedes leer un parámetro, luego el cuerpo y luego una cabecera, en cualquier orden.

| Método | Devuelve | Ejemplo |
|---|---|---|
| `req.method()` | `&Method` | `GET` |
| `req.path()` | `&str` | `/users/42` |
| `req.param("id")` | `Option<&str>` | el segmento `:id` de la ruta |
| `req.param_as::<T>("id")` | `Result<T, _>` | un parámetro tipado, `400` si no es válido |
| `req.query("q")` | `Option<Cow<str>>` | `?q=rust` |
| `req.query_as::<T>()` | `Result<T, _>` | toda la query string en un struct, `400` si no es válida |
| `req.header("user-agent")` | `Option<&str>` | una cabecera (el nombre no distingue mayúsculas y minúsculas) |
| `req.cookie("session")` | `Option<&str>` | una cookie |
| `req.json::<T>().await` | `Result<T, _>` | el cuerpo como JSON, `400` si no es válido |
| `req.form::<T>().await` | `Result<T, _>` | un formulario HTML, `400` si no es válido |
| `req.text().await` | `Result<String, _>` | el cuerpo como texto |
| `req.bytes().await` | `Result<Bytes, _>` | el cuerpo en bruto |
| `req.db()` | `&'static Db` | la base de datos (consulta [Base de datos](database.md)) |
| `req.state::<T>()` | `&T` | el estado compartido (consulta [Configuración](configuration.md#estado-compartido)) |

Todos los métodos que pueden fallar devuelven un error que `?` convierte en el estado correcto, así que un handler solo describe el camino del éxito.

## Query strings

Para uno o dos valores, basta con `req.query(name)`. Devuelve el valor decodificado, o `None`:

```rust
app.get("/search", |req: Request| async move {
    let q = req.query("q").unwrap_or_default();
    let page: u32 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    format!("buscando {q:?}, página {page}")
});
```

`GET /search?q=caf%C3%A9&page=2` responde `buscando "café", página 2`.

Para más valores, describe la query string con un struct `#[data]` y léela en una sola llamada con `req.query_as()`. Usa `Option` para los valores opcionales y `#[serde(default)]` para los valores con un valor por defecto:

```rust
#[data]
struct Search {
    q: String,
    page: Option<u32>,
    #[serde(default)]
    exact: bool,
}

async fn search(req: Request) -> Result<String> {
    let s: Search = req.query_as()?; // 400 si falta `q` o si `page` no es un número
    Ok(format!("{} (página {}, exacta: {})", s.q, s.page.unwrap_or(1), s.exact))
}
```

`#[data]` deriva `Clone`, `Debug`, `Serialize` y `Deserialize` para el struct, con el crate `serde` que reexporta Paquets: no hace falta ninguna otra dependencia.

## Cabeceras y cookies

```rust
app.get("/whoami", |req: Request| async move {
    let agent = req.header("user-agent").unwrap_or("desconocido");
    let visits = req.cookie("visits").unwrap_or("0");
    format!("Usas {agent} y has venido {visits} veces antes.")
});
```

- `req.header(name)` devuelve como texto el primer valor de una cabecera. `req.headers()` las devuelve todas.
- `req.cookie(name)` lee una cookie enviada por el navegador, tal cual: no se decodifica ni se firma.

## La dirección IP del cliente

El escudo de seguridad encuentra la dirección real del cliente, incluso detrás del router de Heroku, y la adjunta a la petición. Léela así:

```rust
use paquets::securite::ClientIp;

async fn ip(req: Request) -> String {
    match req.extensions().get::<ClientIp>() {
        Some(ClientIp(ip)) => format!("Tu IP: {ip}"),
        None => "desconocida".to_string(),
    }
}
```

`req.ip()` también existe, pero devuelve la dirección de la conexión directa: detrás de un proxy o del router de Heroku, es la dirección del proxy. Es preferible `ClientIp`. Consulta [Seguridad](security.md#la-ip-del-cliente-detrás-de-un-proxy).

## Cuerpos JSON

Describe el cuerpo esperado con un struct `#[data]` (o usa un `#[model]`) y léelo con `req.json()`:

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
        return Err(Error::unprocessable("la dirección de correo no es válida"));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "name": user.name, "newsletter": user.newsletter.unwrap_or(false) })),
    ))
}
```

Si el cuerpo no es JSON válido, o si falta un campo o tiene un tipo incorrecto, `req.json()` falla y el cliente recibe un `400` que explica por qué. Enviar `{"name":"Ada"}` a este handler da:

```json
{"error":"invalid JSON: missing field `email` at line 1 column 14"}
```

Los campos desconocidos se ignoran. Para un JSON sin forma fija, pide un `serde_json::Value` (reexportado como `paquets::serde_json`; `json!` está en el prelude).

## Formularios

Los formularios HTML (`application/x-www-form-urlencoded`) funcionan igual, con `req.form()`:

```rust
#[data]
struct Login {
    email: String,
    password: String,
    // Una casilla sin marcar no se envía en absoluto: `false` por defecto.
    #[serde(default)]
    remember: bool,
}

async fn login(req: Request) -> Result<Redirect> {
    let form: Login = req.form().await?;
    if form.email.is_empty() || form.password.is_empty() {
        return Err(Error::bad_request("el correo y la contraseña son obligatorios"));
    }
    // ... comprobar la contraseña, abrir una sesión ...
    Ok(Redirect::see_other("/dashboard"))
}
```

### Subida de archivos

Con la feature `multipart`, `req.multipart()?` lee un formulario enviado con `enctype="multipart/form-data"`, campo a campo; responde `415` cuando la petición es de otro tipo. Los límites del cuerpo (2 MiB por defecto) también se aplican a las subidas: para aceptar archivos más grandes, sube los dos (consulta [Tamaño del cuerpo](#tamaño-del-cuerpo)). Consulta la [guía de Vitesse sobre la subida de archivos](https://maxlestage.github.io/Vitesse/es/docs/requests/#subida-de-archivos-multipart).

Sin la feature, lee el cuerpo en bruto con `req.take_body()` y pásaselo a un crate de multipart.

## Tamaño del cuerpo

Dos límites protegen el servidor de los cuerpos enormes:

| Límite | Por defecto | Lo comprueba | Respuesta |
|---|---|---|---|
| El límite de cuerpo del escudo | 2 MiB | la cabecera `Content-Length`, antes de que se ejecute tu código | `413` |
| El límite de lectura | 2 MiB | `req.json()`, `req.form()`, `req.text()` y `req.bytes()` | `413 {"error":"Payload Too Large"}` |

Así que, por defecto, un handler lee como máximo 2 MiB. Para aceptar cuerpos más grandes, sube los dos límites:

```rust
use paquets::securite::prelude::*;

let mut app = App::new();
app.body_limit(10 * 1024 * 1024); // leer hasta 10 MiB
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

`paquets::shield()` es el escudo por defecto, listo para modificarlo: consulta [Seguridad](security.md#modificar-el-escudo).

## Respuestas

Un handler no escribe en un objeto de respuesta: **devuelve** su respuesta. Funciona cualquier cosa que implemente `IntoResponse`:

| Valor devuelto | Estado | Content-Type |
|---|---|---|
| `&'static str`, `String` | `200` | `text/plain; charset=utf-8` |
| `Json(value)` (cualquier valor `Serialize`) | `200` | `application/json` |
| `json!({ ... })` | `200` | `application/json` |
| `Page` | `200`, o el que se haya fijado con `.status(...)` | `text/html; charset=utf-8` |
| `Html(body)` | `200` | `text/html; charset=utf-8` |
| `StatusCode::NO_CONTENT` | ese estado | la frase del estado, como texto |
| `(status, body)` | `status` | el de `body` |
| `Redirect::to("/")` | `302` | ninguno, con una cabecera `Location` |
| `Option<T>` | el de `T`, o `404` para `None` | |
| `Result<T>` | el de `T`, o el estado del error | |
| `Response` | lo que hayas construido | |

```rust
app.get("/text", |_| async { "¡Hola!" });
app.get("/user", |_| async { Json(json!({ "id": 1, "name": "Ada" })) });
app.get("/stats", |_| async { json!({ "users": 42, "online": true }) });
app.post("/jobs", |_| async { (StatusCode::ACCEPTED, "en cola") });
app.post("/users", |_| async { (201, Json(json!({ "id": 2 }))) });
app.delete("/cache", |_| async { StatusCode::NO_CONTENT });
app.get("/old-page", |_| async { Redirect::permanent("/new-page") });
```

### ¿Page o Html?

Devuelve una [`Page`](pages.md) para el HTML: es un documento completo con etiquetas SEO, el tema y una Content-Security-Policy que permite los estilos y scripts propios de la página. `Html(...)` envía una cadena de HTML tal cual, con la política por defecto del escudo, que bloquea las etiquetas `<style>` y `<script>` inline que no conoce. Usa `Html` solo para fragmentos o para HTML que carga sus recursos desde archivos.

### Redirecciones

| Función | Estado | Úsala para |
|---|---|---|
| `Redirect::to(url)` | `302 Found` | una redirección temporal |
| `Redirect::see_other(url)` | `303 See Other` | después del `POST` de un formulario |
| `Redirect::temporary(url)` | `307 Temporary Redirect` | conservar el método y el cuerpo |
| `Redirect::permanent(url)` | `301 Moved Permanently` | una página que se ha movido para siempre |

### Cabeceras y cookies: el builder res

Cuando necesitas fijar varias cosas a la vez, construye una `Response`. El módulo `res` ofrece puntos de entrada al estilo de Express, y todos los métodos se pueden encadenar:

```rust
app.get("/custom", |_| async {
    res::status(202)
        .header("x-version", "1.0")
        .cookie(Cookie::new("seen", "1"))
        .json(json!({ "ok": true }))
});

app.post("/logout", |_| async { res::redirect("/").clear_cookie("session") });
```

| Método | Efecto |
|---|---|
| `res::status(code)` / `.status(code)` | el estado |
| `res::json(v)` / `.json(v)`, `.text(s)`, `.html(s)` | el cuerpo, con su `Content-Type` |
| `.header(name, value)` | fija una cabecera |
| `.append_header(name, value)` | añade un valor más a una cabecera |
| `.cookie(Cookie::new(name, value))` | fija una cookie (`.max_age(...)`, `.path(...)`…) |
| `.clear_cookie(name)` | borra una cookie |
| `res::redirect(url)` | una redirección `302` |

El guard de cookies del escudo añade `Secure`, `HttpOnly` y `SameSite=Lax` a todas las cookies que no los fijan, así que las cookies son seguras aunque se te olvide. Los navegadores aceptan las cookies `Secure` en `http://localhost` mientras desarrollas.

### Archivos

```rust
app.get("/terms", |_| async { res::file("legal/terms.pdf").await });
app.get("/invoice", |_| async { res::download("files/invoice-42.pdf", "factura.pdf").await });
```

`res::file` deduce el `Content-Type` a partir de la extensión, envía los archivos grandes en streaming y responde `404` si el archivo no existe. `res::download` además le pide al navegador que guarde el archivo con el nombre indicado. Para una carpeta entera, usa `public/` o `app.static_dir` (consulta [Enrutamiento](routing.md#archivos-estáticos)).

### Tus propios tipos de respuesta

Implementa `IntoResponse` para devolver directamente tus propios tipos:

```rust
struct Csv(String);

impl IntoResponse for Csv {
    fn into_response(self) -> Response {
        res::send(self.0).content_type("text/csv; charset=utf-8")
    }
}

app.get("/export.csv", |_| async { Csv("id,name\n1,Ada\n".into()) });
```

## Más información

- [vitesse: Leer la petición](https://maxlestage.github.io/Vitesse/es/docs/requests/) y [Responder](https://maxlestage.github.io/Vitesse/es/docs/responses/): todos los métodos, cuerpos en streaming, Server-Sent Events.
- [Errores](errors.md): lo que hace `?` con cada tipo de error.

## Próximos pasos

- [Páginas](pages.md): páginas HTML con etiquetas SEO y un tema.
- [Base de datos](database.md): leer y escribir datos desde tus handlers.
