# Estructura del proyecto

`paquets new my-app` crea un proyecto completo: un servidor con páginas, un modelo y su API REST, una isla, pruebas, un `Dockerfile` de producción, los archivos para Heroku y dos workflows de GitHub. Esta página recorre cada archivo, explica dónde busca archivos el programa en tiempo de ejecución y propone cómo organizar el código a medida que crece.

## Los archivos

```text
my-app/
├── Cargo.toml                  el paquete: dependencias, dos targets, perfiles de compilación
├── src/
│   ├── main.rs                 el servidor: páginas, un modelo, su API REST, pruebas
│   └── lib.rs                  componentes compartidos con el navegador, y las islas
├── public/
│   └── favicon.svg             archivos estáticos, servidos en la raíz del sitio
├── Dockerfile                  la imagen de producción (islas + servidor)
├── heroku.yml                  le indica a Heroku que construya el Dockerfile
├── app.json                    describe la aplicación para el botón «Deploy to Heroku»
├── .github/
│   └── workflows/
│       ├── ci.yml              prueba cada push y luego fusiona la rama en master
│       └── heroku.yml          despliega master en Heroku
├── README.md                   cómo ejecutar, probar y desplegar el proyecto
├── .gitignore                  ignora target/, pkg/ y los archivos *.db
└── .dockerignore               deja target/, pkg/, .git y *.db fuera de las compilaciones de Docker
```

`paquets new` también ejecuta `git init` con una rama `master`, salvo que la carpeta ya esté dentro de un repositorio Git.

Estos aparecen más tarde y nunca se suben al repositorio (están en `.gitignore`):

| Carpeta o archivo | Lo crea | Contiene |
|---|---|---|
| `target/` | Cargo | todo lo que compila Cargo |
| `pkg/` | `paquets dev`, `paquets build` | el bundle del navegador con las islas: `my_app.js` y `my_app_bg.wasm` |
| `*.db` | SQLite | tu base de datos local, cuando `DATABASE_URL=sqlite://...` |

## src/main.rs: el servidor

Para empezar, el servidor es un solo archivo. Contiene, de arriba abajo:

1. `use paquets::prelude::*;`, que trae todo lo que necesita una aplicación, y luego `use my_app::counter;`, la isla definida en `src/lib.rs`;
2. un modelo `Todo` (`#[model]`): una tabla de la base de datos que también viaja como JSON;
3. un struct `NewTodo` (`#[data]`): los datos del formulario «nueva tarea»;
4. `home`, el handler de la página de inicio: lee las tareas de la base de datos y devuelve una `Page` con una cabecera, una sección hero, una tarjeta con la isla del contador, la lista de tareas y un formulario;
5. `add`, el handler del formulario: guarda la tarea y redirige a `/`;
6. `app()`, que crea la `App` y declara las rutas: `GET /`, `POST /todos` y la API REST en `/api/todos`;
7. `main()`, que ejecuta `app()`;
8. un módulo `tests` que comprueba la página de inicio y la API.

Separar `app()` de `main()` es lo que hace que la aplicación se pueda probar: las pruebas llaman a `app().test().await` y obtienen exactamente las mismas rutas, sin abrir ningún puerto. Consulta [Pruebas](testing.md).

## src/lib.rs: compartido con el navegador

`src/lib.rs` se compila dos veces:

- **para el servidor**, como una biblioteca normal que usa `src/main.rs` (`use my_app::counter;`), para que el servidor pueda renderizar las islas a HTML;
- **para el navegador**, a WebAssembly, con `paquets dev` y `paquets build`, para que el navegador pueda hacer interactivas esas islas.

Contiene la isla `counter` y el punto de entrada del navegador:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[cfg(target_arch = "wasm32")]` significa «solo en la compilación para el navegador». Pon en este archivo (o en módulos de la biblioteca) los componentes y las islas que necesita el navegador, y deja el código exclusivo del servidor (handlers, consultas a la base de datos) en `src/main.rs` y sus módulos. Consulta [Islas](islands.md).

## Cargo.toml: un crate, dos targets

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
publish = false

# Un crate, dos targets: el servidor (src/main.rs) y, compiladas a WebAssembly por
# `paquets build`, las islas de src/lib.rs.
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = "0.1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { version = "0.1", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true

# El bundle del navegador: lo más pequeño posible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

| Sección | Por qué |
|---|---|
| `crate-type = ["cdylib", "rlib"]` | `cdylib` produce el archivo `.wasm` para el navegador; `rlib` permite que `src/main.rs` use la biblioteca. |
| Dependencias `cfg(not(target_arch = "wasm32"))` | El lado servidor: Paquets con sus features por defecto (SQLite, PostgreSQL). |
| Dependencias `cfg(target_arch = "wasm32")` | El lado navegador: Paquets sin el servidor ni las bases de datos, con la feature `client`, más `wasm-bindgen`. |
| `[dev-dependencies]` | tokio, para `#[tokio::test]` en las pruebas. Tu aplicación en sí no lo necesita. |
| `[profile.release]` | Optimizaciones para el servidor de producción: optimización en tiempo de enlazado, binario más pequeño. |
| `[profile.wasm-release]` | Lo usa `paquets build` para las islas: optimizado para el tamaño. |

La línea de `paquets` tiene a la vez una `version` y una fuente `git`. Mientras desarrollas, Cargo usa el repositorio de GitHub. Cuando Paquets esté en crates.io, puedes simplificarla a `paquets = "0.1"` (y conservar `default-features = false, features = ["client"]` en la línea de wasm32).

## public/: archivos estáticos

Cada archivo de `public/` se sirve en la raíz del sitio: `public/favicon.svg` es `/favicon.svg`, `public/css/site.css` es `/css/site.css`. Pon ahí imágenes, fuentes, hojas de estilo, `robots.txt` y similares. Los archivos llevan las cabeceras `Content-Type`, `ETag` y `Last-Modified` correctas, y los navegadores reciben `304 Not Modified` cuando ya tienen el archivo.

> [!NOTE]
> Un archivo de `public/` tiene prioridad sobre una ruta con la misma dirección, porque los archivos estáticos se comprueban antes que el enrutamiento. En particular, un archivo `public/index.html` reemplazaría tu ruta `GET /`.

## Los archivos de despliegue

| Archivo | Función | Más información |
|---|---|---|
| `Dockerfile` | Compila las islas y el servidor en modo release y luego copia el programa, `pkg/` y `public/` en una pequeña imagen Debian que se ejecuta con un usuario sin privilegios de root. | [Docker](deploy-docker.md) |
| `heroku.yml` | Le indica a Heroku que construya la imagen a partir del `Dockerfile` (el stack «container» de Heroku). | [Heroku](deploy-heroku-mobile.md) |
| `app.json` | Describe la aplicación para el botón «Deploy to Heroku»: nombre, descripción, stack container. | [Heroku](deploy-heroku-mobile.md#1-la-forma-más-rápida-el-botón-deploy-to-heroku) |
| `.github/workflows/ci.yml` | En cada push: formato, Clippy (servidor y WebAssembly), pruebas. Cuando todo está en verde, la rama se fusiona automáticamente en `master` (salvo las ramas llamadas `wip/...`). | [Automatización](automation.md) |
| `.github/workflows/heroku.yml` | Despliega `master` en Heroku a través de la Platform API de Heroku, configurado desde tu teléfono con un secreto y una variable. | [Heroku](deploy-heroku-mobile.md#4-despliegues-automáticos-con-github-actions) |

## El proyecto mínimo

`paquets new my-app --minimal` crea el mismo proyecto sin islas:

- sin `src/lib.rs`: toda la aplicación está en `src/main.rs`;
- un `Cargo.toml` sencillo, con `paquets` en `[dependencies]` y sin perfil de WebAssembly;
- un `Dockerfile` que solo compila el servidor.

No necesita ninguna herramienta de WebAssembly. Puedes añadir islas más adelante: consulta [Islas](islands.md#configurar-un-proyecto-a-mano).

## Dónde busca archivos el programa

Al arrancar, el servidor busca dos carpetas **en el directorio de trabajo actual** (la carpeta desde la que lo lanzas, que es la raíz del proyecto con `paquets dev` y `cargo run`):

| Carpeta | Si existe |
|---|---|
| `public/` | se sirve en la raíz del sitio |
| `pkg/` | se sirve en `/pkg/`; si contiene un bundle (`<name>.js` y `<name>_bg.wasm`), las páginas con islas lo cargan |

El archivo SQLite de `DATABASE_URL=sqlite://app.db` también es relativo al directorio de trabajo. En la imagen Docker, el directorio de trabajo es `/app`, donde el `Dockerfile` copia `public/` y `pkg/`. Si ejecutas el binario release a mano, lánzalo desde la raíz del proyecto (o copia `public/` y `pkg/` junto al lugar desde el que lo lanzas).

## Hacer crecer el proyecto

Un solo archivo es perfecto para empezar. Cuando `src/main.rs` se alargue, divídelo en módulos. Una organización habitual:

```text
src/
├── main.rs        declaraciones mod, fn app() con las rutas, fn main()
├── models.rs      los structs #[model] y #[data]
├── pages.rs       los handlers que devuelven páginas, y el layout compartido
├── api.rs         rutas JSON propias, o un Router montado en /api
└── lib.rs         las islas (navegador)
```

En `src/main.rs`, declara los módulos y mantén las rutas en un solo sitio:

```rust
mod models;
mod pages;

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/", pages::home);
    app.api::<models::Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Cada módulo empieza con `use paquets::prelude::*;`. Los elementos que usan otros módulos necesitan `pub`:

```rust
// src/models.rs
use paquets::prelude::*;

#[model]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

```rust
// src/pages.rs
use paquets::prelude::*;

use crate::models::Todo;

/// El layout compartido por todas las páginas.
pub fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .body(header().child(a().href("/").text("📦 Mi aplicación")))
        .body(main().child(content))
        .body(footer().text("Hecho con Paquets"))
}

pub async fn home(req: Request) -> Result<Page> {
    let todos = Todo::all(req.db()).await?;
    Ok(layout(
        "Inicio",
        ul().children(todos.into_iter().map(|todo| li().text(todo.title))),
    ))
}
```

En `pages.rs` no hay `fn main`, así que la etiqueta `<main>` es simplemente `main()`.

## Próximos pasos

- [Enrutamiento](routing.md): rutas, parámetros, routers y middleware.
- [Páginas](pages.md): vistas, layouts, el tema y el SEO.
- [Herramienta de línea de comandos](cli.md): lo que hacen `paquets dev` y `paquets build`.
