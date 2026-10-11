# Configuration

Une application Paquets fonctionne sans configuration. Quand vous avez besoin de changer quelque chose, cela se passe à trois endroits : les variables d'environnement (définies par votre hébergeur, ou par vous), les méthodes de `App`, et les features Cargo. Cette page les liste toutes, montre comment lire vos propres réglages et partager un état entre les requêtes, et explique comment le serveur fonctionne.

## Variables d'environnement

| Variable | Sert à | Par défaut |
|---|---|---|
| `PORT` | le port sur lequel le serveur écoute | `3000` |
| `DATABASE_URL` | la base de données (voir [Base de données](database.md#choisir-la-base-de-données)) | en mémoire |
| `DYNO` | définie par Heroku ; le bouclier par défaut lit alors l'IP du client dans `X-Forwarded-For` | — |

Définissez-les dans votre shell, le temps d'un lancement :

```sh
PORT=8080 DATABASE_URL=sqlite://app.db paquets dev
```

Sur Heroku, `PORT` est définie pour vous, et `DATABASE_URL` aussi quand vous ajoutez l'add-on Heroku Postgres. Vos propres variables vont dans **Settings → Config Vars** (voir [Déployer sur Heroku](deploy-heroku-mobile.md#settings--config-vars)). Avec Docker, passez-les avec `-e NOM=valeur` (voir [Docker](deploy-docker.md)).

Paquets ne lit pas les fichiers `.env`. Si vous y tenez, la crate [`dotenvy`](https://crates.io/crates/dotenvy) en charge un en un seul appel au début de `main`.

## Les réglages de l'application

`App::new()` vous donne une application prête à l'emploi ; ces méthodes la modifient. Appelez-les avant `app.run()`, dans n'importe quel ordre.

| Méthode | Effet | Par défaut |
|---|---|---|
| `app.port(8080)` | le port, à la place de `PORT` | `PORT`, sinon `3000` |
| `app.quiet()` | désactive le journal des requêtes | une ligne par requête |
| `app.pwa(manifeste)` | application installable qui marche hors ligne (voir [Pages](pages.md#application-installable-pwa)) | non installable |
| `app.database(url_ou_db)` | la base de données, à la place de `DATABASE_URL` | `DATABASE_URL`, sinon en mémoire |
| `app.security(bouclier)` | un autre bouclier de sécurité, ou `None` pour le désactiver | `paquets::shield()` |
| `app.on_start(hook)` | du code exécuté une fois la base de données prête | — |
| `app.api::<M>(chemin)` | une API REST et une table pour un modèle | — |
| `app.model::<M>()` | une table pour un modèle | — |

`App` donne aussi accès à toutes les méthodes de l'application de vitesse :

| Méthode | Effet | Par défaut |
|---|---|---|
| `app.state(valeur)` | un état partagé, lu avec `req.state::<T>()` (voir plus bas) | — |
| `app.body_limit(octets)` | le plus gros corps qu'un handler peut lire | 2 Mio, comme la limite de corps du bouclier |
| `app.middleware(m)` | un middleware global (voir [Routes](routing.md#middlewares)) | — |
| `app.mount(préfixe, routeur)` | un routeur sous un préfixe | — |
| `app.static_dir(préfixe, dossier)` | sert un dossier sous un préfixe | `public/` sur `/` |
| `app.fallback(handler)` | la réponse quand aucune route ne correspond | `404` en JSON |
| `app.on_error(f)` | le format de toutes les réponses d'erreur (voir [Erreurs](errors.md#changer-le-format-de-toutes-les-erreurs)) | `{"error": "..."}` |
| `app.workers(n)` | le nombre de threads du serveur (voir plus bas) | un par cœur de processeur |
| `app.thread_per_core(false)` | le runtime de tokio avec vol de tâches, au lieu d'une boucle d'événements par cœur (voir plus bas) | `true` sous Linux |

Les méthodes qui renvoient `&mut App` peuvent s'enchaîner : `app.port(8080).quiet();`. Seuls `run()`, `serve()` et `test()` consomment l'application : ils viennent donc en dernier.

## Vos propres réglages

Lisez vos réglages une seule fois, au démarrage, dans une struct, et gardez cette struct dans l'état de l'application :

```rust
use paquets::prelude::*;

/// Les réglages de l'application, lus une seule fois au démarrage.
struct Settings {
    site_name: String,
    admin_email: String,
}

impl Settings {
    fn from_env() -> Settings {
        Settings {
            site_name: std::env::var("SITE_NAME").unwrap_or_else(|_| "Mon site".into()),
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
            .body(h1().text(format!("Bienvenue sur {}", settings.site_name)))
    });
    app
}
```

Gardez les secrets (clés d'API, mots de passe) dans des variables d'environnement, jamais dans le code ni dans Git.

## État partagé

`app.state(valeur)` enregistre une valeur que chaque requête peut lire avec `req.state::<T>()` : des réglages, un cache, un compteur, un canal (voir le salon de discussion dans [WebSocket](websocket.md#un-salon-de-discussion)).

- L'état se retrouve **par son type** : une valeur par type. Donnez à chaque élément d'état son propre type (une struct), plutôt que d'enregistrer une simple `String`.
- Le type doit être `Send + Sync + 'static`, car tous les threads du serveur le partagent.
- `req.state::<T>()` renvoie un `&T` qui vit aussi longtemps que l'application : ni clone, ni verrou.
- Demander un type qui n'a jamais été enregistré provoque une panique, et le client reçoit une `500`. `req.try_state::<T>()` renvoie plutôt une `Option`.

Pour modifier un état depuis une requête, utilisez un type fait pour être partagé entre threads : un atomique pour un compteur, un `Mutex` ou un `RwLock` pour une collection.

```rust
use std::sync::atomic::{AtomicU64, Ordering};

app.state(AtomicU64::new(0));
app.get("/visits", |req: Request| async move {
    let n = req.state::<AtomicU64>().fetch_add(1, Ordering::Relaxed) + 1;
    format!("visite n°{n}")
});
```

L'état vit en mémoire : il est perdu au redémarrage du serveur, et chaque serveur en a sa propre copie. Les données qui doivent durer ont leur place dans la [base de données](database.md). D'autres façons de faire dans [le guide de l'état de vitesse](https://maxlestage.github.io/Vitesse/fr/docs/state/).

## Le fonctionnement du serveur

`app.run()` prépare l'application (base de données, tables, hooks `on_start`, bouclier, fichiers statiques, îlots), affiche la ligne de démarrage, et sert les requêtes jusqu'à `Ctrl+C` ou `SIGTERM` :

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: none
```

- Il écoute sur **toutes les interfaces réseau** (`0.0.0.0`), ce dont ont besoin les conteneurs et les hébergeurs comme Heroku.
- Avec la base de données en mémoire ou SQLite, il fait tourner **un thread par cœur de processeur**, chacun avec sa propre boucle d'événements : le mode le plus rapide de vitesse. `app.workers(n)` et `app.thread_per_core(false)` s'appliquent dans ce mode.
- Avec un serveur de base de données (PostgreSQL, MySQL, Redis, MongoDB), il tourne sur un runtime tokio avec un thread par cœur, partagé avec les connexions à la base. `workers` et `thread_per_core` ne s'appliquent pas.
- Sur `Ctrl+C` ou `SIGTERM`, il cesse d'accepter des connexions et laisse jusqu'à 10 secondes aux requêtes en cours pour se terminer (arrêt propre). Heroku et Docker envoient `SIGTERM` avant d'arrêter un conteneur.

## Initialisation asynchrone

`app.run()` crée son propre runtime : `main` n'a donc pas besoin d'être `async`. Quand vous devez faire un `.await` avant le démarrage de l'application (ouvrir un pool de connexions avec des réglages sur mesure, récupérer une configuration distante), rendez `main` asynchrone avec tokio et démarrez l'application avec `app.serve().await` :

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
            // 10 connexions au plus, pour une petite offre hébergée.
            let pool = PgPool::connect_with(&url, 10).await.map_err(std::io::Error::other)?;
            app.database(Db::from(pool));
        }
    }
    app.serve().await
}
```

`serve()` fait tout ce que fait `run()`, sur le runtime courant (`workers` et `thread_per_core` ne s'appliquent pas).

## Features Cargo

Les bases de données, les types de colonnes supplémentaires et HTTP/3 sont des features Cargo de la crate `paquets` : voir [Installation](installation.md#features-cargo).

## Liste de vérification pour la production

- [ ] Compilez en mode release : `paquets build` (ou `cargo build --release`). Le `Dockerfile` d'un projet généré s'en charge.
- [ ] Utilisez une base de données persistante : `DATABASE_URL` pointant vers PostgreSQL sur Heroku (pas SQLite, pas la mémoire).
- [ ] Écoutez sur `PORT` (automatique) et lancez le programme depuis le dossier qui contient `public/` et `pkg/`.
- [ ] Derrière un autre proxy que celui de Heroku, indiquez au bouclier où se trouve l'IP du client (voir [Sécurité](security.md#lip-du-client-derrière-un-proxy)).
- [ ] Gardez les secrets dans des variables d'environnement.
- [ ] Servez le site en HTTPS : Heroku s'en charge sur `*.herokuapp.com`, et sur les domaines personnalisés avec sa gestion des certificats ; ailleurs, utilisez un proxy ou un CDN.
- [ ] Protégez ce qui modifie les données : un middleware, `Auth`, ou des sessions (voir [Sécurité](security.md#authentification)).
- [ ] Lancez les tests à chaque push : le workflow de CI s'en charge (voir [Automatisation](automation.md)).

## Étapes suivantes

- [Performances](performance.md) : ce qui rend Paquets rapide, et comment le garder rapide.
- [Déployer sur Heroku](deploy-heroku-mobile.md) : les config vars et les journaux sur Heroku.
- [Sécurité](security.md) : les réglages du bouclier.
