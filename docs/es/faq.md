# Preguntas frecuentes

Respuestas breves a las preguntas habituales, con enlaces a las páginas que entran en detalle.

## Sobre Paquets

### ¿Qué es Paquets, en una frase?

Un framework de Rust que te da, con un solo crate y una sola `App`, un servidor rápido, seguridad delante de cada petición, una base de datos con consultas tipadas, APIs REST, páginas renderizadas en el servidor con etiquetas SEO completas e islas WebAssembly interactivas. Consulta la [Introducción](introduction.md).

### ¿Está listo para producción?

Paquets es joven (versión 0.1). Está construido sobre cuatro paquetes que se prueban por separado (vitesse, securite, memoire-orm, active), y sus valores por defecto están pensados para producción: escudo de seguridad, apagado ordenado, base de datos tomada de `DATABASE_URL`, compilaciones release en el `Dockerfile`. Pero su API todavía puede cambiar entre versiones 0.x, y no tiene años de uso a sus espaldas. Pruébalo en un proyecto en el que puedas seguir su evolución, y lee los límites en la [Introducción](introduction.md#lo-que-paquets-todavía-no-hace).

### ¿Por qué no axum, actix-web o Leptos?

Son proyectos excelentes y más grandes. Paquets elige otros compromisos:

- **Todo en un solo sitio.** El servidor, la seguridad, la base de datos, las páginas y las islas funcionan juntos sin tener que elegir y conectar una docena de crates.
- **Seguro por defecto.** El cortafuegos, el límite de tasa, la protección CSRF y las cabeceras de seguridad están activos antes de que escribas una sola línea de código.
- **Sin macros en las vistas, sin extractores en los handlers.** Un handler recibe una `Request` y devuelve una respuesta; una vista es una cadena de llamadas a funciones. rust-analyzer y el compilador lo entienden todo.
- **Rápido.** Su servidor supera a actix-web y axum en sus benchmarks (consulta [Rendimiento](performance.md)).

Elige un framework más grande si necesitas su ecosistema, las server functions o un equipo que ya lo conozca.

### ¿Necesito conocer bien Rust?

No. La mayoría de los handlers ocupan pocas líneas, y esta documentación explica las partes propias de Rust a medida que aparecen (`async move`, `?`, `Option`). Conocer los fundamentos del lenguaje (los primeros capítulos de [el libro de Rust](https://doc.rust-lang.org/book/)) ayuda mucho.

### ¿Tengo que escribir JavaScript?

No para las páginas, las APIs y las islas: todo es Rust, incluidas las partes interactivas. El navegador recibe un pequeño cargador JavaScript generado por `wasm-bindgen`, que nunca escribes ni modificas. Para un cliente WebSocket en el navegador, un pequeño script es la opción más sencilla (consulta [WebSocket](websocket.md#conectarse-desde-una-página)).

### ¿Es gratuito?

Paquets es de código abierto, con licencia MIT o Apache-2.0. El alojamiento es otra cuestión: Heroku no tiene plan gratuito (un dyno Eco cuesta unos 5 $ al mes; consulta [Desplegar en Heroku](deploy-heroku-mobile.md)). La imagen Docker funciona en cualquier proveedor, incluido tu propio servidor.

## Escribir código

### ¿Por qué `main().child(...)` falla con «no method named child found for enum Result»?

En un archivo que define `fn main` (normalmente `src/main.rs`), el nombre `main` es la función de tu programa, que oculta la etiqueta HTML `main()`. En ese archivo, escribe `html::main()`. En cualquier otro sitio, `main()` funciona. Consulta [Páginas](pages.md#la-etiqueta-main-en-mainrs).

### ¿Por qué el compilador dice «future cannot be sent between threads safely»?

El future de un handler debe poder moverse entre hilos, y hay dos cosas que lo impiden:

- una **vista** (`div()`, `Page`…) creada antes de un `.await`: construye la página después del último `.await` (consulta [Páginas](pages.md#construye-la-página-al-final-del-handler));
- un guard de `std::sync::Mutex` o de `RwLock` conservado a través de un `.await`: lee primero el cuerpo o la base de datos y luego toma el bloqueo, o usa `tokio::sync::Mutex`.

### ¿Cómo cambio el formato de todos los errores?

Con `app.on_error(|err: Error| ...)`: recibe un `paquets::Error` y devuelve cualquier respuesta. Fija tú el estado con `res::status(err.status())`. Consulta [Errores](errors.md#cambiar-el-formato-de-todos-los-errores).

### ¿Cómo comparto datos entre peticiones?

Para los datos que deben durar, usa la [base de datos](database.md). Para los ajustes, las cachés y los contadores que viven tanto como el servidor, usa `app.state(value)` y `req.state::<T>()` (consulta [Configuración](configuration.md#estado-compartido)).

### ¿Cómo obtengo la dirección IP del cliente?

`req.extensions().get::<paquets::securite::ClientIp>()` devuelve la dirección que encontró el escudo, que es correcta detrás del router de Heroku. Consulta [Peticiones y respuestas](requests-responses.md#la-dirección-ip-del-cliente).

### ¿Cómo inicio la sesión de los usuarios?

Paquets no incluye cuentas de usuario. Según lo que necesites: un token compartido para una API, el guard `Auth` de securite para una sección de administración, o sesiones guardadas en tu base de datos. Consulta [Seguridad](security.md#autenticación).

### ¿Los componentes pueden ser asíncronos?

No: los componentes son funciones normales que construyen una vista. Haz primero el trabajo asíncrono (base de datos, llamadas HTTP) en el handler y luego construye la página con los resultados.

### ¿Una isla puede llamar a mi API?

Sí, con el `fetch` del navegador a través de los crates `web-sys` y `wasm-bindgen-futures` (consulta la [guía de eventos de active](https://github.com/maxlestage/Active/blob/master/site/docs/es/events.md#trabajo-asíncrono-en-los-manejadores)). Una isla también puede contener un formulario HTML normal que envíe los datos a una de tus rutas.

### ¿Hay recarga en caliente (hot reload)?

No viene integrada. `paquets dev` compila y ejecuta la aplicación una vez; después de un cambio, detenla y vuelve a ejecutarla. Un vigilante de archivos lo automatiza: `cargo watch -s "paquets dev"` (instálalo con `cargo install cargo-watch`).

## Base de datos

### ¿Qué base de datos debería usar?

En memoria mientras aprendes y en las pruebas, SQLite para un solo servidor con un disco persistente, y PostgreSQL en Heroku o con varios servidores. Consulta [Base de datos](database.md#qué-base-de-datos-usar).

### ¿Por qué han desaparecido mis datos?

Sin `DATABASE_URL`, la base de datos está en memoria y se vacía cuando el servidor se detiene. En Heroku, los archivos SQLite también se borran en cada reinicio. Usa PostgreSQL (el add-on Heroku Postgres) para los datos que deben durar.

### He añadido un campo a un modelo y ahora recibo errores 500.

Las tablas se crean cuando no existen, pero nunca se modifican: la tabla existente no tiene la nueva columna. Añade una migración. Consulta [Base de datos](database.md#modificar-una-tabla-más-adelante).

### ¿Cómo veo el SQL de una consulta?

`Todo::query().filter(Todo::DONE.eq(false)).to_sql(paquets::orm::Dialect::Postgres)` lo devuelve como una cadena, sin ejecutarlo.

## Seguridad

### ¿Por qué recibo 403 Forbidden?

El escudo bloqueó la petición: el campo `code` de la respuesta dice por qué (`waf_blocked`, `csrf_rejected`, `bot_blocked`…). Un formulario enviado desde otro sitio, un patrón de ataque en la URL o el user agent de una herramienta de ataque se rechazan antes de que se ejecute tu código. Consulta [Seguridad](security.md#cómo-es-una-petición-bloqueada).

### ¿Por qué recibo 429 Too Many Requests?

El límite de tasa permite 20 peticiones por segundo por dirección IP, con ráfagas de 100. Tres casos habituales:

- **en las pruebas**, todas las peticiones vienen del mismo cliente: quita el límite de tasa en las pruebas que envían muchas peticiones (consulta [Pruebas](testing.md#probar-la-seguridad));
- **en una prueba de carga**: lo mismo; consulta [Rendimiento](performance.md#medir);
- **detrás de un proxy** que no sea el de Heroku, todos los visitantes comparten la dirección del proxy: dile al escudo dónde está la dirección real (consulta [Seguridad](security.md#la-ip-del-cliente-detrás-de-un-proxy)).

### Mi página no puede cargar una fuente o un script de otro sitio.

La Content-Security-Policy de la página solo permite tus propios archivos. Descarga la fuente o el script en `public/` (mejor para la velocidad y la privacidad), o añade el origen a la política de la página. Consulta [Páginas](pages.md#la-content-security-policy).

### ¿Se admite HTTPS? ¿Y HTTP/2?

El servidor habla HTTP/1.1 (y HTTP/3 con la feature `http3`), sin TLS. HTTPS y HTTP/2 los aporta lo que tenga delante: el router de Heroku, un proxy inverso (Caddy, Nginx) o una CDN. El escudo mantiene después a los navegadores en HTTPS con la cabecera `Strict-Transport-Security`.

### ¿Puedo desactivar la seguridad?

Sí, con `app.security(None)`, pero hazlo solo cuando otra capa proteja la aplicación. Para relajar una regla, modifica el escudo en su lugar: `paquets::shield().without("bot_blocker").build()`. Consulta [Seguridad](security.md#modificar-el-escudo).

## Despliegue

### ¿De verdad puedo desplegar desde un teléfono?

Sí: el botón Deploy to Heroku, el add-on de base de datos, los despliegues automáticos, los cambios de código, los logs y los dominios propios funcionan desde el navegador de un teléfono y la app de GitHub. Solo crear un proyecto nuevo con `paquets new` requiere Rust, una sola vez, en un ordenador o en un entorno de desarrollo en la nube. Consulta [Desplegar en Heroku desde el móvil](deploy-heroku-mobile.md).

### ¿Funciona en otros proveedores además de Heroku?

Sí: el `Dockerfile` funciona en Render, Fly.io, Google Cloud Run, Railway o tu propio servidor. Revisa el ajuste de la IP del cliente para tu plataforma. Consulta [Docker](deploy-docker.md#otras-plataformas).

### ¿Por qué la primera compilación es tan lenta?

Rust compila cada dependencia una vez, con todas las optimizaciones en modo release, y la primera compilación de un proyecto con islas también instala `wasm-bindgen-cli`. Las compilaciones siguientes solo recompilan tu código. En Heroku, la primera construcción suele tardar de 10 a 20 minutos.

## El proyecto

### ¿Cómo actualizo Paquets?

En un proyecto: `cargo update -p paquets` (y los cuatro paquetes se actualizan con él). La herramienta de línea de comandos: `cargo install paquets --locked --force` (o el mismo comando con `--git`). Lee antes el [registro de cambios](https://github.com/maxlestage/Paquets/blob/master/CHANGELOG.md): antes de la 1.0, las versiones menores pueden cambiar la API.

### ¿Cómo informo de un bug o contribuyo?

Abre una issue en [GitHub](https://github.com/maxlestage/Paquets/issues). Las pull requests son bienvenidas; lee antes `CONTRIBUTING.md`. Un bug en uno de los cuatro paquetes es mejor notificarlo en su propio repositorio (consulta [Los cuatro paquetes](packages.md)).

### ¿Qué licencia tiene?

MIT o Apache-2.0, a tu elección, para Paquets, vitesse, securite y memoire-orm; MIT para active.

## Próximos pasos

- [Inicio rápido](quick-start.md): crea tu primera aplicación.
- [Chuleta](cheatsheet.md): toda la API en una sola página.
