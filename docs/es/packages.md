# Los cuatro paquetes

Paquets está construido sobre cuatro paquetes del mismo autor: vitesse, securite, memoire-orm y active. Cada uno es una biblioteca completa que también puedes usar por separado. Paquets los conecta, elige los valores por defecto y los reexporta, de modo que basta con una dependencia y un import. Esta página muestra lo que aporta cada paquete, cómo lo usa Paquets y cómo llegar a las partes que el prelude no incluye.

## Visión general

| Paquete | Función | Reexportado como | Repositorio | Documentación |
|---|---|---|---|---|
| **vitesse** | servidor HTTP al estilo de Express.js: rutas, middleware, routers, WebSocket, HTTP/3, cliente de pruebas | `paquets::vitesse` | [maxlestage/Vitesse](https://github.com/maxlestage/Vitesse) | [documentación](https://github.com/maxlestage/Vitesse/blob/master/docs/es/README.md) |
| **securite** | middleware de seguridad: WAF, límite de tasa, CSRF, CORS, cabeceras de seguridad y CSP, bloqueador de bots, autenticación… | `paquets::securite` | [maxlestage/Securite](https://github.com/maxlestage/Securite) | [documentación](https://github.com/maxlestage/Securite/blob/master/docs/es/introduction.md) |
| **memoire-orm** | ORM asíncrono con tipado seguro, SQL construido en tiempo de compilación: memoria, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB | `paquets::orm` | [maxlestage/Memoire](https://github.com/maxlestage/Memoire) | [documentación](https://github.com/maxlestage/Memoire/blob/master/docs/es/introduction.md) |
| **active** | front end: componentes como funciones de Rust, `use_state`, renderizado en el servidor con etiquetas SEO completas, islas WebAssembly | `paquets::active` | [maxlestage/Active](https://github.com/maxlestage/Active) | [documentación](https://github.com/maxlestage/Active/blob/master/site/docs/es/introduction.md) |

Paquets también reexporta `paquets::serde` y `paquets::serde_json` (y `json!` en el prelude), para que `#[model]` y `#[data]` funcionen sin añadir serde a tu `Cargo.toml`, y `paquets::test`, los tipos del cliente de pruebas.

## vitesse: el servidor

[vitesse](https://github.com/maxlestage/Vitesse) («velocidad» en francés) es el servidor HTTP, con su propio motor HTTP/1.1 sobre tokio. Atiende unas 291 000 peticiones por segundo con 2 núcleos en `GET /`, más que actix-web y axum en la misma máquina, y 2.78 millones con pipelining.

**En Paquets:** `App` contiene una `vitesse::App` y da acceso a todos sus métodos (se desreferencia a ella): `get`, `post`, `ws`, `middleware`, `mount`, `state`, `fallback`, `on_error`, `body_limit`, `static_dir`, `workers`… Los handlers reciben una `vitesse::Request` y devuelven cualquier cosa que implemente `vitesse::IntoResponse`. El cliente de pruebas de `app.test()` es el `TestClient` de vitesse.

**En el prelude:** `Request`, `Response`, `IntoResponse`, `Json`, `Html`, `Redirect`, `Cookie`, `StatusCode`, `Method`, `Router`, `Next`, `HandlerExt` y los módulos `res`, `middleware` y `ws`.

**Más allá del prelude**, a través de `paquets::vitesse`:

```rust
use std::time::Duration;

let mut app = App::new();
// Los middlewares integrados de vitesse: CORS, timeout, logger, helmet.
app.middleware(middleware::timeout(Duration::from_secs(10)));
// Una carpeta con sus propios ajustes de caché.
app.serve_dir("/assets", paquets::vitesse::ServeDir::new("assets").max_age(Duration::from_secs(3600)));
```

- `paquets::vitesse::tokio`: el runtime de tokio sobre el que funciona vitesse (`tokio::spawn`, `tokio::sync`, `tokio::time`…), sin añadir tokio a tus dependencias;
- `paquets::vitesse::Error`: el tipo de error propio de vitesse, envuelto por `paquets::Error` (`err.into_inner()` lo devuelve);
- `paquets::vitesse::ws::Upgrade`: handshakes WebSocket con subprotocolos y límites de tamaño;
- HTTP/3 con la feature `http3` (consulta la [guía de HTTP/3 de vitesse](https://github.com/maxlestage/Vitesse/blob/master/docs/es/http3.md)).

## securite: el escudo

[securite](https://github.com/maxlestage/Securite) es un conjunto de 19 guards de seguridad, reunidos en un `Shield`. Su pipeline recomendado cuesta unos 6 µs por petición. Está escrito para el ecosistema tower (axum, hyper, tonic) y no contiene código `unsafe`.

**En Paquets:** `App::new()` pone `paquets::shield().build()` delante de cada petición: el preset recomendado de securite, con la IP del cliente leída de `X-Forwarded-For` en Heroku. Paquets hace de puente entre las peticiones de vitesse y los guards de securite: el escudo ve la línea de la petición y las cabeceras, añade sus cabeceras a cada respuesta y transmite sus hallazgos a tus handlers (la IP del cliente, el usuario autenticado, el identificador de petición). Las páginas calculan su propia Content-Security-Policy con las funciones de hash de securite.

**Para usarlo:**

```rust
use paquets::securite::prelude::*;

app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
```

`paquets::securite::prelude::*` trae todos los guards (`RateLimit`, `Waf`, `Csrf`, `Cors`, `Auth`, `IpFilter`, `HostFilter`, `SecurityHeaders`…), `Shield`, `ShieldBuilder`, `ClientIpSource`, `ErrorFormat`, `ClientIp` y `Principal`. `paquets::securite::headers::ContentSecurityPolicy` construye políticas. Consulta [Seguridad](security.md).

## memoire-orm: la base de datos

[memoire-orm](https://github.com/maxlestage/Memoire) asocia structs de Rust a tablas y permite escribir las consultas en Rust, comprobadas por el compilador. El SQL de las operaciones por clave primaria se construye en tiempo de compilación, y el mismo código funciona en seis backends.

**En Paquets:**

- `#[model]` añade `#[derive(Model)]` (y serde) a tu struct, y hace que el código generado apunte a `paquets::orm`, así que tu proyecto no necesita su propia dependencia de `memoire-orm`;
- `Db` es un único handle para todos los backends, elegido por `DATABASE_URL`, e implementa los traits `Executor` y `Database` de memoire: todos los métodos de los modelos y todas las consultas aceptan `req.db()`;
- `app.api::<M>()` y `app.model::<M>()` crean las tablas al arrancar; los errores de la base de datos se convierten en `404`, `409` o `400` a través de `?`.

**En el prelude:** los traits `Model`, `Executor`, `Database` y `Transaction`, que dan sus métodos a los modelos y a la base de datos.

**Más allá del prelude**, a través de `paquets::orm`: `Migrator` y `Migration`, `Dialect`, la macro `params!` para el SQL sin procesar, `Page` (el resultado de `paginate`), las columnas `Json<T>` y los propios backends (`SqliteDb`, `PgPool`, `MySqlPool`, `RedisDb`, `MongoDb`, `MemoryDb`), para abrir una base de datos con tus propios ajustes y pasarla a `app.database(Db::from(...))`. Consulta [Base de datos](database.md).

## active: el front end

[active](https://github.com/maxlestage/Active) construye las vistas con una función por etiqueta HTML, las renderiza en el servidor con etiquetas SEO completas y convierte las partes marcadas de una página en islas WebAssembly. No tiene macros, ni lenguaje de plantillas, ni dependencias del lado del servidor; su bundle del contador pesa 36.8 kB comprimido.

**En Paquets:** `Page` envuelve el `Document` de active: le añade el tema, la Content-Security-Policy y la carga del bundle de las islas (que se encuentra en `pkg/` al arrancar). Las islas son las islas de active, hidratadas en el navegador por `paquets::hydrate` (feature `client`).

**En el prelude:** todas las funciones de etiquetas HTML (`div()`, `p()`, `a()`…) y el módulo `html` (para `html::main()`), `Element`, `Node`, `Seo`, `State`, `use_state`, `effect`, `island`, `el`, `text`, `text_dyn`, `fragment`, `fragment_dyn` y `keyed`.

**Más allá del prelude**, a través de `paquets::active`: `Document` para renderizar una página sin el tema ni la política de seguridad de `Page`, por ejemplo para exportar archivos estáticos. Con la feature `client` (lado navegador): `paquets::hydrate`, `paquets::mount_to` y `paquets::mount` para las aplicaciones renderizadas por completo en el navegador, y `paquets::navigate`, `paquets::location`, `paquets::query` y `paquets::query_param` para el enrutamiento sin recargar la página. Consulta [Islas](islands.md) y la [documentación de active](https://github.com/maxlestage/Active/blob/master/site/docs/es/introduction.md).

## Correspondencia de los nombres

| En Paquets | En el paquete |
|---|---|
| `App` | `vitesse::App`, más el escudo, la base de datos, la API REST, los archivos estáticos y las islas |
| `app.run()` | `vitesse::App::run(port)`, con el puerto tomado de `PORT` |
| `app.security(shield)` | un `securite::Shield` |
| `paquets::shield()` | `securite::ShieldBuilder::recommended()` con la IP del cliente de Heroku |
| `#[model]` | `#[derive(memoire_orm::Model, Serialize, Deserialize, Clone, Debug)]` con `#[memoire(crate = "::paquets::orm")]` |
| `#[data]` | `#[derive(Serialize, Deserialize, Clone, Debug)]` |
| `Db`, `req.db()` | cualquier backend de memoire, tras un único tipo |
| `Page` | `active::Document`, más el tema, la CSP y el bundle |
| `paquets::Error`, `Result<T>` | `vitesse::Error`, con el estado correcto para los errores de memoire |

## Usar un paquete por separado

Cada paquete se publica por separado y funciona sin Paquets:

- **vitesse** para un servidor sin base de datos ni páginas, o con tus propias elecciones;
- **securite** delante de un servicio axum, hyper o tonic: `.layer(Shield::recommended())`;
- **memoire-orm** en cualquier programa de Rust que guarde datos, una herramienta de línea de comandos o un worker;
- **active** para un sitio estático, una aplicación del lado del cliente o páginas servidas por otro framework (su documentación tiene una guía para axum).

Sus README explican cómo empezar. Los cuatro usan la edición 2024. Paquets, vitesse, securite y memoire-orm tienen licencia MIT o Apache-2.0, y active, licencia MIT.

## Versiones

Paquets 0.1 depende de vitesse 0.1, securite 0.1, memoire-orm 0.1 y active 0.1. Mientras no estén en crates.io, Cargo los toma de sus repositorios de GitHub; consulta [Publicar en crates.io](publish-crates-io.md) para ver el orden de publicación.

## Próximos pasos

- [Introducción](introduction.md): lo que Paquets añade a los cuatro paquetes.
- [Chuleta](cheatsheet.md): toda la API en una sola página.
- [Preguntas frecuentes](faq.md): dudas habituales.
