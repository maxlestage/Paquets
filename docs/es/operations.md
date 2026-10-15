# Operaciones

Lo que una aplicación en marcha necesita además de sus páginas: la documentación de su API, la lista de sus rutas, migraciones y datos de ejemplo, comprobaciones de salud y métricas, y un servidor de desarrollo que recarga las páginas solo.

## Documentación de la API

```rust
app.api::<Todo>("/api/todos");
app.api_docs("Todos API"); // GET /openapi.json, GET /docs
```

`/openapi.json` es un documento OpenAPI 3.1, construido a partir de las rutas al arrancar; `/docs` es Swagger UI, para leer y probar la API en un navegador (cargado desde jsDelivr, permitido por la Content-Security-Policy propia de la página). Las seis rutas de cada `app.api::<M>()` se describen con un JSON Schema del modelo (tipos, campos opcionales, clave generada); describe tus propias rutas con el `.doc(Operation…)` de Vitesse:

```rust
use paquets::vitesse::openapi::Operation;

app.get("/stats", stats)
    .doc(Operation::new().summary("Estadísticas").tag("stats"));
```

Las páginas de la [autenticación](authentication.md) y del [panel de administración](admin.md) quedan fuera del documento.

## La lista de rutas

```text
$ paquets routes
METHOD  PATH                  DESCRIPTION
GET     /
GET     /admin                Admin: dashboard (page)
GET     /api/todos            List the Todo rows
POST    /api/todos            Create a Todo
...
45 routes
```

`paquets routes` ejecuta la aplicación con `PAQUETS_COMMAND=routes`: muestra sus rutas y se detiene, sin abrir la base de datos. En el código, `app.routes()` devuelve la misma lista.

## Migraciones y datos de ejemplo

`app.model::<M>()` y `app.api::<M>()` crean las tablas que faltan. Para cambiar tablas existentes, usa migraciones versionadas (el `Migrator` de memoire): cada una se ejecuta una vez, en una transacción, al arrancar y con `paquets db:migrate`.

```rust
use paquets::prelude::*;

let mut app = App::new();
app.migrate(
    Migrator::new()
        .add(Migration::new(1, "create_posts").create_table::<Post>())
        .add(Migration::new(2, "add_slug").sql("ALTER TABLE posts ADD COLUMN slug TEXT")),
);
app.seed(|db| async move {
    Post { id: 0, title: "Hola".into() }.insert(db).await?;
    Ok(())
});
```

`paquets make:migration <nombre>` escribe una migración por archivo (ver [Generadores](generators.md)). `paquets db:seed` ejecuta las funciones `app.seed(…)` — nunca al arrancar, a diferencia de `app.on_start(…)`. Ambos comandos usan `DATABASE_URL`, como la aplicación.

## Comprobaciones de salud

```rust
app.health("/health");
```

`GET /health` responde `200` con `{"status": "ok", "database": {"kind": "postgres", "ok": true}, "uptime_seconds": 42, "paquets": "0.1.1"}`, o `503` cuando la base de datos no responde (`SELECT 1`). Apunta ahí los balanceadores de carga, las sondas de Kubernetes o los monitores de disponibilidad. Los proyectos creados con `paquets new` la tienen.

## Métricas

```rust
app.metrics("/metrics");
```

Cada petición se cuenta, por clase de estado, con un histograma de duraciones; `GET /metrics` las sirve en el formato de texto de Prometheus (`paquets_requests_total`, `paquets_request_duration_seconds`, `paquets_requests_in_flight`, `paquets_uptime_seconds`). Responde a las peticiones de esta máquina o, cuando la variable de entorno `METRICS_TOKEN` está definida, a las que envían `Authorization: Bearer <token>`; las demás reciben un `403`.

## Recarga automática

```sh
paquets dev --watch
```

compila y arranca la aplicación, y la vuelve a compilar y arrancar cada vez que cambia un archivo de `src/`, `public/`, `templates/`, `views/` o `Cargo.toml`. Las páginas abiertas en el navegador se recargan solas cuando el nuevo servidor está listo: solo en este modo (`PAQUETS_LIVE_RELOAD=1`), cada `Page` escucha `/__paquets/live-reload` (Server-Sent Events, unos pocos bytes, permitidos por la CSP de la página). Un error de compilación deja el servidor detenido hasta el siguiente cambio.

## Próximos pasos

- [Generadores](generators.md): `make:migration` y los demás.
- [Herramienta de línea de comandos](cli.md): todos los comandos.
- [Desplegar con Docker](deploy-docker.md): las comprobaciones de salud en producción.
