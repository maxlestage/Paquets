# Generadores

`paquets make:*` escribe el código que escribirías a mano — un modelo, su API REST, una página, una migración, una prueba, la autenticación, un panel de administración — y lo conecta en `src/main.rs`, como Artisan de Laravel o los generadores de Rails. El código generado es Rust corriente e idiomático: léelo, cámbialo, es tuyo.

## Los generadores

| Comando | Escribe | Conecta en `fn app()` |
|---|---|---|
| `paquets make:model Post title:string body:text` | `src/models/post.rs`, `src/models.rs` | `app.model::<models::Post>();` |
| `paquets make:api Post title:string` | el modelo, con una prueba de su API | `app.api::<models::Post>("/api/posts");` |
| `paquets make:page about` | `src/pages/about.rs` (con una prueba), `src/pages.rs` | `app.get("/about", pages::about::about);` |
| `paquets make:migration add_slug` | `src/migrations/m<fecha>_add_slug.rs`, `src/migrations.rs` | `app.migrate(migrations::all());` |
| `paquets make:test checkout` | `src/feature_tests/checkout.rs` | el módulo, compilado solo por `cargo test` |
| `paquets make:auth` | `Cargo.toml` (Argon2 optimizado también en debug) | `app.auth(paquets::auth::Auth::new());` |
| `paquets make:admin Post Comment` | — | `app.admin(Admin::new().model::<models::Post>()…);` |

`make:model` acepta también `--api` (como `make:api`) y `--admin` (añade el modelo al [panel de administración](admin.md)). Cada generador declara los módulos que necesita (`mod models;`…) y pasa `rustfmt` sobre el resultado: un proyecto creado con `paquets new` sigue pasando `cargo fmt --check` y `cargo clippy -- -D warnings`, y la CI lo comprueba.

Ejecútalos en la carpeta del proyecto. Nunca sobrescriben un archivo: generar dos veces el mismo modelo es un error.

## Modelos y campos

```sh
paquets make:api Post title:string body:text published:bool "views:i64?" slug:string:unique --admin
```

escribe `src/models/post.rs`:

```rust
//! The `Post` model: a table of the database (`posts`), also sent and
//! received as JSON.

use paquets::prelude::*;

/// Post.
#[model(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub views: Option<i64>,
    #[memoire(unique)]
    pub slug: String,
}
```

y una prueba que crea una fila con `POST /api/posts` y la lista. Los campos se escriben `nombre:tipo`, y se añade primero una clave `id: i64`:

| Tipo | Rust |
|---|---|
| `string`, `text` | `String` |
| `i64`, `int`, `integer` | `i64` |
| `i32` | `i32` |
| `f64`, `float`, `number` | `f64` |
| `bool`, `boolean` | `bool` |

Un `?` después del tipo hace el campo opcional (`Option<T>`, entre comillas en la shell: `"views:i64?"`), y `:unique` añade una restricción de unicidad. Los nombres pasan a `snake_case` (`BlogPost` y `blog-post` dan `blog_post`), el tipo a `PascalCase`, la tabla y la ruta de la API al plural (`posts`, `categories`).

## Páginas, migraciones y pruebas

`paquets make:page pricing` escribe una página renderizada en el servidor, en la ruta `/pricing`, con una prueba que la pide:

```rust
/// `GET /pricing`, rendered on the server with its SEO tags.
pub async fn pricing(_req: Request) -> Page {
    Page::new("Pricing")
        .description("Pricing.")
        .body(
            html::main()
                .class("stack")
                .child(h1().text("Pricing"))
                .child(p().text("Edit src/pages/pricing.rs.")),
        )
}
```

`paquets make:migration add_slug_to_posts` añade una migración versionada (su versión es la hora UTC) a la lista de `src/migrations.rs`; describe sus cambios con el `Migration` de memoire (`.create_table::<M>()`, `.sql(…)`, `.sql_for(dialecto, …)`). Ver [Operaciones](operations.md#migraciones-y-datos-de-ejemplo).

`paquets make:test checkout` escribe una prueba funcional que recorre toda la aplicación, en memoria:

```rust
#[tokio::test]
async fn checkout() {
    let client = crate::app().test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
}
```

Las pruebas y las páginas usan la función `fn app() -> App` que `paquets new` escribe en `src/main.rs`; sin ella, los generadores indican la línea que debes añadir tú.

## Autenticación y administración

`paquets make:auth` activa la [autenticación](authentication.md): registro, inicio de sesión, restablecimiento de contraseña y códigos de dos factores. `paquets make:admin Post Comment` añade modelos al [panel de administración](admin.md) (y lo crea si hace falta).

## Próximos pasos

- [Autenticación](authentication.md): las páginas y la protección de las rutas.
- [Panel de administración](admin.md): gestionar tus modelos desde el navegador.
- [Operaciones](operations.md): documentación de la API, lista de rutas, migraciones, salud y métricas, recarga automática.
