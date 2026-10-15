# Ligne de commande

La commande `paquets` crée les projets, les lance pendant le développement, les compile pour la production et vérifie vos outils. C'est une fine couche au-dessus de Cargo : tout ce qu'elle fait, vous pouvez aussi le faire à la main, et cette page montre comment.

## Installation

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

Installez-la avec cargo :

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

Voir [Installation](installation.md) pour Rust lui-même et les outils facultatifs.

## Les commandes

| Commande | Rôle |
|---|---|
| `paquets new <nom>` | crée un projet complet : serveur, pages, un modèle et son API, un îlot, des tests, `Dockerfile`, fichiers Heroku, workflows GitHub |
| `paquets new <nom> --minimal` | la même chose, avec un seul `src/main.rs` et sans îlots |
| `paquets dev` | compile les îlots (debug), puis lance l'application avec `cargo run` |
| `paquets dev --watch` | la même chose, puis recompile et relance à chaque changement ; les pages ouvertes se rechargent toutes seules |
| `paquets routes` | liste les routes de l'application (méthode, chemin, description) |
| `paquets db:migrate` | exécute les migrations en attente (`app.migrate`) |
| `paquets db:seed` | remplit la base de données (`app.seed`) |
| `paquets make:*` | générateurs : `make:model`, `make:api`, `make:page`, `make:migration`, `make:test`, `make:auth`, `make:admin` (voir [Générateurs](generators.md)) |
| `paquets build` | compilation release : les îlots dans `pkg/`, puis le serveur avec `cargo build --release` |
| `paquets doctor` | vérifie Cargo, la cible `wasm32-unknown-unknown`, `wasm-bindgen-cli` et sa version, Git et Docker |
| `paquets help` | la liste des commandes |
| `paquets --version` | la version de l'outil |

```text
$ paquets help
📦 Paquets — the simplest full-stack framework for Rust

Usage:
  paquets new <name> [--minimal]   create a project (server, pages, API, islands,
                                   Dockerfile, Heroku, CI with auto-merge into master)
  paquets dev [-- <args>]          build the islands, then run the app (cargo run)
  paquets build                    release build: islands in pkg/, then the server
  paquets doctor                   check cargo, the wasm32 target and wasm-bindgen
  paquets help                     this help
  paquets --version                the version

Options of `new`:
  --minimal       one src/main.rs, no islands (no WebAssembly tools needed)
  --path <dir>    use a local checkout of Paquets instead of the published one

Documentation: https://paquets-90dfc06b6fea.herokuapp.com/
```

## paquets new

```sh
paquets new mon-app
```

```text
📦 mon-app is ready (12 files, git repository on master).

  cd mon-app
  paquets dev        # http://localhost:3000

The first `paquets dev` also prepares the WebAssembly tools for the islands.
Deploy from your phone: push the project to GitHub, then see README.md.
```

Elle crée un dossier qui porte le nom du projet, avec tous les fichiers décrits dans [Structure d'un projet](project-structure.md) : `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `public/favicon.svg`, `Dockerfile`, `heroku.yml`, `app.json`, `.github/workflows/ci.yml`, `.github/workflows/heroku.yml`, `README.md`, `.gitignore` et `.dockerignore`. Elle lance ensuite `git init` avec une branche `master` (la branche dans laquelle la CI fusionne), sauf si Git est absent ou si le dossier se trouve déjà dans un dépôt Git.

| Option | Effet |
|---|---|
| `--minimal` | un seul `src/main.rs`, pas de `src/lib.rs`, pas d'îlots : aucun outil WebAssembly nécessaire (11 fichiers) |
| `--path <dossier>` | le projet utilise une copie locale de Paquets (`paquets = { path = "<dossier>" }`) au lieu de la version publiée : pour travailler sur Paquets lui-même |

Le nom doit être un nom de paquet valide : lettres minuscules, chiffres, `-` et `_`, en commençant par une lettre (`mon-app`, `blog2`, `todo_list`). Certains noms sont réservés (`paquets`, `test`, `std`, `core`…). Le dossier ne doit pas encore exister, ou doit être vide :

```text
error: `Mon-App` is not a valid name: use lowercase letters, digits, `-` and `_`, starting with a letter (e.g. my-app)
error: `mon-app` already exists and is not empty
```

Le titre des pages de la nouvelle application vient du nom : `mon-app` devient « Mon App ».

## paquets dev

```sh
paquets dev
```

Lancez-la dans le dossier du projet. Elle :

1. compile les îlots, si le projet en a (un `src/lib.rs` et `crate-type = ["cdylib", ...]` dans `Cargo.toml`) :
   - installe la cible `wasm32-unknown-unknown` si elle manque (une seule fois) ;
   - compile `src/lib.rs` en WebAssembly, en mode debug (rapide à compiler) ;
   - installe `wasm-bindgen-cli` à la version de `wasm-bindgen` du `Cargo.lock` s'il manque ou si sa version diffère (une seule fois, quelques minutes) ;
   - écrit le bundle dans `pkg/` avec `wasm-bindgen --target web` ;
2. lance `cargo run`.

Si les îlots ne peuvent pas être compilés, elle affiche un avertissement et lance quand même l'application : les pages fonctionnent, les îlots restent simplement statiques jusqu'à ce que le problème soit réglé (`paquets doctor` vous aide).

Les arguments placés après `--` vont à votre programme : `paquets dev -- --une-option`. Les variables d'environnement fonctionnent comme d'habitude : `DATABASE_URL=sqlite://app.db paquets dev`.

`paquets dev` ne surveille pas vos fichiers : après une modification, arrêtez le serveur (`Ctrl+C`) et relancez-le. `paquets dev --watch` le fait pour vous : il recompile et relance l'application à chaque changement de `src/`, `public/` ou `Cargo.toml`, et les pages ouvertes dans le navigateur se rechargent toutes seules (voir [Exploitation](operations.md#rechargement-automatique)).

## paquets build

```sh
paquets build
```

```text
🏝️  Islands
  → cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
  → wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/wasm-release/mon_app.wasm
  → cargo build --release
📦 Built: target/release/mon-app
   Islands: pkg/ (deploy it next to the program, with public/)
```

Elle compile les îlots en mode release, avec le profil `wasm-release` du `Cargo.toml` quand il existe (optimisé pour la taille), puis le serveur avec `cargo build --release`. Le programme se trouve dans `target/release/`. Pour le lancer, exécutez-le depuis un dossier qui contient `public/` et `pkg/` :

```sh
./target/release/mon-app
```

`CARGO_TARGET_DIR` est respectée, si vous la définissez. Le `Dockerfile` d'un projet exécute les mêmes étapes (voir [Docker](deploy-docker.md)).

## paquets doctor

```sh
paquets doctor
```

```text
📦 Paquets 0.1.1

  ✓ cargo 1.97.0 (c980f4866 2026-06-30)
  ✓ wasm32-unknown-unknown target
  ✓ wasm-bindgen 0.2.129 (matches Cargo.lock)
  ✓ git version 2.43.0
  ✗ docker (optional)
      → only to build the production image locally

Ready. Create an app with `paquets new my-app`.
```

| Vérification | Nécessaire pour | Quand il manque |
|---|---|---|
| `cargo` | tout | le seul outil indispensable : installez Rust depuis https://rustup.rs |
| la cible `wasm32-unknown-unknown` | les îlots | `rustup target add wasm32-unknown-unknown`, ou laissez `paquets dev` s'en charger |
| `wasm-bindgen` | les îlots | `cargo install wasm-bindgen-cli --version <version> --locked`, ou laissez `paquets dev` s'en charger |
| `git` | créer des projets, déployer | https://git-scm.com |
| `docker` | construire l'image de production sur votre ordinateur | facultatif |

Lancée dans un projet, elle compare aussi le `wasm-bindgen` installé à la version du `Cargo.lock`, et vous donne la commande exacte quand elles diffèrent :

```text
  ✗ wasm-bindgen 0.2.100, Cargo.lock wants 0.2.129
      → cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

## Générateurs et commandes de l'application

`paquets make:model`, `make:api`, `make:page`, `make:migration`, `make:test`, `make:auth` et `make:admin` écrivent du code et le branchent dans `src/main.rs` : voir [Générateurs](generators.md).

`paquets routes`, `paquets db:migrate` et `paquets db:seed` lancent votre application (`cargo run`) avec la variable d'environnement `PAQUETS_COMMAND` : au lieu de servir, elle affiche ses routes, exécute ses migrations ou ses données d'exemple, puis s'arrête. Voir [Exploitation](operations.md).

## Sans la ligne de commande

Chaque commande est un raccourci vers des outils standard :

| Commande | À la main |
|---|---|
| `paquets dev` | `cargo build --lib --target wasm32-unknown-unknown` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/debug/mon_app.wasm` + `cargo run` |
| `paquets build` | `cargo build --lib --target wasm32-unknown-unknown --profile wasm-release` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/wasm-release/mon_app.wasm` + `cargo build --release` |
| `paquets dev` dans un projet minimal | `cargo run` |

Le fichier `.wasm` porte le nom du paquet, avec `-` remplacé par `_`. En CI, ces commandes évitent d'installer l'outil `paquets`.

## Étapes suivantes

- [Démarrage rapide](quick-start.md) : votre premier projet, pas à pas.
- [Structure d'un projet](project-structure.md) : chaque fichier que crée `paquets new`.
- [Îlots](islands.md) : ce que compile `paquets dev`.
