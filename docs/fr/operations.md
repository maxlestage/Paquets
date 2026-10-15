# Exploitation

Ce dont une application en marche a besoin en plus de ses pages : la documentation de son API, la liste de ses routes, des migrations et des données d'exemple, des contrôles de santé et des métriques, et un serveur de développement qui recharge les pages tout seul.

## Documentation de l'API

```rust
app.api::<Todo>("/api/todos");
app.api_docs("Todos API"); // GET /openapi.json, GET /docs
```

`/openapi.json` est un document OpenAPI 3.1, construit à partir des routes au démarrage ; `/docs` est Swagger UI, pour lire et essayer l'API dans un navigateur (chargé depuis jsDelivr, autorisé par la Content-Security-Policy propre à la page). Les six routes de chaque `app.api::<M>()` sont décrites avec un JSON Schema du modèle (types, champs facultatifs, clé générée) ; décrivez vos propres routes avec le `.doc(Operation…)` de Vitesse :

```rust
use paquets::vitesse::openapi::Operation;

app.get("/stats", stats)
    .doc(Operation::new().summary("Statistiques").tag("stats"));
```

Les pages de l'[authentification](authentication.md) et du [panneau d'administration](admin.md) restent hors du document.

## La liste des routes

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

`paquets routes` lance l'application avec `PAQUETS_COMMAND=routes` : elle affiche ses routes et s'arrête, sans ouvrir la base de données. Dans le code, `app.routes()` renvoie la même liste.

## Migrations et données d'exemple

`app.model::<M>()` et `app.api::<M>()` créent les tables manquantes. Pour changer des tables existantes, utilisez des migrations versionnées (le `Migrator` de memoire) : chacune s'exécute une fois, dans une transaction, au démarrage et avec `paquets db:migrate`.

```rust
use paquets::prelude::*;

let mut app = App::new();
app.migrate(
    Migrator::new()
        .add(Migration::new(1, "create_posts").create_table::<Post>())
        .add(Migration::new(2, "add_slug").sql("ALTER TABLE posts ADD COLUMN slug TEXT")),
);
app.seed(|db| async move {
    Post { id: 0, title: "Bonjour".into() }.insert(db).await?;
    Ok(())
});
```

`paquets make:migration <nom>` écrit une migration par fichier (voir [Générateurs](generators.md)). `paquets db:seed` exécute les fonctions `app.seed(…)` — jamais au démarrage, contrairement à `app.on_start(…)`. Les deux commandes utilisent `DATABASE_URL`, comme l'application.

## Contrôles de santé

```rust
app.health("/health");
```

`GET /health` répond `200` avec `{"status": "ok", "database": {"kind": "postgres", "ok": true}, "uptime_seconds": 42, "paquets": "0.2.0"}`, ou `503` quand la base de données ne répond pas (`SELECT 1`). Branchez-y répartiteurs de charge, sondes Kubernetes ou moniteurs de disponibilité. Les projets créés par `paquets new` l'ont.

## Métriques

```rust
app.metrics("/metrics");
```

Chaque requête est comptée, par classe de statut, avec un histogramme des durées ; `GET /metrics` les sert au format texte de Prometheus (`paquets_requests_total`, `paquets_request_duration_seconds`, `paquets_requests_in_flight`, `paquets_uptime_seconds`). Il répond aux requêtes de cette machine ou, quand la variable d'environnement `METRICS_TOKEN` est définie, à celles qui envoient `Authorization: Bearer <jeton>` ; les autres reçoivent un `403`.

## Rechargement automatique

```sh
paquets dev --watch
```

compile et lance l'application, puis la recompile et la relance dès qu'un fichier de `src/`, `public/`, `templates/`, `views/` ou `Cargo.toml` change. Les pages ouvertes dans le navigateur se rechargent toutes seules une fois le nouveau serveur prêt : dans ce mode seulement (`PAQUETS_LIVE_RELOAD=1`), chaque `Page` écoute `/__paquets/live-reload` (Server-Sent Events, quelques octets, autorisés par la CSP de la page). Une erreur de compilation laisse le serveur arrêté jusqu'au changement suivant.

## Étapes suivantes

- [Générateurs](generators.md) : `make:migration` et les autres.
- [Ligne de commande](cli.md) : toutes les commandes.
- [Déployer avec Docker](deploy-docker.md) : les contrôles de santé en production.
