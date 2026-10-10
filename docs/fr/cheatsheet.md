# Aide-mémoire

Tout Paquets sur une seule page. Chaque exemple suppose `use paquets::prelude::*;`. Suivez les liens pour les détails.

## Ligne de commande

| Tâche | Commande |
|---|---|
| Installer l'outil | `cargo install paquets --locked` |
| Créer un projet | `paquets new mon-app` (`--minimal` : sans îlots) |
| Lancer pendant le développement | `paquets dev` |
| Compiler pour la production | `paquets build` |
| Vérifier les outils | `paquets doctor` |
| Lancer les tests | `cargo test` |

Voir [Ligne de commande](cli.md).

## L'application

| Tâche | Code |
|---|---|
| Créer | `let mut app = App::new();` |
| Lancer (`PORT` ou 3000) | `app.run()` depuis `fn main() -> std::io::Result<()>` |
| Lancer dans `#[tokio::main]` | `app.serve().await` |
| Choisir le port | `app.port(8080);` |
| Désactiver le journal des requêtes | `app.quiet();` |
| Choisir la base de données | `app.database("sqlite://app.db");` ou `app.database(Db::memory());` |
| Modifier le bouclier | `app.security(paquets::shield().guard(...).build());` |
| Désactiver le bouclier | `app.security(None);` |
| Du code au démarrage | `app.on_start(\|db\| async move { ...; Ok(()) });` |
| État partagé | `app.state(valeur);` puis `req.state::<T>()` |
| Limite de lecture des corps | `app.body_limit(10 * 1024 * 1024);` |
| Client de test | `let client = app.test().await;` |

Voir [Configuration](configuration.md).

## Routes

| Tâche | Code |
|---|---|
| Méthodes | `app.get`, `.post`, `.put`, `.patch`, `.delete`, `.all`, `.head`, `.options` |
| Handler en closure | `app.get("/", \|_\| async { "Accueil" });` |
| Closure qui utilise la requête | `app.get("/hi/:name", \|req: Request\| async move { ... });` |
| Handler en fonction | `async fn home(req: Request) -> Result<Page> { ... }` puis `app.get("/", home);` |
| Paramètre | `/users/:id` → `req.param("id")` (`Option<&str>`) |
| Paramètre typé | `let id: i64 = req.param_as("id")?;` (400 s'il est invalide) |
| Joker | `/files/*path` → `req.param("path")` |
| Routeur | `let mut api = Router::new(); api.get(...); app.mount("/api", api);` |
| Middleware global | `app.middleware(\|req: Request, next: Next\| async move { next.run(req).await });` |
| Middleware sur une route | `app.get("/admin", handler.with(auth));` |
| Middleware sur un routeur | `router.middleware(auth);` |
| 404 personnalisée | `app.fallback(\|_\| async { Page::new("Introuvable").status(404) });` |
| Dossier statique | `public/` sur `/` automatiquement ; `app.static_dir("/files", "files");` |
| WebSocket | `app.ws("/chat", \|req, mut socket\| async move { ... });` |

Voir [Routes](routing.md).

## Requêtes

| Lire | Code |
|---|---|
| Méthode, chemin | `req.method()`, `req.path()` |
| Valeur de la query string | `req.query("q")` (`Option<Cow<str>>`) |
| Toute la query string | `let s: Search = req.query_as()?;` |
| En-tête, cookie | `req.header("user-agent")`, `req.cookie("session")` |
| Corps JSON | `let body: NewTodo = req.json().await?;` |
| Corps de formulaire | `let form: Login = req.form().await?;` |
| Texte, octets | `req.text().await?`, `req.bytes().await?` |
| IP du client (derrière des proxys) | `req.extensions().get::<paquets::securite::ClientIp>()` |
| Base de données | `req.db()` |
| État | `req.state::<Settings>()` |

Voir [Requêtes et réponses](requests-responses.md).

## Réponses

| Valeur renvoyée | Donne |
|---|---|
| `"texte"`, `String` | `200`, texte brut |
| `Json(valeur)`, `json!({...})` | `200`, JSON |
| `Page::new("Titre").body(...)` | `200`, une page HTML |
| `StatusCode::NO_CONTENT` | ce statut |
| `(201, Json(valeur))` | un statut et un corps |
| `Redirect::to("/")`, `Redirect::see_other("/")`, `Redirect::permanent("/")` | `302`, `303`, `301` |
| `Result<T>` | `T`, ou le statut de l'erreur |
| `Option<T>` | `T`, ou `404` |
| `res::status(202).header("x-a", "1").cookie(Cookie::new("k", "v")).json(...)` | une réponse sur mesure |
| `res::file("report.pdf").await` | un fichier |

## Pages et vues

| Tâche | Code |
|---|---|
| Page | `Page::new("Titre").description("…").body(vue)` |
| Langue | `.lang("fr")` |
| Balises SEO | `.seo(Seo::new().canonical("…").image("…"))` |
| CSS | `.style("h1{color:red}")`, `.stylesheet("/site.css")` |
| Balise supplémentaire dans le head | `.head(r#"<link rel="icon" href="/favicon.svg"/>"#)` |
| Sans thème | `.plain()` |
| Statut | `.status(404)` |
| Élément | `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`… |
| `<main>` dans `main.rs` | `html::main()` |
| Texte (échappé) | `.text("…")` |
| Attributs | `.class("card")`, `.id("x")`, `.href("/")`, `.attr("type", "email")` |
| Enfants | `.child(vue)`, `.children(iter.map(...))` |
| Condition | `.child(flag.then(\|\| p().text("…")))` |
| Îlot | `island("counter", "0", counter)` |

Classes du thème : `.hero`, `.card`, `.grid`, `.row`, `.stack`, `.center`, `.muted`, `.button`, `.ghost`. Variables du thème : `--accent`, `--bg`, `--fg`, `--muted`, `--line`, `--card`, `--radius`, `--font`. Voir [Pages](pages.md).

## Modèles

| Tâche | Code |
|---|---|
| Table + JSON | `#[model] struct Todo { id: i64, title: String, done: bool }` |
| Nom de la table | `#[model(table = "todos")]` (par défaut : `todo`) |
| JSON seulement | `#[data] struct Login { email: String, password: String }` |
| Unique, index | `#[memoire(unique)]`, `#[memoire(index)]` |
| Nullable | `bio: Option<String>` |
| Facultatif en JSON | `#[serde(default)] done: bool` |
| Créer seulement la table | `app.model::<Todo>();` |

## Requêtes à la base de données

| Tâche | Code |
|---|---|
| Tout, une ligne, peut-être une ligne | `Todo::all(db)`, `Todo::get(db, id)` (404), `Todo::find(db, id)` (`Option`) |
| Compter | `Todo::count(db)` |
| Insérer | `let mut t = Todo { id: 0, ... }; t.insert(db).await?;` (id réécrit) |
| Mettre à jour, supprimer | `t.update(db)`, `t.delete(db)`, `Todo::delete_by_key(db, id)` |
| Filtrer | `Todo::query().filter(Todo::DONE.eq(false)).all(db)` |
| Combiner | `a & b`, `a \| b`, `!a` |
| Comparer | `.eq`, `.ne`, `.gt`, `.ge`, `.lt`, `.le`, `.is_in([..])`, `.between(a, b)`, `.is_null()` |
| Texte | `.contains("x")`, `.icontains("x")`, `.starts_with("x")`, `.like("%x")` |
| Trier, limiter | `.order_by(Todo::ID.desc()).limit(20).offset(40)` |
| Paginer | `.paginate(db, page, 20)` → `items`, `total` |
| Premier, existe | `.first(db)`, `.exists(db)` |
| Mettre à jour en masse | `Todo::update_many().set(Todo::DONE, true).filter(...).execute(db)` |
| Supprimer en masse | `Todo::delete_many().filter(...).execute(db)` |
| Transaction | `let tx = db.begin().await?; ...(&tx)...; tx.commit().await?;` |
| SQL brut | `db.raw::<(i64, String)>("SELECT …", &paquets::orm::params![…])` |

`db` est `req.db()` dans un handler, ou l'argument de `on_start`. Voir [Base de données](database.md).

## Bases de données

| `DATABASE_URL` | Base de données |
|---|---|
| non définie, `memory` | en mémoire |
| `sqlite://app.db` | fichier SQLite |
| `postgres://…` | PostgreSQL (Heroku Postgres) |
| `mysql://…`, `mariadb://…` | MySQL, MariaDB (feature `mysql`) |
| `redis://…` | Redis (feature `redis`) |
| `mongodb://…` | MongoDB (feature `mongodb`) |

## API REST

`app.api::<Todo>("/api/todos");`

| Requête | Réponse |
|---|---|
| `GET /api/todos?limit=100&offset=0` | `200`, tableau, `X-Total-Count` |
| `GET /api/todos/:id` | `200` ou `404` |
| `POST /api/todos` | `201` avec l'`id` généré |
| `PUT /api/todos/:id` | `200`, toute la ligne remplacée |
| `PATCH /api/todos/:id` | `200`, les champs envoyés modifiés |
| `DELETE /api/todos/:id` | `204` ou `404` |

Voir [API REST](rest-api.md).

## Erreurs

| Tâche | Code |
|---|---|
| Type de retour | `Result<T>` (= `Result<T, paquets::Error>`) |
| Propager | `?` |
| Créer | `Error::bad_request(…)`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `unprocessable`, `internal`, `Error::new(418, "…")` |
| Dans une closure | `Ok::<_, Error>(valeur)` |
| À partir de `None` | `.ok_or_else(\|\| Error::not_found("…"))?` |
| Format sur mesure | `app.on_error(\|err\| res::status(err.status()).json(json!({ "message": err.message() })));` |

| `?` sur | Statut |
|---|---|
| paramètre, query string, JSON, formulaire invalide | `400` |
| ligne introuvable | `404` |
| valeur unique déjà prise | `409` |
| tout le reste | `500` (détails dans les journaux) |

Voir [Erreurs](errors.md).

## Sécurité

`use paquets::securite::prelude::*;`

| Tâche | Code |
|---|---|
| Limitation de débit | `paquets::shield().guard(RateLimit::per_minute(600)).build()` |
| Corps plus gros | `.guard(BodyLimit::new(10 << 20))` et `app.body_limit(10 << 20)` |
| Retirer un guard | `.without("bot_blocker")` |
| Front-end sur une autre origine | `.guard(Cors::new().allow_origin(o)).guard(Csrf::new().trusted_origin(o))` |
| Exemption CSRF | `.guard(Csrf::new().exempt("/auth/callback"))` |
| IP du client derrière un proxy | `.client_ip(ClientIpSource::heroku())`, `::fly()`, `::cloudflare()` |
| Authentification Basic sur une section | `.guard(Auth::basic("Admin").user("admin", "…").protect(["/admin"]))` |
| Journaliser les requêtes bloquées | `.on_event(\|e\| eprintln!("{} a bloqué {}", e.guard, e.path))` |
| Préréglages | `ShieldBuilder::strict()`, `ShieldBuilder::api()` |

Voir [Sécurité](security.md).

## Îlots

| Tâche | Code |
|---|---|
| Îlot (dans `src/lib.rs`) | `pub fn counter(props: &str) -> Node { ... }` |
| État | `let n = use_state(0);` `n.get()`, `n.set(v)`, `n.update(\|n\| *n += 1)`, `n.with(\|v\| …)` |
| Événements | `.on_click(\|e\| …)`, `.on_input(\|e\| …)`, `.on("keydown", \|e\| …)` ; `e.value()`, `e.key()` |
| Dynamique | `.text_dyn(\|\| …)`, `.attr_dyn(nom, \|\| …)`, `.class_if(classe, \|\| …)`, `.bool_attr(nom, \|\| …)` |
| Enregistrer | `#[wasm_bindgen(start)] pub fn start() { paquets::hydrate(&[("counter", counter)]); }` |
| Placer | `island("counter", "0", counter)` |
| Compiler | `paquets dev`, `paquets build` → `pkg/` |

Voir [Îlots](islands.md).

## Tests

| Tâche | Code |
|---|---|
| Client | `let client = app().test().await;` |
| Requêtes | `client.get("/")`, `.post(...)`, `.put`, `.patch`, `.delete`, puis `.await` |
| Corps, en-tête | `.json(&json!({...}))`, `.form(&[("k", "v")])`, `.header("k", "v")` |
| Réponse | `res.status()`, `res.text()`, `res.json::<T>()`, `res.header("k")` |
| Beaucoup de requêtes | `app.security(paquets::shield().without("rate_limit").build());` |

Voir [Tests](testing.md).

## Variables d'environnement

| Variable | Effet |
|---|---|
| `PORT` | le port (`3000` par défaut) |
| `DATABASE_URL` | la base de données (par défaut : en mémoire) |
| `DYNO` | définie par Heroku : l'IP du client est lue dans `X-Forwarded-For` |

## Déploiement

| Tâche | Où |
|---|---|
| Heroku en un geste | `https://heroku.com/deploy?template=https://github.com/<vous>/<dépôt>` |
| Déploiements automatiques sur Heroku | secret GitHub `HEROKU_API_KEY`, variable `HEROKU_APP_NAME` (`HEROKU_REGION` en option) |
| Base de données sur Heroku | Resources → Heroku Postgres |
| Redéployer à la main | Actions → Deploy to Heroku → Run workflow |
| Docker | `docker build -t mon-app . && docker run -p 8080:8080 mon-app` |
| Éviter la fusion automatique | nommer la branche `wip/...` |
| Publier sur crates.io | secret `CARGO_REGISTRY_TOKEN`, puis Releases → Draft a new release → `v0.1.0` |

Voir [Heroku](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Automatisation](automation.md) et [Publier sur crates.io](publish-crates-io.md).
