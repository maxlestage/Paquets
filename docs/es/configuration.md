# Configuración

Una aplicación Paquets funciona sin configuración. Cuando necesitas cambiar algo, hay tres sitios: las variables de entorno (que define tu proveedor de alojamiento, o tú), los métodos de `App` y las features de Cargo. Esta página los enumera todos, muestra cómo leer tus propios ajustes y compartir estado entre peticiones, y explica cómo funciona el servidor.

## Variables de entorno

| Variable | Sirve para | Por defecto |
|---|---|---|
| `PORT` | el puerto en el que escucha el servidor | `3000` |
| `DATABASE_URL` | la base de datos (consulta [Base de datos](database.md#elegir-la-base-de-datos)) | en memoria |
| `DYNO` | la define Heroku; el escudo por defecto lee entonces la IP del cliente de `X-Forwarded-For` | — |

Defínelas en tu shell para una sola ejecución:

```sh
PORT=8080 DATABASE_URL=sqlite://app.db paquets dev
```

En Heroku, `PORT` se define por ti, y `DATABASE_URL` también cuando añades el add-on Heroku Postgres. Tus propias variables van en **Settings → Config Vars** (consulta [Desplegar en Heroku](deploy-heroku-mobile.md#ajustes-config-vars)). Con Docker, pásalas con `-e NAME=value` (consulta [Docker](deploy-docker.md)).

Paquets no lee archivos `.env`. Si te gustan, el crate [`dotenvy`](https://crates.io/crates/dotenvy) carga uno con una sola llamada al principio de `main`.

## Ajustes de la aplicación

`App::new()` te da una aplicación lista; estos métodos la modifican. Llámalos antes de `app.run()`, en cualquier orden.

| Método | Efecto | Por defecto |
|---|---|---|
| `app.port(8080)` | el puerto, en lugar de `PORT` | `PORT`, y si no `3000` |
| `app.quiet()` | desactiva el log de peticiones | una línea por petición |
| `app.pwa(manifiesto)` | aplicación instalable que funciona sin conexión (ver [Páginas](pages.md#aplicación-instalable-pwa)) | no instalable |
| `app.database(url_or_db)` | la base de datos, en lugar de `DATABASE_URL` | `DATABASE_URL`, y si no en memoria |
| `app.security(shield)` | otro escudo de seguridad, o `None` para desactivarlo | `paquets::shield()` |
| `app.on_start(hook)` | código que se ejecuta cuando la base de datos está lista | — |
| `app.api::<M>(path)` | una API REST y una tabla para un modelo | — |
| `app.model::<M>()` | una tabla para un modelo | — |

`App` también da acceso a todos los métodos de la aplicación de vitesse:

| Método | Efecto | Por defecto |
|---|---|---|
| `app.state(value)` | estado compartido, que se lee con `req.state::<T>()` (ver más abajo) | — |
| `app.body_limit(bytes)` | el cuerpo más grande que puede leer un handler | 2 MiB, como el límite de cuerpo del escudo |
| `app.middleware(m)` | un middleware global (consulta [Enrutamiento](routing.md#middleware)) | — |
| `app.mount(prefix, router)` | un router bajo un prefijo | — |
| `app.static_dir(prefix, dir)` | sirve una carpeta bajo un prefijo | `public/` en `/` |
| `app.fallback(handler)` | la respuesta cuando ninguna ruta coincide | `404` en JSON |
| `app.on_error(f)` | el formato de todas las respuestas de error (consulta [Errores](errors.md#cambiar-el-formato-de-todos-los-errores)) | `{"error": "..."}` |
| `app.workers(n)` | el número de hilos del servidor (ver más abajo) | uno por núcleo de CPU |
| `app.thread_per_core(false)` | el runtime de tokio con work-stealing en lugar de un bucle de eventos por núcleo (ver más abajo) | `true` en Linux |

Los métodos que devuelven `&mut App` se pueden encadenar: `app.port(8080).quiet();`. Solo `run()`, `serve()` y `test()` consumen la aplicación, así que van al final.

## Tus propios ajustes

Lee tus ajustes una sola vez, al arrancar, en un struct, y guarda ese struct en el estado de la aplicación:

```rust
use paquets::prelude::*;

/// Los ajustes de la aplicación, leídos una vez al arrancar.
struct Settings {
    site_name: String,
    admin_email: String,
}

impl Settings {
    fn from_env() -> Settings {
        Settings {
            site_name: std::env::var("SITE_NAME").unwrap_or_else(|_| "Mi sitio".into()),
            admin_email: std::env::var("ADMIN_EMAIL").unwrap_or_default(),
        }
    }
}

fn app() -> App {
    let mut app = App::new();
    app.state(Settings::from_env());
    app.get("/", |req: Request| async move {
        let settings = req.state::<Settings>();
        Page::new(settings.site_name.clone())
            .body(h1().text(format!("Te damos la bienvenida a {}", settings.site_name)))
    });
    app
}
```

Guarda los secretos (claves de API, contraseñas) en variables de entorno, nunca en el código ni en Git.

## Estado compartido

`app.state(value)` registra un valor que cada petición puede leer con `req.state::<T>()`: ajustes, una caché, un contador, un canal (consulta la sala de chat en [WebSocket](websocket.md#una-sala-de-chat)).

- El estado se encuentra **por tipo**: un valor por tipo. Da a cada elemento del estado su propio tipo (un struct), en lugar de registrar un `String` sin más.
- El tipo debe ser `Send + Sync + 'static`, porque todos los hilos del servidor lo comparten.
- `req.state::<T>()` devuelve un `&T` que vive tanto como la aplicación: sin clonar y sin bloqueos.
- Pedir un tipo que nunca se registró provoca un pánico, y el cliente recibe un `500`. `req.try_state::<T>()` devuelve un `Option` en su lugar.

Para modificar el estado desde una petición, usa un tipo pensado para compartirse entre hilos: un atómico para un contador, un `Mutex` o un `RwLock` para una colección.

```rust
use std::sync::atomic::{AtomicU64, Ordering};

app.state(AtomicU64::new(0));
app.get("/visits", |req: Request| async move {
    let n = req.state::<AtomicU64>().fetch_add(1, Ordering::Relaxed) + 1;
    format!("visita #{n}")
});
```

El estado vive en memoria: se pierde cuando el servidor se reinicia, y cada servidor tiene su propia copia. Los datos que deben durar van en la [base de datos](database.md). Más patrones en la [guía de estado de vitesse](https://github.com/maxlestage/Vitesse/blob/master/docs/es/state.md).

## Cómo funciona el servidor

`app.run()` prepara la aplicación (base de datos, tablas, hooks `on_start`, escudo, archivos estáticos, islas), muestra la línea de arranque y sirve peticiones hasta `Ctrl+C` o `SIGTERM`:

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: none
```

- Escucha en **todas las interfaces de red** (`0.0.0.0`), que es lo que necesitan los contenedores y los proveedores como Heroku.
- Con la base de datos en memoria o SQLite, ejecuta **un hilo por núcleo de CPU**, cada uno con su propio bucle de eventos: el modo más rápido de vitesse. `app.workers(n)` y `app.thread_per_core(false)` se aplican en este modo.
- Con un servidor de base de datos (PostgreSQL, MySQL, Redis, MongoDB), funciona sobre un runtime de tokio con un hilo por núcleo, compartido con las conexiones a la base de datos. `workers` y `thread_per_core` no se aplican.
- Con `Ctrl+C` o `SIGTERM`, deja de aceptar conexiones y da hasta 10 segundos a las peticiones en curso para terminar (apagado ordenado). Heroku y Docker envían `SIGTERM` antes de detener un contenedor.

## Inicialización asíncrona

`app.run()` crea su propio runtime, así que `main` no necesita ser `async`. Cuando necesites hacer `.await` de algo antes de que arranque la aplicación (abrir un pool de base de datos con ajustes propios, obtener una configuración remota), haz que `main` sea asíncrono con tokio y arranca la aplicación con `app.serve().await`:

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use paquets::orm::PgPool;
use paquets::prelude::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut app = app();
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if url.starts_with("postgres") {
            // Como máximo 10 conexiones, para un plan alojado pequeño.
            let pool = PgPool::connect_with(&url, 10).await.map_err(std::io::Error::other)?;
            app.database(Db::from(pool));
        }
    }
    app.serve().await
}
```

`serve()` hace todo lo que hace `run()`, en el runtime actual (`workers` y `thread_per_core` no se aplican).

## Features de Cargo

Las bases de datos, los tipos de columna adicionales y HTTP/3 son features de Cargo del crate `paquets`: consulta [Instalación](installation.md#features-de-cargo).

## Lista de comprobación para producción

- [ ] Compila en modo release: `paquets build` (o `cargo build --release`). El `Dockerfile` de un proyecto generado lo hace.
- [ ] Usa una base de datos persistente: `DATABASE_URL` apuntando a PostgreSQL en Heroku (ni SQLite ni en memoria).
- [ ] Escucha en `PORT` (automático) y lanza el programa desde la carpeta que contiene `public/` y `pkg/`.
- [ ] Detrás de un proxy que no sea el de Heroku, dile al escudo dónde está la IP del cliente (consulta [Seguridad](security.md#la-ip-del-cliente-detrás-de-un-proxy)).
- [ ] Guarda los secretos en variables de entorno.
- [ ] Sirve el sitio por HTTPS: Heroku lo hace en `*.herokuapp.com` y, con su gestión de certificados, en los dominios propios; en otros sitios, usa un proxy o una CDN.
- [ ] Protege lo que modifica datos: un middleware, `Auth` o sesiones (consulta [Seguridad](security.md#autenticación)).
- [ ] Ejecuta las pruebas en cada push: el workflow de CI lo hace (consulta [Automatización](automation.md)).

## Próximos pasos

- [Rendimiento](performance.md): qué hace rápido a Paquets y cómo mantenerlo rápido.
- [Desplegar en Heroku](deploy-heroku-mobile.md): config vars y logs en Heroku.
- [Seguridad](security.md): los ajustes del escudo.
