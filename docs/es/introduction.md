# Introducción

Paquets 📦 es **el framework full-stack más sencillo para Rust**. Un crate, un import y una `App` te dan un servidor web rápido, seguridad delante de cada petición, una base de datos, APIs REST, páginas web con etiquetas SEO completas y partes interactivas que se ejecutan en el navegador como WebAssembly. Esta página explica qué es Paquets, qué hace por ti y qué no hace.

## ¿Qué es Paquets?

Paquets («paquetes» en francés) es una biblioteca de Rust para crear sitios web y APIs web. Reúne cuatro paquetes escritos por el mismo autor, y cada uno de ellos también se puede usar por separado:

| Paquete | Qué hace en Paquets |
|---|---|
| [Vitesse](https://maxlestage.github.io/Vitesse/) | El servidor HTTP, con una API al estilo de Express.js: `app.get(...)`, `req.param(...)`, middleware, routers, WebSocket. |
| [Securite](https://maxlestage.github.io/Securite/) | El escudo de seguridad delante de cada petición: cortafuegos, límite de tasa, protección CSRF, cabeceras de seguridad y más. |
| [memoire-orm](https://maxlestage.github.io/Memoire/) | La base de datos: modelos y consultas con tipado seguro, en memoria, SQLite, PostgreSQL, MySQL, Redis o MongoDB. |
| [Active](https://maxlestage.github.io/Active/) | El front end: vistas escritas como funciones de Rust, renderizado en el servidor con etiquetas SEO e islas WebAssembly. |

No necesitas aprender cuatro bibliotecas para empezar. Paquets las conecta, elige buenos valores por defecto y expone una sola API, pequeña. Cuando necesites más, cada paquete está a un import de distancia (consulta [Los cuatro paquetes](packages.md)).

## Una aplicación completa

Este es un programa completo. Sirve una página de inicio, una ruta de saludo y una API REST respaldada por una base de datos:

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
        Page::new("Inicio")
            .description("Mi sitio, en Rust.")
            .body(html::main().child(h1().text("Hola 👋")))
    });

    app.get("/hello/:name", |req: Request| async move {
        format!("¡Hola, {}!", req.param("name").unwrap_or("tú"))
    });

    app.api::<Todo>("/api/todos");

    app.run()
}
```

Ejecútalo y obtendrás:

- `http://localhost:3000/`: una página HTML con estilos, con su título, su descripción y sus etiquetas Open Graph y Twitter, en modo claro u oscuro;
- `http://localhost:3000/hello/Ada`: el texto `¡Hola, Ada!`;
- `http://localhost:3000/api/todos`: una API JSON para listar, leer, crear, reemplazar, modificar y eliminar tareas (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`), guardadas en una tabla de la base de datos que se creó al arrancar.

> [!NOTE]
> En un archivo que contiene `fn main`, la etiqueta HTML `<main>` se escribe `html::main()`, porque el nombre `main` ya pertenece a la función de tu programa. En cualquier otro sitio, `main()` funciona. Consulta [Páginas](pages.md#la-etiqueta-main-en-mainrs).

## Lo que obtienes sin configurar nada

`App::new()` está lista para producción. Nada de la tabla siguiente necesita una sola línea de configuración:

| Funcionalidad | Comportamiento por defecto | Se cambia con |
|---|---|---|
| Servidor | Puerto tomado de la variable `PORT` (Heroku), `3000` si no existe. `Ctrl+C` o `SIGTERM` lo detienen de forma ordenada. | `app.port(8080)` |
| Seguridad | El escudo recomendado de Securite en cada petición: identificadores de petición, comprobaciones del protocolo, límite de 2 MiB para el cuerpo, límite de tasa de 20 peticiones por segundo por IP, bloqueador de bots, cortafuegos, protección CSRF, cabeceras de seguridad, cookies reforzadas. | `app.security(...)` |
| Base de datos | Tomada de la variable `DATABASE_URL`; en memoria cuando no está definida. Las tablas de tus modelos se crean al arrancar. | `app.database(...)` |
| Páginas | HTML renderizado en el servidor con etiquetas SEO, un tema integrado y una Content-Security-Policy que permite exactamente lo que contiene la página. | Métodos de `Page` |
| Archivos estáticos | La carpeta `public/` se sirve en la raíz del sitio. | `app.static_dir(...)` |
| Islas | El bundle del navegador en `pkg/` (generado por `paquets build`) se detecta al arrancar y solo lo cargan las páginas que contienen islas. | — |
| Logs | Una línea por petición: `GET /hello/Ada 200 0.339 ms`. | `app.quiet()` |

## ¿Por qué Paquets?

- **Sencillez.** Una dependencia, un import (`use paquets::prelude::*;`), una `App`. Un handler es una función `async` que recibe una petición y devuelve una respuesta. Los errores son valores, y `?` los convierte en el estado HTTP correcto.
- **Seguro por defecto.** Ataques como la inyección SQL en la URL, la falsificación de peticiones entre sitios (CSRF), el clickjacking o la fuerza bruta se bloquean antes de que se ejecute tu código. Tienes que desactivar la protección, no activarla.
- **Rápido.** Vitesse atiende unas 291 000 peticiones por segundo en `GET /` con 2 núcleos de CPU, más que actix-web y axum en la misma máquina, y 2.78 millones por segundo con pipelining. Todo el pipeline de Securite cuesta unos 6 µs por petición. El bundle de isla más pequeño (un contador) pesa 36.8 kB comprimido. Consulta [Rendimiento](performance.md).
- **Tipado seguro de la base de datos a la página.** Una columna mal escrita, una comparación entre un número y una cadena o un campo que falta en una respuesta son errores de compilación, no bugs en producción.
- **Fácil de desplegar.** Un proyecto creado con `paquets new` contiene un `Dockerfile`, los archivos para desplegar en Heroku con un solo toque y workflows de GitHub. Puedes desplegar, redesplegar y leer tus logs [desde un teléfono](deploy-heroku-mobile.md).

## ¿Para quién es?

- **Principiantes en Rust** que quieren crear un sitio web de verdad. La mayoría de los handlers ocupan pocas líneas, y esta documentación explica las partes propias de Rust a medida que aparecen.
- **Desarrolladores web** que vienen de Express, Rails, Laravel o Django y quieren un único framework que cubra todo el stack, sin tener que ensamblar una docena de crates.
- **Desarrolladores de Rust** que quieren un framework pequeño y explícito, con valores por defecto seguros, y a quienes les gusta leer el código del que dependen.

## Lo que Paquets (todavía) no hace

Paquets es joven (versión 0.1). Ten en cuenta estos límites antes de elegirlo:

- **Sin TLS (HTTPS) ni HTTP/2 en el propio servidor.** Ponlo detrás del router de una plataforma (Heroku lo hace por ti), de un proxy inverso (Nginx, Caddy) o de una CDN, que se encargan de HTTPS. HTTP/3 sobre QUIC está disponible como opción (feature `http3`).
- **Sin cambios automáticos del esquema.** Las tablas se crean cuando no existen, pero añadir un campo a un modelo no modifica una tabla existente: escribe una migración (consulta [Base de datos](database.md#modificar-una-tabla-más-adelante)).
- **Las islas necesitan dos herramientas más**: el target `wasm32-unknown-unknown` y `wasm-bindgen-cli`. `paquets dev` las instala por ti la primera vez. Las páginas sin islas no necesitan nada.
- **Sin cuentas de usuario integradas.** El escudo protege cada petición, pero el inicio de sesión de los usuarios (contraseñas, sesiones) queda en tus manos. Consulta [Seguridad](security.md#autenticación).
- **Todavía no está en crates.io.** Hasta que se publiquen los cuatro paquetes y Paquets, los proyectos dependen del repositorio de GitHub. Consulta [Instalación](installation.md) y [Publicar en crates.io](publish-crates-io.md).

## Cómo está organizada esta documentación

- **Primeros pasos**: [Instalación](installation.md), un tutorial de [Inicio rápido](quick-start.md) y la [Estructura del proyecto](project-structure.md).
- **Lo esencial**: [Enrutamiento](routing.md), [Peticiones y respuestas](requests-responses.md), [Páginas](pages.md), [Base de datos](database.md), [API REST](rest-api.md) y [Seguridad](security.md).
- **Más allá**: [Islas](islands.md), [WebSocket](websocket.md), [Pruebas](testing.md), [Errores](errors.md), [Configuración](configuration.md) y [Rendimiento](performance.md).
- **Desplegar**: [Heroku desde el móvil](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Publicar en crates.io](publish-crates-io.md) y [Automatización](automation.md).
- **Referencia**: la [Herramienta de línea de comandos](cli.md), la [Chuleta](cheatsheet.md), [Los cuatro paquetes](packages.md) y las [Preguntas frecuentes](faq.md).

## Próximos pasos

- [Instalación](installation.md): instala Rust y el comando `paquets`.
- [Inicio rápido](quick-start.md): crea tu primera aplicación en diez minutos.
