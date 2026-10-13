<div align="center">

[![Paquets : le framework full-stack le plus simple pour Rust](https://raw.githubusercontent.com/maxlestage/Paquets/master/site/public/assets/og-image.png)](https://paquets-90dfc06b6fea.herokuapp.com/)

# 📦 Paquets

**Le framework full-stack le plus simple pour Rust.**

Serveur, sécurité, base de données, API REST, pages optimisées pour le SEO et îlots WebAssembly : une crate, un import, une `App`.

[![crates.io](https://img.shields.io/crates/v/paquets.svg)](https://crates.io/crates/paquets)
[![docs.rs](https://img.shields.io/docsrs/paquets)](https://docs.rs/paquets)
[![Licence : propriétaire](https://img.shields.io/badge/license-proprietary-2a2a2e.svg)](#licence)

[English](README.md) · **Français** · [Español](README.es.md)

[Site et documentation](https://paquets-90dfc06b6fea.herokuapp.com/) · [Déployer sur Heroku depuis un téléphone](#déployer-depuis-un-téléphone)

</div>

---

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
        Page::new("Accueil").body(h1().text("Bonjour 👋"))
    });
    app.api::<Todo>("/api/todos"); // GET, POST, PUT, PATCH, DELETE

    app.run() // http://localhost:3000, ou $PORT sur Heroku
}
```

C'est une application web complète : une page rendue par le serveur avec ses balises SEO, une
API REST enregistrée dans une base de données, et un bouclier de sécurité devant chaque requête.
Rien à configurer.

## Pourquoi Paquets

| | |
|---|---|
| ✨ **Simple** | `use paquets::prelude::*;`, `App::new()`, `app.run()`. Les handlers sont des fonctions `async` ; `?` fonctionne sur toutes les erreurs et choisit le bon statut HTTP. |
| 🛡️ **Sécurisé par défaut** | WAF (injection SQL, XSS, RCE, Log4Shell…), limitation de débit, CSRF, en-têtes de sécurité, identifiants de requête, blocage des robots d'attaque. Les pages envoient une Content-Security-Policy qui n'autorise que leur propre code. |
| 🗄️ **Toutes les bases** | En mémoire, SQLite, PostgreSQL, MySQL/MariaDB, Redis, MongoDB, choisies par `DATABASE_URL`. `#[model]` donne des requêtes typées et une API REST en une ligne. |
| 🔎 **Pages SEO** | Les pages sont du HTML fini, rendu par le serveur : titre, description, Open Graph, Twitter, URL canonique, JSON-LD. Un thème intégré (clair et sombre) les rend belles sans CSS. |
| 📲 **Installable (PWA)** | `app.pwa(Pwa::new("Mon app"))` : un manifeste d'application web et un service worker hors ligne, pour que l'application aille sur l'écran d'accueil et fonctionne sans réseau. |
| 🏝️ **Îlots** | Les parties interactives sont des composants Rust compilés en WebAssembly, et seulement elles : le reste de la page n'envoie aucun code. |
| ⚡ **Rapide** | Construit sur un serveur plus rapide qu'actix-web et axum, une chaîne de sécurité d'environ 6 µs, du SQL généré à la compilation. |
| 📱 **Déployé depuis un téléphone** | Dockerfile, `heroku.yml`, `app.json` et GitHub Actions sont générés : un appui sur *Deploy to Heroku*. |

## Quatre paquets, un framework

Paquets assemble quatre crates, chacune utilisable seule :

| Paquet | Rôle | Dans Paquets |
|---|---|---|
| [**Vitesse**](https://vitesse-f16135654c3d.herokuapp.com/) | Serveur HTTP dans le style d'Express : routes, middlewares, routeurs, WebSocket, sessions, HTTPS, HTTP/2 et HTTP/3 | `App` (se déréférence en `vitesse::App`), `Request`, `Json`, `Router`… |
| [**Securite**](https://securite-2db4c1994be7.herokuapp.com/) | Middlewares de sécurité : WAF, limitation de débit, CSRF, CORS, en-têtes et CSP, authentification… | devant chaque requête, `app.security(...)` |
| [**memoire-orm**](https://memoire-7f4525f1fdaf.herokuapp.com/) | ORM asynchrone typé, SQL généré à la compilation, SQL et NoSQL, relations et recherche plein texte | `#[model]`, `req.db()`, `app.api::<M>()` |
| [**active**](https://active-c1883a966650.herokuapp.com/) | Front-end : composants, état, rendu serveur, SEO, îlots WebAssembly | `Page`, `div()`, `h1()`, `island(...)`, `use_state` |

## Démarrer

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked    # la commande `paquets`
paquets new mon-app               # un projet complet, prêt à déployer
cd mon-app
paquets dev                       # http://localhost:3000
```

`paquets new` crée le serveur (`src/main.rs` : des pages, un modèle et son API), les composants du
navigateur (`src/lib.rs` : un îlot), un `Dockerfile`, les fichiers Heroku, et des GitHub Actions
qui testent chaque push et **fusionnent automatiquement chaque branche verte dans `master`**.
`paquets build` fait la compilation de production, `paquets doctor` vérifie vos outils.

Ou ajoutez Paquets à un projet existant :

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }
```

## Visite guidée

```rust
use paquets::prelude::*;

#[model]                         // une table, et du JSON en entrée comme en sortie
struct Todo {
    id: i64,                     // clé primaire, générée par la base
    title: String,
    done: bool,
}

/// Une page, rendue par le serveur.
async fn accueil(req: Request) -> Result<Page> {
    let todos = Todo::query()
        .filter(Todo::DONE.eq(false))
        .order_by(Todo::ID.asc())
        .all(req.db())
        .await?;
    Ok(Page::new("À faire")
        .lang("fr")
        .description("Ce qu'il reste à faire.")
        .body(h1().text("À faire"))
        .body(ul().children(todos.into_iter().map(|t| li().text(t.title)))))
}

/// Du JSON, avec des erreurs qui deviennent le bon statut.
async fn detail(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si `id` n'est pas un nombre
    let todo = Todo::get(req.db(), id).await?; // 404 si la ligne n'existe pas
    Ok(Json(todo))
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", accueil);
    app.get("/todos/:id", detail);
    app.api::<Todo>("/api/todos");
    app
}

#[tokio::test]
async fn ca_marche() {
    let client = app().test().await;            // sans port, base en mémoire
    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Essayer Paquets", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    assert!(client.get("/").await.text().contains("Essayer Paquets"));
}
```

## Documentation

Tout est expliqué sur le [site](https://paquets-90dfc06b6fea.herokuapp.com/fr/docs/) et dans
[`docs/`](docs/fr/README.md), en français, en [anglais](docs/en/README.md) et en [espagnol](docs/es/README.md) :

| Démarrer | L'essentiel | Aller plus loin | Déployer | Référence |
|---|---|---|---|---|
| [Introduction](docs/fr/introduction.md) | [Routes](docs/fr/routing.md) | [Îlots](docs/fr/islands.md) | [Heroku depuis un téléphone](docs/fr/deploy-heroku-mobile.md) | [Ligne de commande](docs/fr/cli.md) |
| [Installation](docs/fr/installation.md) | [Requêtes et réponses](docs/fr/requests-responses.md) | [WebSocket](docs/fr/websocket.md) | [Docker](docs/fr/deploy-docker.md) | [Aide-mémoire](docs/fr/cheatsheet.md) |
| [Démarrage rapide](docs/fr/quick-start.md) | [Pages](docs/fr/pages.md) | [Tests](docs/fr/testing.md) | [Publier sur crates.io](docs/fr/publish-crates-io.md) | [Les quatre paquets](docs/fr/packages.md) |
| [Structure d'un projet](docs/fr/project-structure.md) | [Base de données](docs/fr/database.md) | [Erreurs](docs/fr/errors.md) | [Automatisation](docs/fr/automation.md) | [FAQ](docs/fr/faq.md) |
| | [API REST](docs/fr/rest-api.md) · [Sécurité](docs/fr/security.md) | [Configuration](docs/fr/configuration.md) · [Performances](docs/fr/performance.md) | | |

Référence de l'API : [docs.rs/paquets](https://docs.rs/paquets).

## Déployer depuis un téléphone

1. Ouvrez `https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets` sur votre téléphone,
   choisissez un nom d'application, appuyez sur **Deploy app**. Heroku lit [`app.json`](app.json)
   et construit, avec le [buildpack emk/rust](https://elements.heroku.com/buildpacks/emk/heroku-buildpack-rust)
   sur la pile heroku-26 ([`RustConfig`](RustConfig), [`heroku-build.sh`](heroku-build.sh),
   [`Procfile`](Procfile)), le site, qui est lui-même une application Paquets, avec sa démo en
   direct (`/demo`, `/api/todos`).
2. Déploiements automatiques de `master` : dans GitHub, *Settings → Secrets and variables →
   Actions*, ajoutez le secret `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key*) et la
   variable `HEROKU_APP_NAME`. [`heroku.yml`](.github/workflows/heroku.yml) déploie alors chaque fusion.
   Avec la variable facultative `HEROKU_BUILD` à `docker`, il construit le [`Dockerfile`](Dockerfile)
   à la place ([`heroku.yml`](heroku.yml), pile « container »).
3. Une base de données : Heroku → votre application → *Resources* → *Heroku Postgres*. Sa
   `DATABASE_URL` est utilisée automatiquement.

Sans GitHub Actions : `heroku create --stack heroku-26 --buildpack emk/rust` (sur une application
existante, `heroku stack:set heroku-26` puis `heroku buildpacks:set emk/rust`), puis
`git push heroku master` ; ou, dans le tableau de bord, *Settings → Buildpacks → Add buildpack* →
`emk/rust` (la pile ne doit pas être « container »).

Heroku n'a pas d'offre gratuite (un dyno Eco coûte environ 5 $ par mois). Les projets créés avec
`paquets new` se déploient de la même façon, construits à partir de leur `Dockerfile`. Guide pas à pas :
[docs/fr/deploy-heroku-mobile.md](docs/fr/deploy-heroku-mobile.md).

Le site est aussi publié gratuitement sur **GitHub Pages** à l'adresse
<https://maxlestage.github.io/Paquets/> ([`pages.yml`](.github/workflows/pages.yml) ; à faire une
fois : *Settings → Pages → Source : GitHub Actions*).

## Toujours fusionné dans master

Chaque push lance la [CI](.github/workflows/ci.yml) : formatage, clippy (serveur, navigateur et
toutes les features), tests, documentation, version minimale de Rust, projets générés par
`paquets new`, site. Quand tout est vert, la branche est **fusionnée automatiquement dans
`master`** (sauf les branches nommées `wip/...`), puis le site et l'application Heroku sont déployés.

## Publier sur crates.io (depuis un téléphone)

1. crates.io → *Account Settings* → *API Tokens* → un jeton avec les droits *publish-new* et
   *publish-update* ; ajoutez-le dans GitHub comme secret `CARGO_REGISTRY_TOKEN`.
2. Changez `version` dans `Cargo.toml` et `macros/Cargo.toml`, écrivez sa section dans `CHANGELOG.md`,
   poussez : une fois la CI verte, la version est taguée (`v0.1.1`) et publiée toute seule, rien à remplir.

[`release.yml`](.github/workflows/release.yml) publie `paquets-macros`, puis `paquets`. crates.io
n'accepte que des dépendances publiées sur crates.io : Vitesse, Securite, memoire-orm et Active sont
donc publiés d'abord, chacun depuis son dépôt, de la même façon ; le workflow le vérifie. Guide :
[docs/fr/publish-crates-io.md](docs/fr/publish-crates-io.md).

## Développement

```sh
cargo test --workspace                     # la crate, la commande, les exemples
cargo run --example todo                   # l'application de démo sur http://localhost:3000
cargo run -- new mon-app --path .          # la commande, avec ce dépôt de Paquets
cd site && cargo run                       # le site, lui-même une application Paquets (voir site/README.md)
```

| Dossier | Contenu |
|---|---|
| [`src/`](src) | la crate `paquets` (`app.rs`, `db.rs`, `api.rs`, `page.rs`, `shield.rs`, `error.rs`) et la commande (`main.rs`, `cli/`) |
| [`macros/`](macros) | `#[model]` et `#[data]` (`paquets-macros`) |
| [`examples/`](examples) | `hello` et `todo` (l'application déployée sur Heroku) |
| [`docs/`](docs) | la documentation, en anglais, français et espagnol |
| [`site/`](site) | le site : une application Paquets (pages rendues par Active, îlots en WebAssembly), exportée en fichiers statiques pour GitHub Pages |

## Licence

Paquets est un logiciel propriétaire : © 2026 Maxime Nathan Lestage, tous
droits réservés. Ce n'est pas un logiciel open source. Le code source est
publié sur crates.io et GitHub pour pouvoir être lu, audité et compilé, et
vous pouvez l'utiliser librement comme dépendance de vos propres programmes,
y compris commerciaux. Le modifier ou redistribuer son code source demande
l'accord écrit de l'auteur. Conditions complètes : [LICENSE](LICENSE).

Les contributions (code, documentation, idées) sont les bienvenues et
rejoignent le projet aux mêmes conditions : en soumettre une, c'est en céder
les droits à l'auteur, comme l'indique la licence.

## Crédits

Conçu et développé par Maxime Nathan Lestage,
au sein d'une même famille de projets Rust : [Active](https://active-c1883a966650.herokuapp.com/), [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/), [Securite](https://securite-2db4c1994be7.herokuapp.com/), [Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/) et [Paquets](https://paquets-90dfc06b6fea.herokuapp.com/).
