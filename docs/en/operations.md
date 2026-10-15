# Operations

What a running app needs besides its pages: documentation of its API, the list of its routes, migrations and example data, health checks and metrics, and a development server that reloads the pages by itself.

## API documentation

```rust
app.api::<Todo>("/api/todos");
app.api_docs("Todos API"); // GET /openapi.json, GET /docs
```

`/openapi.json` is an OpenAPI 3.1 document, built from the routes when the app starts; `/docs` is Swagger UI, to read and try the API in a browser (loaded from jsDelivr, allowed by the page's own Content-Security-Policy). The six routes of every `app.api::<M>()` are described with a JSON Schema of the model (types, optional fields, generated key); describe your own routes with Vitesse's `.doc(Operation…)`:

```rust
use paquets::vitesse::openapi::Operation;

app.get("/stats", stats)
    .doc(Operation::new().summary("Statistics").tag("stats"));
```

The pages of [authentication](authentication.md) and of the [admin panel](admin.md) are left out of the document.

## The route list

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

`paquets routes` runs the app with `PAQUETS_COMMAND=routes`: it prints its routes and stops, without opening the database. In code, `app.routes()` returns the same list.

## Migrations and seeds

`app.model::<M>()` and `app.api::<M>()` create missing tables. To change existing ones, use versioned migrations (memoire's `Migrator`): each runs once, in a transaction, at startup and with `paquets db:migrate`.

```rust
use paquets::prelude::*;

let mut app = App::new();
app.migrate(
    Migrator::new()
        .add(Migration::new(1, "create_posts").create_table::<Post>())
        .add(Migration::new(2, "add_slug").sql("ALTER TABLE posts ADD COLUMN slug TEXT")),
);
app.seed(|db| async move {
    Post { id: 0, title: "Hello".into() }.insert(db).await?;
    Ok(())
});
```

`paquets make:migration <name>` writes one migration per file (see [Generators](generators.md)). `paquets db:seed` runs the `app.seed(…)` hooks — never at startup, unlike `app.on_start(…)`. Both commands use `DATABASE_URL`, like the app.

## Health checks

```rust
app.health("/health");
```

`GET /health` answers `200` with `{"status": "ok", "database": {"kind": "postgres", "ok": true}, "uptime_seconds": 42, "paquets": "0.1.1"}`, or `503` when the database does not answer (`SELECT 1`). Point load balancers, Kubernetes probes or uptime monitors at it. Projects made by `paquets new` have it.

## Metrics

```rust
app.metrics("/metrics");
```

Every request is counted, by status class, with a duration histogram; `GET /metrics` serves them in Prometheus' text format (`paquets_requests_total`, `paquets_request_duration_seconds`, `paquets_requests_in_flight`, `paquets_uptime_seconds`). It answers requests from this machine, or, when the `METRICS_TOKEN` environment variable is set, those with `Authorization: Bearer <token>`; the others get a `403`.

## Live reload

```sh
paquets dev --watch
```

builds and starts the app, then rebuilds and restarts it whenever a file of `src/`, `public/`, `templates/`, `views/` or `Cargo.toml` changes. Pages opened in the browser reload by themselves once the new server is up: in this mode only (`PAQUETS_LIVE_RELOAD=1`), every `Page` listens to `/__paquets/live-reload` (Server-Sent Events, a few bytes, allowed by the page's CSP). A build error leaves the old server stopped until the next change.

## Next steps

- [Generators](generators.md): `make:migration` and the others.
- [Command line tool](cli.md): every command.
- [Deploy with Docker](deploy-docker.md): health checks in production.
