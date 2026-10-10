# Pruebas

`app.test().await` convierte tu aplicación en un cliente de pruebas: las peticiones van directamente a la aplicación, pasando por el escudo, el middleware y tus handlers, sin abrir ningún puerto. La base de datos está en memoria y vacía en cada prueba, así que las pruebas son rápidas, independientes y se pueden ejecutar en paralelo. Esta página muestra cómo escribirlas.

## Preparación

Las pruebas son funciones `async` marcadas con `#[tokio::test]`. Necesitan tokio como dependencia de desarrollo, que los proyectos creados con `paquets new` ya tienen:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Tu aplicación en sí no necesita tokio: `App::run()` gestiona su propio runtime.

## Haz que la aplicación se pueda probar

Las pruebas deben construir la misma aplicación que `main`, sin arrancar el servidor. Pon la aplicación en una función:

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
    app.get("/", |_| async { Page::new("Inicio").body(h1().text("Hola")) });
    app.api::<Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

## Tu primera prueba

Añade un módulo `tests` al final de `src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_page() {
        let client = app().test().await;

        let res = client.get("/").await;

        assert_eq!(res.status(), 200);
        assert!(res.text().contains("<h1>Hola</h1>"));
    }
}
```

Ejecútala:

```sh
cargo test                 # todas las pruebas
cargo test home_page       # las pruebas cuyo nombre contiene "home_page"
cargo test -- --nocapture  # muestra también lo que imprimen las pruebas
```

`app().test().await` prepara la aplicación exactamente como lo haría `run()`: abre la base de datos, crea las tablas, ejecuta los hooks `on_start` y conecta el escudo y los archivos estáticos. Solo quedan fuera el log de peticiones y la red.

## Enviar peticiones

Empieza una petición desde el cliente, encadena lo que necesites y envíala con `.await`:

| Método | Petición |
|---|---|
| `client.get(uri)` | `GET` |
| `client.post(uri)` | `POST` |
| `client.put(uri)` | `PUT` |
| `client.patch(uri)` | `PATCH` |
| `client.delete(uri)` | `DELETE` |
| `client.request(Method::OPTIONS, uri)` | cualquier otro método |

| Después | Efecto |
|---|---|
| `.header(name, value)` | añade una cabecera |
| `.json(&value)` | un cuerpo JSON, con `content-type: application/json` |
| `.form(&value)` | un cuerpo de formulario, como `&[("email", "ada@example.com")]` |
| `.body(data)` | un cuerpo en bruto |

```rust
let res = client
    .post("/api/todos")
    .json(&json!({ "title": "Escribir pruebas", "done": false }))
    .await;

let res = client.get("/search?q=caf%C3%A9").await; // la query string forma parte del URI, codificada
let res = client.post("/login").form(&[("email", "ada@example.com"), ("password", "secreto")]).await;
```

## Leer las respuestas

| Método | Devuelve |
|---|---|
| `res.status()` | el estado, comparable con un número: `assert_eq!(res.status(), 404)` |
| `res.header(name)` | una cabecera, como `Option<&str>` |
| `res.headers()` | todas las cabeceras |
| `res.text()` | el cuerpo como `String` |
| `res.json::<T>()` | el cuerpo decodificado desde JSON (provoca un pánico, mostrando el cuerpo, si no es válido) |
| `res.bytes()` | el cuerpo en bruto |

```rust
#[tokio::test]
async fn creates_a_todo() {
    let client = app().test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Escribir pruebas", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    let todo: Todo = res.json(); // tipado
    assert_eq!(todo.title, "Escribir pruebas");

    let body: paquets::serde_json::Value = client.get("/api/todos/1").await.json(); // sin tipar
    assert_eq!(body["done"], false);
}
```

## La base de datos en las pruebas

Cada llamada a `app().test()` obtiene una base de datos en memoria nueva y vacía, diga lo que diga `DATABASE_URL`. Las pruebas nunca ven los datos de las demás, ni los tuyos.

Para partir de datos conocidos, añade un hook `on_start` en la prueba:

```rust
#[tokio::test]
async fn lists_are_paginated() {
    let mut app = app();
    app.on_start(|db| async move {
        for title in ["Una", "Dos", "Tres"] {
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

Para probar con un motor SQL real (para SQL sin procesar o migraciones solo SQL), elígelo en la prueba. Una base de datos SQLite en memoria se sigue creando nueva para cada prueba:

```rust
let mut app = app();
app.database("sqlite::memory:");
let client = app.test().await;
```

## Probar páginas

Una página es HTML: comprueba el estado, el tipo de contenido y el texto que esperas.

```rust
#[tokio::test]
async fn the_home_page_has_its_seo_tags() {
    let client = app().test().await;
    let res = client.get("/").await;

    assert_eq!(res.status(), 200);
    assert!(res.header("content-type").unwrap().starts_with("text/html"));
    let html = res.text();
    assert!(html.contains("<title>Inicio</title>"));
    assert!(html.contains("<h1>Hola</h1>"));
}
```

## Probar la seguridad

El escudo también se ejecuta en las pruebas, así que puedes comprobar que los ataques se rechazan:

```rust
#[tokio::test]
async fn attacks_are_blocked() {
    let client = app().test().await;

    // Inyección SQL en la query string: la bloquea el cortafuegos.
    let res = client.get("/?q=1%27%20OR%20%271%27=%271").await;
    assert_eq!(res.status(), 403);

    // Una herramienta de ataque conocida: la bloquea el bloqueador de bots.
    let res = client.get("/").header("user-agent", "sqlmap/1.7").await;
    assert_eq!(res.status(), 403);

    // Una petición enviada desde otro sitio web: la bloquea el guard CSRF.
    let res = client
        .post("/api/todos")
        .header("sec-fetch-site", "cross-site")
        .json(&json!({ "title": "x", "done": false }))
        .await;
    assert_eq!(res.status(), 403);
}
```

> [!WARNING]
> El límite de tasa también se aplica en las pruebas: todas las peticiones de una prueba vienen del mismo cliente, así que después de 100 peticiones seguidas las siguientes reciben `429 Too Many Requests`. Para una prueba que envía muchas peticiones, quita el límite de tasa en esa prueba:
>
> ```rust
> let mut app = app();
> app.security(paquets::shield().without("rate_limit").build());
> let client = app.test().await;
> ```

## Probar los errores y el middleware

Los errores son respuestas normales: comprueba el estado y el mensaje.

```rust
#[tokio::test]
async fn bad_input_is_refused() {
    let client = app().test().await;

    let res = client.post("/api/todos").body("{oops").await; // no es JSON
    assert_eq!(res.status(), 400);

    assert_eq!(client.get("/api/todos/abc").await.status(), 400); // no es un número
    assert_eq!(client.get("/api/todos/999").await.status(), 404); // no existe esa fila
}
```

Para un middleware, comprueba los dos caminos: la petición que deja pasar y la que detiene (por ejemplo, con y sin la cabecera `Authorization`).

## Pruebas en la carpeta tests

El módulo `tests` de `src/main.rs` es el sitio más sencillo para las pruebas. Si prefieres pruebas de integración en una carpeta `tests/`, estas solo pueden usar tu biblioteca (`src/lib.rs`), no `src/main.rs`. Mueve `app()` a la biblioteca y déjala fuera de la compilación del navegador cuando el proyecto tenga islas:

```rust
// src/lib.rs
#[cfg(not(target_arch = "wasm32"))]
pub fn app() -> paquets::App {
    let mut app = paquets::App::new();
    // ... rutas ...
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

Los modelos que usa `app()` también pasan entonces a la biblioteca, detrás del mismo `#[cfg(not(target_arch = "wasm32"))]`.

## Lo que el cliente de pruebas no cubre

- **La red**: las cabeceras que escribe el motor HTTP (`content-length`, `date`) no aparecen en las respuestas de las pruebas.
- **Las conversaciones WebSocket**: el cliente de pruebas solo ve el handshake. Consulta [WebSocket](websocket.md#pruebas) para probar en un puerto real.
- **El lado navegador de las islas**: prueba el HTML de una isla generado por el servidor con el cliente de pruebas, y su lógica llamando directamente a tus funciones de Rust.

## En integración continua

El workflow `ci.yml` de un proyecto generado ejecuta `cargo test` en cada push, junto con `cargo fmt --check` y Clippy, y fusiona la rama en `master` solo cuando todo está en verde. Consulta [Automatización](automation.md).

## Próximos pasos

- [Errores](errors.md): cómo es cada error, para probarlo.
- [Automatización](automation.md): pruebas en cada push.
- [La guía de pruebas de vitesse](https://github.com/maxlestage/Vitesse/blob/master/docs/es/testing.md): más ejemplos con el mismo cliente de pruebas.
