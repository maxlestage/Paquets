# Structure d'un projet

`paquets new mon-app` crée un projet complet : un serveur avec des pages, un modèle et son API REST, un îlot, des tests, un `Dockerfile` de production, les fichiers pour Heroku et deux workflows GitHub. Cette page passe chaque fichier en revue, explique où le programme cherche ses fichiers à l'exécution, et propose une façon d'organiser le code à mesure qu'il grandit.

## Les fichiers

```text
mon-app/
├── Cargo.toml                  le paquet : dépendances, deux cibles, profils de compilation
├── src/
│   ├── main.rs                 le serveur : pages, un modèle, son API REST, tests
│   └── lib.rs                  les composants partagés avec le navigateur, et les îlots
├── public/
│   └── favicon.svg             les fichiers statiques, servis à la racine du site
├── Dockerfile                  l'image de production (îlots + serveur)
├── heroku.yml                  indique à Heroku de construire le Dockerfile
├── app.json                    décrit l'application pour le bouton « Deploy to Heroku »
├── .github/
│   └── workflows/
│       ├── ci.yml              teste chaque push, puis fusionne la branche dans master
│       └── heroku.yml          déploie master sur Heroku
├── README.md                   comment lancer, tester et déployer le projet
├── .gitignore                  ignore target/, pkg/ et les fichiers *.db
└── .dockerignore               tient target/, pkg/, .git et *.db à l'écart des builds Docker
```

`paquets new` lance aussi `git init` avec une branche `master`, sauf si le dossier se trouve déjà dans un dépôt Git.

Les éléments suivants apparaissent plus tard et ne sont jamais commités (ils sont dans `.gitignore`) :

| Dossier ou fichier | Créé par | Contenu |
|---|---|---|
| `target/` | Cargo | tout ce que Cargo compile |
| `pkg/` | `paquets dev`, `paquets build` | le bundle navigateur des îlots : `mon_app.js` et `mon_app_bg.wasm` |
| `*.db` | SQLite | votre base de données locale, quand `DATABASE_URL=sqlite://...` |

## src/main.rs : le serveur

Au départ, le serveur tient dans un seul fichier. Il contient, de haut en bas :

1. `use paquets::prelude::*;`, qui importe tout ce dont une application a besoin, puis `use mon_app::counter;`, l'îlot défini dans `src/lib.rs` ;
2. un modèle `Todo` (`#[model]`) : une table de la base de données qui voyage aussi en JSON ;
3. une struct `NewTodo` (`#[data]`) : les données du formulaire « nouvelle tâche » ;
4. `home`, le handler de la page d'accueil : il lit les tâches dans la base de données et renvoie une `Page` avec un en-tête, une section hero, une carte contenant l'îlot compteur, la liste des tâches et un formulaire ;
5. `add`, le handler du formulaire : il enregistre la tâche et redirige vers `/` ;
6. `app()`, qui crée l'`App` et déclare les routes : `GET /`, `POST /todos` et l'API REST sur `/api/todos` ;
7. `main()`, qui lance `app()` ;
8. un module `tests` qui vérifie la page d'accueil et l'API.

C'est la séparation entre `app()` et `main()` qui rend l'application testable : les tests appellent `app().test().await` et obtiennent exactement les mêmes routes, sans ouvrir de port. Voir [Tests](testing.md).

## src/lib.rs : partagé avec le navigateur

`src/lib.rs` est compilé deux fois :

- **pour le serveur**, comme une bibliothèque ordinaire qu'utilise `src/main.rs` (`use mon_app::counter;`), pour que le serveur puisse rendre les îlots en HTML ;
- **pour le navigateur**, en WebAssembly, par `paquets dev` et `paquets build`, pour que le navigateur puisse rendre ces îlots interactifs.

Il contient l'îlot `counter` et le point d'entrée du navigateur :

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[cfg(target_arch = "wasm32")]` signifie « seulement dans la compilation pour le navigateur ». Mettez dans ce fichier (ou dans des modules de la bibliothèque) les composants et les îlots dont le navigateur a besoin, et gardez le code propre au serveur (handlers, requêtes à la base de données) dans `src/main.rs` et ses modules. Voir [Îlots](islands.md).

## Cargo.toml : une crate, deux cibles

```toml
[package]
name = "mon-app"
version = "0.1.0"
edition = "2024"
publish = false

# Une crate, deux cibles : le serveur (src/main.rs) et, compilés en WebAssembly par
# `paquets build`, les îlots de src/lib.rs.
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = "0.1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { version = "0.1", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true

# Le bundle du navigateur : le plus petit possible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

| Section | Pourquoi |
|---|---|
| `crate-type = ["cdylib", "rlib"]` | `cdylib` produit le fichier `.wasm` pour le navigateur, `rlib` permet à `src/main.rs` d'utiliser la bibliothèque. |
| Dépendances `cfg(not(target_arch = "wasm32"))` | Le côté serveur : Paquets avec ses features par défaut (SQLite, PostgreSQL). |
| Dépendances `cfg(target_arch = "wasm32")` | Le côté navigateur : Paquets sans le serveur ni les bases de données, avec la feature `client`, plus `wasm-bindgen`. |
| `[dev-dependencies]` | tokio, pour `#[tokio::test]` dans les tests. Votre application elle-même n'en a pas besoin. |
| `[profile.release]` | Les optimisations du serveur de production : optimisation à l'édition de liens, binaire plus petit. |
| `[profile.wasm-release]` | Utilisé par `paquets build` pour les îlots : optimisé pour la taille. |

La ligne `paquets` a à la fois une `version` et une source `git`. Pendant le développement, Cargo utilise le dépôt GitHub. Une fois Paquets sur crates.io, vous pourrez la simplifier en `paquets = "0.1"` (en gardant `default-features = false, features = ["client"]` sur la ligne wasm32).

## public/ : les fichiers statiques

Chaque fichier de `public/` est servi à la racine du site : `public/favicon.svg` devient `/favicon.svg`, `public/css/site.css` devient `/css/site.css`. Mettez-y les images, les polices, les feuilles de style, `robots.txt` et compagnie. Les fichiers partent avec les bons en-têtes `Content-Type`, `ETag` et `Last-Modified`, et les navigateurs reçoivent `304 Not Modified` quand ils ont déjà le fichier.

> [!NOTE]
> Un fichier de `public/` l'emporte sur une route qui a le même chemin, car les fichiers statiques sont vérifiés avant le routage. En particulier, un fichier `public/index.html` remplacerait votre route `GET /`.

## Les fichiers de déploiement

| Fichier | Rôle | Pour en savoir plus |
|---|---|---|
| `Dockerfile` | Compile les îlots et le serveur en mode release, puis copie le programme, `pkg/` et `public/` dans une petite image Debian qui tourne avec un utilisateur non root. | [Docker](deploy-docker.md) |
| `heroku.yml` | Indique à Heroku de construire l'image à partir du `Dockerfile` (la pile « container » de Heroku). | [Heroku](deploy-heroku-mobile.md) |
| `app.json` | Décrit l'application pour le bouton « Deploy to Heroku » : nom, description, pile container. | [Heroku](deploy-heroku-mobile.md#1-le-plus-rapide--le-bouton-deploy-to-heroku) |
| `.github/workflows/ci.yml` | À chaque push : formatage, Clippy (serveur et WebAssembly), tests. Quand tout est vert, la branche est fusionnée automatiquement dans `master` (sauf les branches nommées `wip/...`). | [Automatisation](automation.md) |
| `.github/workflows/heroku.yml` | Déploie `master` sur Heroku via l'API Platform de Heroku, configuré depuis votre téléphone avec un secret et une variable. | [Heroku](deploy-heroku-mobile.md#4-déploiements-automatiques-avec-github-actions) |

## Le projet minimal

`paquets new mon-app --minimal` crée le même projet, sans îlots :

- pas de `src/lib.rs` : toute l'application est dans `src/main.rs` ;
- un `Cargo.toml` tout simple, avec `paquets` sous `[dependencies]` et sans profil WebAssembly ;
- un `Dockerfile` qui ne compile que le serveur.

Il n'a besoin d'aucun outil WebAssembly. Vous pourrez ajouter des îlots plus tard : voir [Îlots](islands.md#préparer-un-projet-à-la-main).

## Où le programme cherche ses fichiers

Au démarrage, le serveur cherche deux dossiers **dans le répertoire de travail courant** (le dossier depuis lequel vous le lancez, c'est-à-dire la racine du projet avec `paquets dev` et `cargo run`) :

| Dossier | S'il existe |
|---|---|
| `public/` | servi à la racine du site |
| `pkg/` | servi sous `/pkg/` ; s'il contient un bundle (`<nom>.js` et `<nom>_bg.wasm`), les pages avec des îlots le chargent |

Le fichier SQLite de `DATABASE_URL=sqlite://app.db` est lui aussi relatif au répertoire de travail. Dans l'image Docker, le répertoire de travail est `/app`, où le `Dockerfile` copie `public/` et `pkg/`. Si vous lancez le binaire release à la main, lancez-le depuis la racine du projet (ou copiez `public/` et `pkg/` là où vous le lancez).

## Faire grandir le projet

Un seul fichier, c'est parfait pour commencer. Quand `src/main.rs` s'allonge, découpez-le en modules. Une organisation courante :

```text
src/
├── main.rs        les déclarations mod, fn app() avec les routes, fn main()
├── models.rs      les structs #[model] et #[data]
├── pages.rs       les handlers qui renvoient des pages, et la mise en page commune
├── api.rs         les routes JSON sur mesure, ou un Router monté sur /api
└── lib.rs         les îlots (navigateur)
```

Dans `src/main.rs`, déclarez les modules et gardez les routes au même endroit :

```rust
mod models;
mod pages;

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/", pages::home);
    app.api::<models::Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Chaque module commence par `use paquets::prelude::*;`. Les éléments utilisés par d'autres modules doivent être `pub` :

```rust
// src/models.rs
use paquets::prelude::*;

#[model]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}
```

```rust
// src/pages.rs
use paquets::prelude::*;

use crate::models::Todo;

/// La mise en page commune à toutes les pages.
pub fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .body(header().child(a().href("/").text("📦 Mon appli")))
        .body(main().child(content))
        .body(footer().text("Fait avec Paquets"))
}

pub async fn home(req: Request) -> Result<Page> {
    let todos = Todo::all(req.db()).await?;
    Ok(layout(
        "Accueil",
        ul().children(todos.into_iter().map(|todo| li().text(todo.title))),
    ))
}
```

Dans `pages.rs`, il n'y a pas de `fn main` : la balise `<main>` s'écrit donc simplement `main()`.

## Étapes suivantes

- [Routes](routing.md) : routes, paramètres, routeurs et middlewares.
- [Pages](pages.md) : vues, mises en page, thème et SEO.
- [Ligne de commande](cli.md) : ce que font `paquets dev` et `paquets build`.
