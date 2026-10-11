# Chuleta

Todo Paquets en una sola página. Todos los ejemplos suponen `use paquets::prelude::*;`. Sigue los enlaces para ver los detalles.

## Línea de comandos

| Tarea | Comando |
|---|---|
| Instalar la herramienta | `cargo install --git https://github.com/maxlestage/Paquets paquets --locked` |
| Crear un proyecto | `paquets new my-app` (`--minimal`: sin islas) |
| Ejecutar mientras desarrollas | `paquets dev` |
| Compilar para producción | `paquets build` |
| Comprobar las herramientas | `paquets doctor` |
| Ejecutar las pruebas | `cargo test` |

Consulta [Herramienta de línea de comandos](cli.md).

## La aplicación

| Tarea | Código |
|---|---|
| Crear | `let mut app = App::new();` |
| Ejecutar (`PORT` o 3000) | `app.run()` desde `fn main() -> std::io::Result<()>` |
| Ejecutar dentro de `#[tokio::main]` | `app.serve().await` |
| Elegir el puerto | `app.port(8080);` |
| Desactivar el log de peticiones | `app.quiet();` |
| Aplicación instalable, sin conexión (PWA) | `app.pwa(Pwa::new("Mi app").theme_color("#6d4aff"));` |
| Elegir la base de datos | `app.database("sqlite://app.db");` o `app.database(Db::memory());` |
| Modificar el escudo | `app.security(paquets::shield().guard(...).build());` |
| Desactivar el escudo | `app.security(None);` |
| Código al arrancar | `app.on_start(\|db\| async move { ...; Ok(()) });` |
| Estado compartido | `app.state(value);` y luego `req.state::<T>()` |
| Límite de lectura de los cuerpos | `app.body_limit(10 * 1024 * 1024);` |
| Cliente de pruebas | `let client = app.test().await;` |

Consulta [Configuración](configuration.md).

## Rutas

| Tarea | Código |
|---|---|
| Métodos | `app.get`, `.post`, `.put`, `.patch`, `.delete`, `.all`, `.head`, `.options` |
| Handler closure | `app.get("/", \|_\| async { "Inicio" });` |
| Closure que usa la petición | `app.get("/hi/:name", \|req: Request\| async move { ... });` |
| Handler función | `async fn home(req: Request) -> Result<Page> { ... }` y luego `app.get("/", home);` |
| Parámetro | `/users/:id` → `req.param("id")` (`Option<&str>`) |
| Parámetro tipado | `let id: i64 = req.param_as("id")?;` (400 si no es válido) |
| Comodín | `/files/*path` → `req.param("path")` |
| Router | `let mut api = Router::new(); api.get(...); app.mount("/api", api);` |
| Middleware global | `app.middleware(\|req: Request, next: Next\| async move { next.run(req).await });` |
| Middleware en una ruta | `app.get("/admin", handler.with(auth));` |
| Middleware en un router | `router.middleware(auth);` |
| 404 personalizada | `app.fallback(\|_\| async { Page::new("Página no encontrada").status(404) });` |
| Carpeta estática | `public/` en `/` automáticamente; `app.static_dir("/files", "files");` |
| WebSocket | `app.ws("/chat", \|req, mut socket\| async move { ... });` |

Consulta [Enrutamiento](routing.md).

## Peticiones

| Leer | Código |
|---|---|
| Método, ruta | `req.method()`, `req.path()` |
| Valor de la query | `req.query("q")` (`Option<Cow<str>>`) |
| Toda la query string | `let s: Search = req.query_as()?;` |
| Cabecera, cookie | `req.header("user-agent")`, `req.cookie("session")` |
| Cuerpo JSON | `let body: NewTodo = req.json().await?;` |
| Cuerpo de formulario | `let form: Login = req.form().await?;` |
| Texto, bytes | `req.text().await?`, `req.bytes().await?` |
| IP del cliente (detrás de proxies) | `req.extensions().get::<paquets::securite::ClientIp>()` |
| Base de datos | `req.db()` |
| Estado | `req.state::<Settings>()` |

Consulta [Peticiones y respuestas](requests-responses.md).

## Respuestas

| Devolver | Da |
|---|---|
| `"texto"`, `String` | `200`, texto plano |
| `Json(value)`, `json!({...})` | `200`, JSON |
| `Page::new("Título").body(...)` | `200`, una página HTML |
| `StatusCode::NO_CONTENT` | ese estado |
| `(201, Json(value))` | un estado y un cuerpo |
| `Redirect::to("/")`, `Redirect::see_other("/")`, `Redirect::permanent("/")` | `302`, `303`, `301` |
| `Result<T>` | `T`, o el estado del error |
| `Option<T>` | `T`, o `404` |
| `res::status(202).header("x-a", "1").cookie(Cookie::new("k", "v")).json(...)` | una respuesta a medida |
| `res::file("report.pdf").await` | un archivo |

## Páginas y vistas

| Tarea | Código |
|---|---|
| Página | `Page::new("Título").description("…").body(view)` |
| Idioma | `.lang("fr")` |
| Etiquetas SEO | `.seo(Seo::new().canonical("…").image("…"))` |
| CSS | `.style("h1{color:red}")`, `.stylesheet("/site.css")` |
| Etiqueta extra en el head | `.head(r#"<link rel="icon" href="/favicon.svg"/>"#)` |
| Sin tema | `.plain()` |
| Estado | `.status(404)` |
| Elemento | `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`… |
| `<main>` en `main.rs` | `html::main()` |
| Texto (escapado) | `.text("…")` |
| Atributos | `.class("card")`, `.id("x")`, `.href("/")`, `.attr("type", "email")` |
| Hijos | `.child(view)`, `.children(iter.map(...))` |
| Condición | `.child(flag.then(\|\| p().text("…")))` |
| Isla | `island("counter", "0", counter)` |

Clases del tema: `.hero`, `.card`, `.grid`, `.row`, `.stack`, `.center`, `.muted`, `.button`, `.ghost`. Variables del tema: `--accent`, `--bg`, `--fg`, `--muted`, `--line`, `--card`, `--radius`, `--font`. Consulta [Páginas](pages.md).

## Modelos

| Tarea | Código |
|---|---|
| Tabla + JSON | `#[model] struct Todo { id: i64, title: String, done: bool }` |
| Nombre de la tabla | `#[model(table = "todos")]` (por defecto: `todo`) |
| Solo JSON | `#[data] struct Login { email: String, password: String }` |
| Único, índice | `#[memoire(unique)]`, `#[memoire(index)]` |
| Admite nulos | `bio: Option<String>` |
| Opcional en JSON | `#[serde(default)] done: bool` |
| Crear solo la tabla | `app.model::<Todo>();` |

## Consultas

| Tarea | Código |
|---|---|
| Todas, una, quizá una | `Todo::all(db)`, `Todo::get(db, id)` (404), `Todo::find(db, id)` (`Option`) |
| Contar | `Todo::count(db)` |
| Insertar | `let mut t = Todo { id: 0, ... }; t.insert(db).await?;` (el id se escribe de vuelta) |
| Actualizar, eliminar | `t.update(db)`, `t.delete(db)`, `Todo::delete_by_key(db, id)` |
| Filtrar | `Todo::query().filter(Todo::DONE.eq(false)).all(db)` |
| Combinar | `a & b`, `a \| b`, `!a` |
| Comparar | `.eq`, `.ne`, `.gt`, `.ge`, `.lt`, `.le`, `.is_in([..])`, `.between(a, b)`, `.is_null()` |
| Texto | `.contains("x")`, `.icontains("x")`, `.starts_with("x")`, `.like("%x")` |
| Ordenar, limitar | `.order_by(Todo::ID.desc()).limit(20).offset(40)` |
| Página | `.paginate(db, page, 20)` → `items`, `total` |
| Primera, existe | `.first(db)`, `.exists(db)` |
| Actualizar varias | `Todo::update_many().set(Todo::DONE, true).filter(...).execute(db)` |
| Eliminar varias | `Todo::delete_many().filter(...).execute(db)` |
| Transacción | `let tx = db.begin().await?; ...(&tx)...; tx.commit().await?;` |
| SQL sin procesar | `db.raw::<(i64, String)>("SELECT …", &paquets::orm::params![…])` |

`db` es `req.db()` en un handler, o el argumento de `on_start`. Consulta [Base de datos](database.md).

## Bases de datos

| `DATABASE_URL` | Base de datos |
|---|---|
| sin definir, `memory` | en memoria |
| `memory://app.memoire` | en memoria, guardada en un archivo |
| `sqlite://app.db` | archivo SQLite |
| `postgres://…` | PostgreSQL (Heroku Postgres) |
| `mysql://…`, `mariadb://…` | MySQL, MariaDB (feature `mysql`) |
| `redis://…` | Redis (feature `redis`) |
| `mongodb://…` | MongoDB (feature `mongodb`) |

## API REST

`app.api::<Todo>("/api/todos");`

| Petición | Respuesta |
|---|---|
| `GET /api/todos?limit=100&offset=0` | `200`, array, `X-Total-Count` |
| `GET /api/todos/:id` | `200` o `404` |
| `POST /api/todos` | `201` con el `id` generado |
| `PUT /api/todos/:id` | `200`, fila entera reemplazada |
| `PATCH /api/todos/:id` | `200`, solo cambian los campos enviados |
| `DELETE /api/todos/:id` | `204` o `404` |

Consulta [API REST](rest-api.md).

## Errores

| Tarea | Código |
|---|---|
| Tipo de retorno | `Result<T>` (= `Result<T, paquets::Error>`) |
| Propagar | `?` |
| Crear | `Error::bad_request(…)`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `unprocessable`, `internal`, `Error::new(418, "…")` |
| En una closure | `Ok::<_, Error>(value)` |
| A partir de `None` | `.ok_or_else(\|\| Error::not_found("…"))?` |
| Formato propio | `app.on_error(\|err\| res::status(err.status()).json(json!({ "message": err.message() })));` |

| `?` sobre | Estado |
|---|---|
| parámetro, query, JSON o formulario incorrecto | `400` |
| fila no encontrada | `404` |
| valor único ya usado | `409` |
| cualquier otra cosa | `500` (detalles en los logs) |

Consulta [Errores](errors.md).

## Seguridad

`use paquets::securite::prelude::*;`

| Tarea | Código |
|---|---|
| Límite de tasa | `paquets::shield().guard(RateLimit::per_minute(600)).build()` |
| Cuerpos más grandes | `.guard(BodyLimit::new(10 << 20))` y `app.body_limit(10 << 20)` |
| Quitar un guard | `.without("bot_blocker")` |
| Front end en otro origen | `.guard(Cors::new().allow_origin(o)).guard(Csrf::new().trusted_origin(o))` |
| Excepción CSRF | `.guard(Csrf::new().exempt("/auth/callback"))` |
| IP del cliente detrás de un proxy | `.client_ip(ClientIpSource::heroku())`, `::fly()`, `::cloudflare()` |
| Autenticación Basic en una sección | `.guard(Auth::basic("Admin").user("admin", "…").protect(["/admin"]))` |
| Registrar las peticiones bloqueadas | `.on_event(\|e\| eprintln!("{} bloqueó {}", e.guard, e.path))` |
| Presets | `ShieldBuilder::strict()`, `ShieldBuilder::api()` |

Consulta [Seguridad](security.md).

## Islas

| Tarea | Código |
|---|---|
| Isla (en `src/lib.rs`) | `pub fn counter(props: &str) -> Node { ... }` |
| Estado | `let n = use_state(0);` `n.get()`, `n.set(v)`, `n.update(\|n\| *n += 1)`, `n.with(\|v\| …)` |
| Estado derivado | `let total = memo(move \|\| items.with(\|v\| v.len()));`, `untrack(\|\| …)`, `on_cleanup(\|\| …)` |
| Eventos | `.on_click(\|e\| …)`, `.on_input(\|e\| …)`, `.on("keydown", \|e\| …)`; `e.value()`, `e.key()` |
| Dinámico | `.text_dyn(\|\| …)`, `.attr_dyn(name, \|\| …)`, `.class_if(class, \|\| …)`, `.bool_attr(name, \|\| …)` |
| Registrar | `#[wasm_bindgen(start)] pub fn start() { paquets::hydrate(&[("counter", counter)]); }` |
| Colocar | `island("counter", "0", counter)` |
| Compilar | `paquets dev`, `paquets build` → `pkg/` |

Consulta [Islas](islands.md).

## Pruebas

| Tarea | Código |
|---|---|
| Cliente | `let client = app().test().await;` |
| Peticiones | `client.get("/")`, `.post(...)`, `.put`, `.patch`, `.delete`, y luego `.await` |
| Cuerpo, cabecera | `.json(&json!({...}))`, `.form(&[("k", "v")])`, `.header("k", "v")` |
| Respuesta | `res.status()`, `res.text()`, `res.json::<T>()`, `res.header("k")` |
| Muchas peticiones | `app.security(paquets::shield().without("rate_limit").build());` |

Consulta [Pruebas](testing.md).

## Variables de entorno

| Variable | Efecto |
|---|---|
| `PORT` | el puerto (por defecto `3000`) |
| `DATABASE_URL` | la base de datos (por defecto: en memoria) |
| `DYNO` | la define Heroku: la IP del cliente se lee de `X-Forwarded-For` |

## Despliegue

| Tarea | Dónde |
|---|---|
| Heroku con un toque | `https://heroku.com/deploy?template=https://github.com/<you>/<repo>` |
| Despliegues automáticos en Heroku | secreto de GitHub `HEROKU_API_KEY`, variable `HEROKU_APP_NAME` (opcional `HEROKU_REGION`) |
| Base de datos en Heroku | Resources → Heroku Postgres |
| Redesplegar a mano | Actions → Deploy to Heroku → Run workflow |
| Docker | `docker build -t my-app . && docker run -p 8080:8080 my-app` |
| Evitar la fusión automática | llama a la rama `wip/...` |
| Publicar en crates.io | secreto `CARGO_REGISTRY_TOKEN` y luego Releases → Draft a new release → `v0.1.0` |

Consulta [Heroku](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Automatización](automation.md) y [Publicar en crates.io](publish-crates-io.md).
