<div align="center">

# 📦 Paquets

**El framework full-stack más sencillo para Rust.**

Servidor, seguridad, base de datos, API REST, páginas optimizadas para SEO e islas WebAssembly: una crate, un import, una `App`.

[![CI](https://github.com/maxlestage/Paquets/actions/workflows/ci.yml/badge.svg)](https://github.com/maxlestage/Paquets/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/paquets.svg)](https://crates.io/crates/paquets)
[![docs.rs](https://img.shields.io/docsrs/paquets)](https://docs.rs/paquets)
[![Licencia: MIT O Apache-2.0](https://img.shields.io/badge/licencia-MIT%20O%20Apache--2.0-blue.svg)](#licencia)

[English](README.md) · [Français](README.fr.md) · **Español**

[Sitio web y documentación](https://maxlestage.github.io/Paquets/) · [Desplegar en Heroku desde el móvil](#desplegar-desde-el-móvil)

</div>

---

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async {
        Page::new("Inicio").body(h1().text("Hola 👋"))
    });
    app.api::<Todo>("/api/todos"); // GET, POST, PUT, PATCH, DELETE

    app.run() // http://localhost:3000, o $PORT en Heroku
}
```

Es una aplicación web completa: una página renderizada en el servidor con sus etiquetas SEO, una
API REST guardada en una base de datos y un escudo de seguridad delante de cada petición. Nada que
configurar.

## Por qué Paquets

| | |
|---|---|
| ✨ **Sencillo** | `use paquets::prelude::*;`, `App::new()`, `app.run()`. Los handlers son funciones `async`; `?` funciona con cualquier error y elige el código HTTP correcto. |
| 🛡️ **Seguro por defecto** | WAF (inyección SQL, XSS, RCE, Log4Shell…), límite de peticiones, CSRF, cabeceras de seguridad, identificadores de petición, bloqueo de bots de ataque. Las páginas envían una Content-Security-Policy que solo permite su propio código. |
| 🗄️ **Cualquier base de datos** | En memoria, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB, elegida con `DATABASE_URL`. `#[model]` te da consultas tipadas y una API REST en una línea. |
| 🔎 **Páginas SEO** | Las páginas son HTML terminado en el servidor: título, descripción, Open Graph, Twitter, URL canónica, JSON-LD. Un tema integrado (claro y oscuro) las hace bonitas sin CSS. |
| 📲 **Instalable (PWA)** | `app.pwa(Pwa::new("Mi app"))`: un manifiesto de aplicación web y un service worker sin conexión, para que la aplicación vaya a la pantalla de inicio y funcione sin red. |
| 🏝️ **Islas** | Las partes interactivas son componentes Rust compilados a WebAssembly, y solo ellas: el resto de la página no envía código. |
| ⚡ **Rápido** | Construido sobre un servidor más rápido que actix-web y axum, una cadena de seguridad de unos 6 µs y SQL generado al compilar. |
| 📱 **Despliegue desde el móvil** | Se generan el Dockerfile, `heroku.yml`, `app.json` y las GitHub Actions: basta un toque en *Deploy to Heroku*. |

## Cuatro paquetes, un framework

Paquets reúne cuatro crates, cada una utilizable por separado:

| Paquete | Papel | En Paquets |
|---|---|---|
| [**vitesse**](https://github.com/maxlestage/Vitesse) | Servidor HTTP al estilo de Express: rutas, middlewares, routers, WebSocket, HTTP/3 | `App` (se desreferencia a `vitesse::App`), `Request`, `Json`, `Router`… |
| [**securite**](https://github.com/maxlestage/Securite) | Middlewares de seguridad: WAF, límite de peticiones, CSRF, CORS, cabeceras y CSP, autenticación… | delante de cada petición, `app.security(...)` |
| [**memoire-orm**](https://github.com/maxlestage/Memoire) | ORM asíncrono tipado, SQL generado al compilar, SQL y NoSQL | `#[model]`, `req.db()`, `app.api::<M>()` |
| [**active**](https://github.com/maxlestage/Active) | Front-end: componentes, estado, renderizado en servidor, SEO, islas WebAssembly | `Page`, `div()`, `h1()`, `island(...)`, `use_state` |

## Empezar

```sh
cargo install paquets --locked    # el comando `paquets`
paquets new mi-app                # un proyecto completo, listo para desplegar
cd mi-app
paquets dev                       # http://localhost:3000
```

Hasta la primera publicación en crates.io, instala el comando desde GitHub:
`cargo install --git https://github.com/maxlestage/Paquets paquets --locked`.

`paquets new` crea el servidor (`src/main.rs`: páginas, un modelo y su API), los componentes del
navegador (`src/lib.rs`: una isla), un `Dockerfile`, los archivos de Heroku y unas GitHub Actions
que prueban cada push y **fusionan automáticamente cada rama en verde con `master`**.
`paquets build` hace la compilación de producción y `paquets doctor` comprueba tus herramientas.

O añade Paquets a un proyecto existente:

```toml
[dependencies]
paquets = { version = "0.1", git = "https://github.com/maxlestage/Paquets" }
```

## Un recorrido

```rust
use paquets::prelude::*;

#[model]                         // una tabla, y JSON de entrada y de salida
struct Todo {
    id: i64,                     // clave primaria, generada por la base de datos
    title: String,
    done: bool,
}

/// Una página, renderizada en el servidor.
async fn inicio(req: Request) -> Result<Page> {
    let todos = Todo::query()
        .filter(Todo::DONE.eq(false))
        .order_by(Todo::ID.asc())
        .all(req.db())
        .await?;
    Ok(Page::new("Pendientes")
        .lang("es")
        .description("Lo que queda por hacer.")
        .body(h1().text("Pendientes"))
        .body(ul().children(todos.into_iter().map(|t| li().text(t.title)))))
}

/// JSON, con errores que se convierten en el código correcto.
async fn detalle(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si `id` no es un número
    let todo = Todo::get(req.db(), id).await?; // 404 si la fila no existe
    Ok(Json(todo))
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", inicio);
    app.get("/todos/:id", detalle);
    app.api::<Todo>("/api/todos");
    app
}

#[tokio::test]
async fn funciona() {
    let client = app().test().await;            // sin puerto, base en memoria
    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Probar Paquets", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    assert!(client.get("/").await.text().contains("Probar Paquets"));
}
```

## Documentación

Todo está explicado en el [sitio web](https://maxlestage.github.io/Paquets/#/docs) y en
[`docs/`](docs/es/README.md), en español, [inglés](docs/en/README.md) y [francés](docs/fr/README.md):

| Primeros pasos | Lo esencial | Más allá | Desplegar | Referencia |
|---|---|---|---|---|
| [Introducción](docs/es/introduction.md) | [Rutas](docs/es/routing.md) | [Islas](docs/es/islands.md) | [Heroku desde el móvil](docs/es/deploy-heroku-mobile.md) | [Línea de comandos](docs/es/cli.md) |
| [Instalación](docs/es/installation.md) | [Peticiones y respuestas](docs/es/requests-responses.md) | [WebSocket](docs/es/websocket.md) | [Docker](docs/es/deploy-docker.md) | [Chuleta](docs/es/cheatsheet.md) |
| [Inicio rápido](docs/es/quick-start.md) | [Páginas](docs/es/pages.md) | [Pruebas](docs/es/testing.md) | [Publicar en crates.io](docs/es/publish-crates-io.md) | [Los cuatro paquetes](docs/es/packages.md) |
| [Estructura del proyecto](docs/es/project-structure.md) | [Base de datos](docs/es/database.md) | [Errores](docs/es/errors.md) | [Automatización](docs/es/automation.md) | [Preguntas frecuentes](docs/es/faq.md) |
| | [API REST](docs/es/rest-api.md) · [Seguridad](docs/es/security.md) | [Configuración](docs/es/configuration.md) · [Rendimiento](docs/es/performance.md) | | |

Referencia de la API: [docs.rs/paquets](https://docs.rs/paquets).

## Desplegar desde el móvil

[![Deploy to Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets)

1. Toca **Deploy to Heroku**, elige un nombre de aplicación y toca **Deploy app**. Heroku construye
   el [`Dockerfile`](Dockerfile): el sitio web, que es en sí una aplicación Paquets, con su demo en
   vivo (`/demo`, `/api/todos`).
2. Despliegues automáticos de `master`: en GitHub, *Settings → Secrets and variables → Actions*,
   añade el secreto `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key*) y la variable
   `HEROKU_APP_NAME`. [`heroku.yml`](.github/workflows/heroku.yml) despliega entonces cada fusión.
3. Una base de datos: Heroku → tu aplicación → *Resources* → *Heroku Postgres*. Su `DATABASE_URL`
   se usa automáticamente.

Heroku no tiene plan gratuito (un dyno Eco cuesta unos 5 $ al mes). Los proyectos creados con
`paquets new` contienen los mismos archivos. Guía paso a paso:
[docs/es/deploy-heroku-mobile.md](docs/es/deploy-heroku-mobile.md).

El sitio web también se publica gratis en **GitHub Pages**, en
<https://maxlestage.github.io/Paquets/> ([`pages.yml`](.github/workflows/pages.yml); una sola vez:
*Settings → Pages → Source: GitHub Actions*).

## Siempre fusionado en master

Cada push lanza la [CI](.github/workflows/ci.yml): formato, clippy (servidor, navegador y todas las
features), pruebas, documentación, versión mínima de Rust, los proyectos generados por
`paquets new` y el sitio web. Cuando todo está en verde, la rama se **fusiona automáticamente con
`master`** (salvo las ramas llamadas `wip/...`) y después se despliegan el sitio y la aplicación de Heroku.

## Publicar en crates.io (desde el móvil)

1. crates.io → *Account Settings* → *API Tokens* → un token con los permisos *publish-new* y
   *publish-update*; añádelo en GitHub como secreto `CARGO_REGISTRY_TOKEN`.
2. GitHub → *Releases* → *Draft a new release* → etiqueta `v0.1.0` → *Publish release*.

[`release.yml`](.github/workflows/release.yml) publica `paquets-macros` y después `paquets`.
crates.io solo acepta dependencias publicadas en crates.io, así que vitesse, securite, memoire-orm
y active se publican antes, cada una desde su repositorio y de la misma manera; el workflow lo
comprueba. Guía: [docs/es/publish-crates-io.md](docs/es/publish-crates-io.md).

## Desarrollo

```sh
cargo test --workspace                     # la crate, el comando, los ejemplos
cargo run --example todo                   # la aplicación de demo en http://localhost:3000
cargo run -- new mi-app --path .           # el comando, con este repositorio de Paquets
cd site && cargo run                       # el sitio web, que es una aplicación Paquets (ver site/README.md)
```

| Carpeta | Contenido |
|---|---|
| [`src/`](src) | la crate `paquets` (`app.rs`, `db.rs`, `api.rs`, `page.rs`, `shield.rs`, `error.rs`) y el comando (`main.rs`, `cli/`) |
| [`macros/`](macros) | `#[model]` y `#[data]` (`paquets-macros`) |
| [`examples/`](examples) | `hello` y `todo` (la aplicación desplegada en Heroku) |
| [`docs/`](docs) | la documentación, en inglés, francés y español |
| [`site/`](site) | el sitio web: una aplicación Paquets (páginas renderizadas por active, islas en WebAssembly), exportada a archivos estáticos para GitHub Pages |

## Licencia

A tu elección, bajo la [licencia Apache 2.0](LICENSE-APACHE) o la [licencia MIT](LICENSE-MIT). Salvo
que indiques expresamente lo contrario, cualquier contribución que envíes intencionadamente para
su inclusión en este proyecto, tal como se define en la licencia Apache-2.0, se distribuirá bajo
esta doble licencia, sin términos ni condiciones adicionales.
