# Les quatre paquets

Paquets repose sur quatre paquets du même auteur : Vitesse, Securite, memoire-orm et Active. Chacun est une bibliothèque complète, que vous pouvez aussi utiliser seule. Paquets les relie, choisit les réglages par défaut et les réexporte : une dépendance et un import suffisent. Cette page montre ce qu'apporte chaque paquet, comment Paquets s'en sert, et comment atteindre les parties que le prélude n'inclut pas.

## Vue d'ensemble

| Paquet | Rôle | Réexporté sous | Dépôt | Documentation |
|---|---|---|---|---|
| **Vitesse** | serveur HTTP dans le style d'Express.js : routes, middlewares, routeurs, WebSocket, HTTPS, HTTP/2 et HTTP/3, sessions, templates, client de test | `paquets::vitesse` | [maxlestage/Vitesse](https://vitesse-f16135654c3d.herokuapp.com/) | [docs](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/introduction/) |
| **Securite** | middlewares de sécurité : WAF, limitation de débit, CSRF, CORS, en-têtes de sécurité et CSP, blocage des robots, authentification… | `paquets::securite` | [maxlestage/Securite](https://securite-2db4c1994be7.herokuapp.com/) | [docs](https://securite-2db4c1994be7.herokuapp.com/fr/docs/introduction/) |
| **memoire-orm** | ORM asynchrone typé, SQL généré à la compilation : mémoire, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB | `paquets::orm` | [maxlestage/Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/) | [docs](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/introduction/) |
| **active** | front-end : composants écrits comme des fonctions Rust, `use_state`, rendu serveur avec toutes les balises SEO, îlots WebAssembly | `paquets::active` | [maxlestage/Active](https://active-c1883a966650.herokuapp.com/) | [docs](https://active-c1883a966650.herokuapp.com/fr/docs/introduction/) |

Paquets réexporte aussi `paquets::serde` et `paquets::serde_json` (et `json!` dans le prélude), pour que `#[model]` et `#[data]` fonctionnent sans ajouter serde à votre `Cargo.toml`, ainsi que `paquets::test`, les types du client de test.

## Vitesse : le serveur

[Vitesse](https://vitesse-f16135654c3d.herokuapp.com/) est le serveur HTTP, avec son propre moteur HTTP/1.1 au-dessus de tokio. Il traite environ 291 000 requêtes par seconde sur 2 cœurs pour `GET /`, plus qu'actix-web et axum sur la même machine, et 2,78 millions avec le pipelining.

**Dans Paquets :** `App` contient une `vitesse::App` et donne accès à toutes ses méthodes (elle se déréférence vers elle) : `get`, `post`, `ws`, `middleware`, `mount`, `state`, `fallback`, `on_error`, `body_limit`, `static_dir`, `workers`… Les handlers reçoivent une `vitesse::Request` et renvoient tout ce qui implémente `vitesse::IntoResponse`. Le client de test de `app.test()` est le `TestClient` de Vitesse.

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

- `paquets::vitesse::tokio` : le runtime tokio sur lequel tourne Vitesse (`tokio::spawn`, `tokio::sync`, `tokio::time`…), sans ajouter tokio à vos dépendances ;
- `paquets::vitesse::Error` : le type d'erreur propre à Vitesse, enveloppé par `paquets::Error` (`err.into_inner()` vous le rend) ;
- `paquets::vitesse::ws::Upgrade` : les poignées de main WebSocket avec sous-protocoles et limites de taille ;
- `paquets::vitesse::sse` : les Server-Sent Events, un flux d'événements vers le navigateur ;
- les middlewares `request_id`, `basic_auth`, `bearer_auth`, `concurrency_limit` et `rate_limit`, à côté de `cors`, `timeout`, `logger` et `helmet`.

Les options de Vitesse sont des features Cargo de Paquets, avec les mêmes noms :

| Feature | Apporte | Guide de Vitesse |
|---|---|---|
| `tls`, `http2`, `http3` | HTTPS dans le serveur (`app.tls(...)`), HTTP/2, HTTP/3 sur QUIC | [HTTPS](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/https/), [HTTP/3](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/http3/) |
| `session` | les cookies signés (`app.cookie_secret(...)`) et les sessions (`middleware::session(...)`, `req.session()`) | [Sessions](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/sessions/) |
| `templates` | les templates minijinja : `app.views("views")`, `req.render("page.html", json!({...}))` | [Templates](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/templates/) |
| `multipart` | les envois de fichiers : `req.multipart()` | [Requêtes](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/requests/) |
| `compress` | les réponses en Brotli ou en gzip (`middleware::compress()`), les corps de requête compressés (`middleware::decompress()`) | [Middlewares](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/middleware/) |
| `tower` | les couches tower comme middlewares : `middleware::layer(...)` | [Middlewares](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/middleware/) |

Les pages rendues avec `req.render` reçoivent la Content-Security-Policy du bouclier, qui refuse les scripts et les styles en ligne : placez-les dans `public/`, ou construisez la page avec `Page`, qui autorise les siens (voir [Sécurité](security.md#les-en-têtes-de-sécurité-et-la-content-security-policy)).

## Securite : le bouclier

[Securite](https://securite-2db4c1994be7.herokuapp.com/) est un ensemble de 19 guards de sécurité, assemblés dans un `Shield`. Sa chaîne recommandée coûte environ 6 µs par requête. Il est écrit pour Vitesse (et pour l'écosystème tower : hyper, tonic) et ne contient aucun code `unsafe`.

**Dans Paquets :** `App::new()` place `paquets::shield().build()` devant chaque requête : le préréglage recommandé de Securite, avec l'IP du client lue dans `X-Forwarded-For` sur Heroku. Paquets fait le pont entre les requêtes de Vitesse et les guards de Securite : le bouclier voit la ligne de requête et les en-têtes, ajoute ses en-têtes à chaque réponse, et transmet ce qu'il a trouvé à vos handlers (l'IP du client, l'utilisateur authentifié, l'identifiant de requête). Les pages calculent leur propre Content-Security-Policy avec les fonctions de hachage de Securite.

**Pour l'utiliser :**

```rust
use paquets::securite::prelude::*;

app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
```

`paquets::securite::prelude::*` importe tous les guards (`RateLimit`, `Waf`, `Csrf`, `Cors`, `Auth`, `IpFilter`, `HostFilter`, `SecurityHeaders`…), `Shield`, `ShieldBuilder`, `ClientIpSource`, `ErrorFormat`, `ClientIp` et `Principal`. `paquets::securite::headers::ContentSecurityPolicy` construit des politiques. Voir [Sécurité](security.md).

**Sa crate sœur, Pratique.** Le dépôt de Securite contient aussi `pratique`, des middlewares Vitesse du quotidien qui n'ont rien à voir avec la sécurité : `ETag` et `304`, `Cache-Control`, un cache de réponses, les journaux d'accès, les métriques Prometheus, les sondes de santé, le mode maintenance, les redirections, les barres obliques finales, la langue du visiteur. Ils fonctionnent tels quels avec une application Paquets, puisque `App` est une application Vitesse :

```toml
[dependencies]
pratique = { git = "https://github.com/maxlestage/Securite" }
```

```rust
use pratique::prelude::*;

app.middleware(Health::new().version(env!("CARGO_PKG_VERSION"))); // GET /health
app.middleware(ETag::new());
```

Voir la [documentation de Pratique](https://securite-2db4c1994be7.herokuapp.com/fr/docs/pratique/).

## memoire-orm : la base de données

[memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/) fait correspondre des structs Rust à des tables, et permet d'écrire les requêtes en Rust, vérifiées par le compilateur. Le SQL des opérations par clé primaire est généré à la compilation, et le même code fonctionne sur six backends.

**Dans Paquets :**

- `#[model]` ajoute `#[derive(Model)]` (et serde) à votre struct, et fait pointer le code généré vers `paquets::orm` : votre projet n'a donc pas besoin de sa propre dépendance `memoire-orm` ;
- `Db` est une poignée unique pour tous les backends, choisie par `DATABASE_URL`, qui implémente les traits `Executor` et `Database` de Memoire : toutes les méthodes de modèle et toutes les requêtes acceptent `req.db()` ;
- `app.api::<M>()` et `app.model::<M>()` créent les tables au démarrage ; les erreurs de base de données deviennent des `404`, `409` ou `400` grâce à `?`.

**Dans le prélude :** les traits `Model`, `Executor`, `Database` et `Transaction`, qui donnent leurs méthodes aux modèles et à la base de données, et `LoadRelated`, qui charge les lignes liées d'une liste en une requête par niveau (`users.load_children::<Post>(db)`).

**Ce que Memoire apporte depuis peu**, le tout disponible via `req.db()` : les relations lues dans les clés étrangères ([Lignes liées](database.md#lignes-liées)), la recherche plein texte avec `Column::matches`, les index sur plusieurs colonnes, `find_many` et `update_only`, une base en mémoire gardée dans un fichier (`DATABASE_URL=memory://app.memoire`, chiffrée avec la feature `encryption`), et le stockage dans le navigateur pour les îlots (feature `browser-storage`, voir [Îlots](islands.md#garder-des-données-dans-le-navigateur)).

**Au-delà du prélude**, via `paquets::orm` : `Migrator` et `Migration`, `Dialect`, la macro `params!` pour le SQL brut, `Page` (le résultat de `paginate`), les colonnes `Json<T>`, `Observed` (une poignée qui annonce chaque écriture faite à travers elle), `StorageOptions`, et les backends eux-mêmes (`SqliteDb`, `PgPool`, `MySqlPool`, `RedisDb`, `MongoDb`, `MemoryDb`), pour ouvrir une base de données avec vos propres réglages et la passer à `app.database(Db::from(...))`. Voir [Base de données](database.md).

## Active : le front-end

[Active](https://active-c1883a966650.herokuapp.com/) construit les vues avec une fonction par balise HTML, les rend sur le serveur avec toutes les balises SEO, et transforme les parties marquées d'une page en îlots WebAssembly. Il n'a ni macros, ni langage de templates, ni dépendance côté serveur ; son bundle compteur pèse 36,8 ko compressé.

**Dans Paquets :** `Page` enveloppe le `Document` d'active : elle y ajoute le thème, la Content-Security-Policy et le chargement du bundle des îlots (trouvé dans `pkg/` au démarrage). Les îlots sont ceux d'active, hydratés dans le navigateur par `paquets::hydrate` (feature `client`).

**Dans le prélude :** toutes les fonctions de balises HTML (`div()`, `p()`, `a()`…) et le module `html` (pour `html::main()`), `Element`, `Node`, `Seo`, `State`, `use_state`, `effect`, `memo` (un état dérivé qui ne notifie que quand sa valeur change), `untrack`, `on_cleanup`, `island`, `el`, `text`, `text_dyn`, `fragment`, `fragment_dyn`, `keyed`, `indexed` (une liste dont les lignes suivent leur position) et `switch` (une vue qui garde sa page pendant que les paramètres de la page changent).

**Au-delà du prélude**, via `paquets::active` : `Document`, pour rendre une page sans le thème ni la politique de sécurité de `Page`, par exemple pour exporter des fichiers statiques. Avec la feature `client` (côté navigateur) : `paquets::hydrate`, `paquets::mount_to` et `paquets::mount` pour les applications entièrement rendues dans le navigateur, `paquets::navigate`, `paquets::location`, `paquets::query` et `paquets::query_param` pour le routage sans rechargement. Voir [Îlots](islands.md) et [la documentation d'active](https://active-c1883a966650.herokuapp.com/fr/docs/introduction/).

## Correspondance des noms

| Dans Paquets | Dans le paquet |
|---|---|
| `App` | `vitesse::App`, plus le bouclier, la base de données, l'API REST, les fichiers statiques et les îlots |
| `app.run()` | `vitesse::App::run(port)`, avec le port de `PORT` |
| `app.security(shield)` | un `securite::Shield` |
| `paquets::shield()` | `securite::ShieldBuilder::recommended()` avec l'IP du client de Heroku |
| `#[model]` | `#[derive(memoire_orm::Model, Serialize, Deserialize, Clone, Debug)]` avec `#[memoire(crate = "::paquets::orm")]` |
| `#[data]` | `#[derive(Serialize, Deserialize, Clone, Debug)]` |
| `Db`, `req.db()` | n'importe quel backend de Memoire, derrière un seul type |
| `Page` | `active::Document`, plus le thème, la CSP et le bundle |
| `paquets::Error`, `Result<T>` | `vitesse::Error`, avec le bon statut pour les erreurs de Memoire |

## Utiliser un paquet seul

Chaque paquet est publié séparément et fonctionne sans Paquets :

- **Vitesse** pour un serveur sans base de données ni pages, ou avec vos propres choix ;
- **Securite** dans une application Vitesse (`app.middleware(Shield::recommended())`) ou devant un service hyper ou tonic (`.layer(Shield::recommended())`) ;
- **memoire-orm** dans n'importe quel programme Rust qui stocke des données, un outil en ligne de commande ou un worker ;
- **active** pour un site statique, une application côté client, ou des pages servies par un autre framework (sa documentation a un guide pour Vitesse).

Leurs README expliquent comment démarrer. Les quatre utilisent l'édition 2024. Comme Paquets, les quatre paquets sont des logiciels propriétaires de Maxime Nathan Lestage, libres d'utilisation comme dépendances (voir [Quelle licence ?](faq.md#quelle-licence-)).

## Versions

Paquets 0.1 dépend de Vitesse 0.1, Securite 0.1, memoire-orm 0.1 et Active 0.1. Tant qu'ils ne sont pas sur crates.io, Cargo les récupère depuis leurs dépôts GitHub ; voir [Publier sur crates.io](publish-crates-io.md) pour l'ordre de publication.

## Étapes suivantes

- [Introduction](introduction.md) : ce que Paquets ajoute aux quatre paquets.
- [Aide-mémoire](cheatsheet.md) : toute l'API sur une seule page.
- [FAQ](faq.md) : les questions fréquentes.
