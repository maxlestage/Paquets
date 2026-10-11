# Les quatre paquets

Paquets repose sur quatre paquets du même auteur : vitesse, securite, memoire-orm et active. Chacun est une bibliothèque complète, que vous pouvez aussi utiliser seule. Paquets les relie, choisit les réglages par défaut et les réexporte : une dépendance et un import suffisent. Cette page montre ce qu'apporte chaque paquet, comment Paquets s'en sert, et comment atteindre les parties que le prélude n'inclut pas.

## Vue d'ensemble

| Paquet | Rôle | Réexporté sous | Dépôt | Documentation |
|---|---|---|---|---|
| **vitesse** | serveur HTTP dans le style d'Express.js : routes, middlewares, routeurs, WebSocket, HTTP/3, client de test | `paquets::vitesse` | [maxlestage/Vitesse](https://maxlestage.github.io/Vitesse/) | [docs](https://maxlestage.github.io/Vitesse/fr/docs/README/) |
| **securite** | middlewares de sécurité : WAF, limitation de débit, CSRF, CORS, en-têtes de sécurité et CSP, blocage des robots, authentification… | `paquets::securite` | [maxlestage/Securite](https://maxlestage.github.io/Securite/) | [docs](https://maxlestage.github.io/Securite/fr/docs/introduction/) |
| **memoire-orm** | ORM asynchrone typé, SQL généré à la compilation : mémoire, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB | `paquets::orm` | [maxlestage/Memoire](https://maxlestage.github.io/Memoire/) | [docs](https://maxlestage.github.io/Memoire/fr/docs/introduction/) |
| **active** | front-end : composants écrits comme des fonctions Rust, `use_state`, rendu serveur avec toutes les balises SEO, îlots WebAssembly | `paquets::active` | [maxlestage/Active](https://maxlestage.github.io/Active/) | [docs](https://maxlestage.github.io/Active/fr/docs/introduction/) |

Paquets réexporte aussi `paquets::serde` et `paquets::serde_json` (et `json!` dans le prélude), pour que `#[model]` et `#[data]` fonctionnent sans ajouter serde à votre `Cargo.toml`, ainsi que `paquets::test`, les types du client de test.

## vitesse : le serveur

[vitesse](https://maxlestage.github.io/Vitesse/) est le serveur HTTP, avec son propre moteur HTTP/1.1 au-dessus de tokio. Il traite environ 291 000 requêtes par seconde sur 2 cœurs pour `GET /`, plus qu'actix-web et axum sur la même machine, et 2,78 millions avec le pipelining.

**Dans Paquets :** `App` contient une `vitesse::App` et donne accès à toutes ses méthodes (elle se déréférence vers elle) : `get`, `post`, `ws`, `middleware`, `mount`, `state`, `fallback`, `on_error`, `body_limit`, `static_dir`, `workers`… Les handlers reçoivent une `vitesse::Request` et renvoient tout ce qui implémente `vitesse::IntoResponse`. Le client de test de `app.test()` est le `TestClient` de vitesse.

**Dans le prélude :** `Request`, `Response`, `IntoResponse`, `Json`, `Html`, `Redirect`, `Cookie`, `StatusCode`, `Method`, `Router`, `Next`, `HandlerExt`, et les modules `res`, `middleware` et `ws`.

**Au-delà du prélude**, via `paquets::vitesse` :

```rust
use std::time::Duration;

let mut app = App::new();
// Les middlewares fournis par vitesse : CORS, timeout, logger, helmet.
app.middleware(middleware::timeout(Duration::from_secs(10)));
// Un dossier avec ses propres réglages de cache.
app.serve_dir("/assets", paquets::vitesse::ServeDir::new("assets").max_age(Duration::from_secs(3600)));
```

- `paquets::vitesse::tokio` : le runtime tokio sur lequel tourne vitesse (`tokio::spawn`, `tokio::sync`, `tokio::time`…), sans ajouter tokio à vos dépendances ;
- `paquets::vitesse::Error` : le type d'erreur propre à vitesse, enveloppé par `paquets::Error` (`err.into_inner()` vous le rend) ;
- `paquets::vitesse::ws::Upgrade` : les poignées de main WebSocket avec sous-protocoles et limites de taille ;
- HTTP/3 avec la feature `http3` (voir [le guide HTTP/3 de vitesse](https://maxlestage.github.io/Vitesse/fr/docs/http3/)).

## securite : le bouclier

[securite](https://maxlestage.github.io/Securite/) est un ensemble de 19 guards de sécurité, assemblés dans un `Shield`. Sa chaîne recommandée coûte environ 6 µs par requête. Il est écrit pour l'écosystème tower (axum, hyper, tonic) et ne contient aucun code `unsafe`.

**Dans Paquets :** `App::new()` place `paquets::shield().build()` devant chaque requête : le préréglage recommandé de securite, avec l'IP du client lue dans `X-Forwarded-For` sur Heroku. Paquets fait le pont entre les requêtes de vitesse et les guards de securite : le bouclier voit la ligne de requête et les en-têtes, ajoute ses en-têtes à chaque réponse, et transmet ce qu'il a trouvé à vos handlers (l'IP du client, l'utilisateur authentifié, l'identifiant de requête). Les pages calculent leur propre Content-Security-Policy avec les fonctions de hachage de securite.

**Pour l'utiliser :**

```rust
use paquets::securite::prelude::*;

app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
```

`paquets::securite::prelude::*` importe tous les guards (`RateLimit`, `Waf`, `Csrf`, `Cors`, `Auth`, `IpFilter`, `HostFilter`, `SecurityHeaders`…), `Shield`, `ShieldBuilder`, `ClientIpSource`, `ErrorFormat`, `ClientIp` et `Principal`. `paquets::securite::headers::ContentSecurityPolicy` construit des politiques. Voir [Sécurité](security.md).

## memoire-orm : la base de données

[memoire-orm](https://maxlestage.github.io/Memoire/) fait correspondre des structs Rust à des tables, et permet d'écrire les requêtes en Rust, vérifiées par le compilateur. Le SQL des opérations par clé primaire est généré à la compilation, et le même code fonctionne sur six backends.

**Dans Paquets :**

- `#[model]` ajoute `#[derive(Model)]` (et serde) à votre struct, et fait pointer le code généré vers `paquets::orm` : votre projet n'a donc pas besoin de sa propre dépendance `memoire-orm` ;
- `Db` est une poignée unique pour tous les backends, choisie par `DATABASE_URL`, qui implémente les traits `Executor` et `Database` de memoire : toutes les méthodes de modèle et toutes les requêtes acceptent `req.db()` ;
- `app.api::<M>()` et `app.model::<M>()` créent les tables au démarrage ; les erreurs de base de données deviennent des `404`, `409` ou `400` grâce à `?`.

**Dans le prélude :** les traits `Model`, `Executor`, `Database` et `Transaction`, qui donnent leurs méthodes aux modèles et à la base de données.

**Au-delà du prélude**, via `paquets::orm` : `Migrator` et `Migration`, `Dialect`, la macro `params!` pour le SQL brut, `Page` (le résultat de `paginate`), les colonnes `Json<T>`, et les backends eux-mêmes (`SqliteDb`, `PgPool`, `MySqlPool`, `RedisDb`, `MongoDb`, `MemoryDb`), pour ouvrir une base de données avec vos propres réglages et la passer à `app.database(Db::from(...))`. Voir [Base de données](database.md).

## active : le front-end

[active](https://maxlestage.github.io/Active/) construit les vues avec une fonction par balise HTML, les rend sur le serveur avec toutes les balises SEO, et transforme les parties marquées d'une page en îlots WebAssembly. Il n'a ni macros, ni langage de templates, ni dépendance côté serveur ; son bundle compteur pèse 36,8 ko compressé.

**Dans Paquets :** `Page` enveloppe le `Document` d'active : elle y ajoute le thème, la Content-Security-Policy et le chargement du bundle des îlots (trouvé dans `pkg/` au démarrage). Les îlots sont ceux d'active, hydratés dans le navigateur par `paquets::hydrate` (feature `client`).

**Dans le prélude :** toutes les fonctions de balises HTML (`div()`, `p()`, `a()`…) et le module `html` (pour `html::main()`), `Element`, `Node`, `Seo`, `State`, `use_state`, `effect`, `island`, `el`, `text`, `text_dyn`, `fragment`, `fragment_dyn` et `keyed`.

**Au-delà du prélude**, via `paquets::active` : `Document`, pour rendre une page sans le thème ni la politique de sécurité de `Page`, par exemple pour exporter des fichiers statiques. Avec la feature `client` (côté navigateur) : `paquets::hydrate`, `paquets::mount_to` et `paquets::mount` pour les applications entièrement rendues dans le navigateur, `paquets::navigate`, `paquets::location`, `paquets::query` et `paquets::query_param` pour le routage sans rechargement. Voir [Îlots](islands.md) et [la documentation d'active](https://maxlestage.github.io/Active/fr/docs/introduction/).

## Correspondance des noms

| Dans Paquets | Dans le paquet |
|---|---|
| `App` | `vitesse::App`, plus le bouclier, la base de données, l'API REST, les fichiers statiques et les îlots |
| `app.run()` | `vitesse::App::run(port)`, avec le port de `PORT` |
| `app.security(shield)` | un `securite::Shield` |
| `paquets::shield()` | `securite::ShieldBuilder::recommended()` avec l'IP du client de Heroku |
| `#[model]` | `#[derive(memoire_orm::Model, Serialize, Deserialize, Clone, Debug)]` avec `#[memoire(crate = "::paquets::orm")]` |
| `#[data]` | `#[derive(Serialize, Deserialize, Clone, Debug)]` |
| `Db`, `req.db()` | n'importe quel backend de memoire, derrière un seul type |
| `Page` | `active::Document`, plus le thème, la CSP et le bundle |
| `paquets::Error`, `Result<T>` | `vitesse::Error`, avec le bon statut pour les erreurs de memoire |

## Utiliser un paquet seul

Chaque paquet est publié séparément et fonctionne sans Paquets :

- **vitesse** pour un serveur sans base de données ni pages, ou avec vos propres choix ;
- **securite** devant un service axum, hyper ou tonic : `.layer(Shield::recommended())` ;
- **memoire-orm** dans n'importe quel programme Rust qui stocke des données, un outil en ligne de commande ou un worker ;
- **active** pour un site statique, une application côté client, ou des pages servies par un autre framework (sa documentation a un guide pour axum).

Leurs README expliquent comment démarrer. Les quatre utilisent l'édition 2024. Comme Paquets, les quatre paquets sont des logiciels propriétaires de Maxime Nathan Lestage, libres d'utilisation comme dépendances (voir [Quelle licence ?](faq.md#quelle-licence-)).

## Versions

Paquets 0.1 dépend de vitesse 0.1, securite 0.1, memoire-orm 0.1 et active 0.1. Tant qu'ils ne sont pas sur crates.io, Cargo les récupère depuis leurs dépôts GitHub ; voir [Publier sur crates.io](publish-crates-io.md) pour l'ordre de publication.

## Étapes suivantes

- [Introduction](introduction.md) : ce que Paquets ajoute aux quatre paquets.
- [Aide-mémoire](cheatsheet.md) : toute l'API sur une seule page.
- [FAQ](faq.md) : les questions fréquentes.
