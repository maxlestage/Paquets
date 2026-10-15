<div align="center">

[![Paquets: el framework full-stack más sencillo para Rust](https://raw.githubusercontent.com/maxlestage/Paquets/master/site/public/assets/og-image.png)](https://paquets-90dfc06b6fea.herokuapp.com/)

# 📦 Paquets

**El framework full-stack más sencillo para Rust.**

Servidor, seguridad, base de datos, API REST, páginas optimizadas para SEO e islas WebAssembly: una crate, un import, una `App`.

[![crates.io](https://img.shields.io/crates/v/paquets.svg)](https://crates.io/crates/paquets)
[![docs.rs](https://img.shields.io/docsrs/paquets)](https://docs.rs/paquets)
[![Licencia: propietaria](https://img.shields.io/badge/license-proprietary-2a2a2e.svg)](#licencia)

[English](README.md) · [Français](README.fr.md) · **Español**

[Sitio web y documentación](https://paquets-90dfc06b6fea.herokuapp.com/) · [Desplegar en Heroku desde el móvil](#desplegar-desde-el-móvil)

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
| [**Vitesse**](https://vitesse-f16135654c3d.herokuapp.com/) | Servidor HTTP al estilo de Express: rutas, middlewares, routers, WebSocket, sesiones, HTTPS, HTTP/2 y HTTP/3 | `App` (se desreferencia a `vitesse::App`), `Request`, `Json`, `Router`… |
| [**Securite**](https://securite-2db4c1994be7.herokuapp.com/) | Middlewares de seguridad: WAF, límite de peticiones, CSRF, CORS, cabeceras y CSP, autenticación… | delante de cada petición, `app.security(...)` |
| [**memoire-orm**](https://memoire-7f4525f1fdaf.herokuapp.com/) | ORM asíncrono tipado, SQL generado al compilar, SQL y NoSQL, relaciones y búsqueda de texto completo | `#[model]`, `req.db()`, `app.api::<M>()` |
| [**active**](https://active-c1883a966650.herokuapp.com/) | Front-end: componentes, estado, renderizado en servidor, SEO, islas WebAssembly | `Page`, `div()`, `h1()`, `island(...)`, `use_state` |

## Empezar

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked    # el comando `paquets`
paquets new mi-app                # un proyecto completo, listo para desplegar
cd mi-app
paquets dev                       # http://localhost:3000
```

`paquets new` crea el servidor (`src/main.rs`: páginas, un modelo y su API), los componentes del
navegador (`src/lib.rs`: una isla), un `Dockerfile`, los archivos de Heroku y unas GitHub Actions
que prueban cada push y **fusionan automáticamente cada rama en verde con `master`**.
`paquets build` hace la compilación de producción y `paquets doctor` comprueba tus herramientas.

O añade Paquets a un proyecto existente:

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }
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

Todo está explicado en el [sitio web](https://paquets-90dfc06b6fea.herokuapp.com/es/docs/) y en
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

1. Abre `https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets` en tu móvil, elige
   un nombre de aplicación y toca **Deploy app**. Heroku lee [`app.json`](app.json) y construye, con
   el [buildpack emk/rust](https://elements.heroku.com/buildpacks/emk/heroku-buildpack-rust) en el
   stack heroku-26 ([`RustConfig`](RustConfig), [`heroku-build.sh`](heroku-build.sh),
   [`Procfile`](Procfile)), el sitio web, que es en sí una aplicación Paquets, con su demo en vivo
   (`/demo`, `/api/todos`).
2. Despliegues automáticos de `master`: en GitHub, *Settings → Secrets and variables → Actions*,
   añade el secreto `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key*) y la variable
   `HEROKU_APP_NAME`. [`heroku.yml`](.github/workflows/heroku.yml) despliega entonces cada fusión.
   Con la variable opcional `HEROKU_BUILD` a `docker`, construye el [`Dockerfile`](Dockerfile) en su
   lugar ([`heroku.yml`](heroku.yml), stack «container»).
3. Una base de datos: Heroku → tu aplicación → *Resources* → *Heroku Postgres*. Su `DATABASE_URL`
   se usa automáticamente.

Sin GitHub Actions: `heroku create --stack heroku-26 --buildpack emk/rust` (en una aplicación
existente, `heroku stack:set heroku-26` y después `heroku buildpacks:set emk/rust`), y después
`git push heroku master`; o, en el panel, *Settings → Buildpacks → Add buildpack* → `emk/rust`
(el stack no debe ser «container»).

Heroku no tiene plan gratuito (un dyno Eco cuesta unos 5 $ al mes). Los proyectos creados con
`paquets new` se despliegan igual, construidos a partir de su `Dockerfile`. Guía paso a paso:
[docs/es/deploy-heroku-mobile.md](docs/es/deploy-heroku-mobile.md).

## Siempre fusionado en master

Cada push lanza la [CI](.github/workflows/ci.yml): formato, clippy (servidor, navegador y todas las
features), pruebas, documentación, versión mínima de Rust, los proyectos generados por
`paquets new` y el sitio web. Cuando todo está en verde, la rama se **fusiona automáticamente con
`master`** (salvo las ramas llamadas `wip/...`) y después se despliega la aplicación de Heroku, que sirve el sitio web.

## Publicar en crates.io (desde el móvil)

1. crates.io → *Account Settings* → *API Tokens* → un token con los permisos *publish-new* y
   *publish-update*; añádelo en GitHub como secreto `CARGO_REGISTRY_TOKEN`.
2. Cambia `version` en `Cargo.toml` y `macros/Cargo.toml`, escribe su sección en `CHANGELOG.md`,
   haz push: cuando la CI está en verde, la versión se etiqueta (`v0.1.1`) y se publica sola, sin rellenar nada.

[`release.yml`](.github/workflows/release.yml) publica `paquets-macros` y después `paquets`.
crates.io solo acepta dependencias publicadas en crates.io, así que vitesse, Securite, memoire-orm
y Active se publican antes, cada una desde su repositorio y de la misma manera; el workflow lo
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
| [`site/`](site) | el sitio web: una aplicación Paquets (páginas renderizadas por Active, islas en WebAssembly), servida por Heroku |

## Licencia

Paquets es software propietario: © 2026 Maxime Nathan Lestage, todos los
derechos reservados. No es software de código abierto. El código fuente se
publica en crates.io y GitHub para que pueda leerse, auditarse y compilarse,
y puedes usarlo libremente como dependencia de tus propios programas, también
comerciales. Modificarlo o redistribuir su código fuente requiere el permiso
escrito del autor. Condiciones completas: [LICENSE](LICENSE).

Las contribuciones (código, documentación, ideas) son bienvenidas y pasan a
formar parte del proyecto en los mismos términos: al enviar una, cedes sus
derechos al autor, como indica la licencia.

## Créditos

Diseñado y desarrollado por Maxime Nathan Lestage,
dentro de una misma familia de proyectos Rust: [Active](https://active-c1883a966650.herokuapp.com/), [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/), [Securite](https://securite-2db4c1994be7.herokuapp.com/), [Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/) y [Paquets](https://paquets-90dfc06b6fea.herokuapp.com/).
