# Installation

Cette page installe tout ce qu'il faut pour créer des applications Paquets : Rust, la commande `paquets` et, seulement si vous utilisez des îlots, deux outils WebAssembly. Elle montre aussi comment ajouter Paquets à un projet existant sans passer par la commande.

## Prérequis

| Outil | Nécessaire pour | Version |
|---|---|---|
| Rust et Cargo | tout | 1.88 ou plus récente (Paquets utilise l'édition 2024) |
| Un compilateur C | la base de données SQLite, compilée dans votre programme | n'importe laquelle (`cc`, `gcc`, `clang` ou MSVC) |
| La cible `wasm32-unknown-unknown` | les îlots seulement | installée par `paquets dev` |
| `wasm-bindgen-cli` | les îlots seulement | exactement la version de la crate `wasm-bindgen` de votre `Cargo.lock`, installée par `paquets dev` |
| Git | créer un projet, déployer | n'importe laquelle |
| Docker | construire l'image de production sur votre ordinateur (facultatif) | n'importe laquelle |

Pas besoin de Node.js, de npm, d'OpenSSL ni d'un serveur de base de données.

## 1. Installer Rust

Si Rust n'est pas encore installé, utilisez [rustup](https://rustup.rs), l'installateur officiel :

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Sous Windows, téléchargez plutôt `rustup-init.exe` depuis https://rustup.rs et lancez-le. Il propose aussi d'installer les outils de compilation Microsoft C++, dont Rust a besoin sous Windows.

Ouvrez un nouveau terminal, puis vérifiez l'installation :

```sh
rustc --version   # rustc 1.88.0 ou plus récent
cargo --version
```

Si Rust était déjà installé, mettez-le à jour avec `rustup update`.

> [!TIP]
> Sous Linux, si une compilation échoue plus tard avec `linker 'cc' not found`, installez une chaîne de compilation C : `sudo apt install build-essential` sous Debian et Ubuntu, `sudo dnf install gcc` sous Fedora. Sous macOS, lancez `xcode-select --install`.

## 2. Installer la commande paquets

La commande `paquets` crée les projets, les lance et les compile pour la production. Installez-la avec Cargo :

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

`--locked` compile l'outil avec exactement les versions de dépendances avec lesquelles il a été testé.

La première installation prend quelques minutes, le temps que Cargo compile l'outil et ses dépendances. Vérifiez ensuite :

```sh
paquets --version
paquets doctor
```

`paquets doctor` liste les outils qu'il trouve et vous explique comment installer ceux qui manquent :

```text
📦 Paquets 0.1.1

  ✓ cargo 1.97.0 (c980f4866 2026-06-30)
  ✗ wasm32-unknown-unknown target (islands)
      → rustup target add wasm32-unknown-unknown (paquets dev does it for you)
  ✗ wasm-bindgen (islands)
      → cargo install wasm-bindgen-cli --locked (paquets dev does it for you)
  ✓ git version 2.43.0
  ✗ docker (optional)
      → only to build the production image locally

Ready. Create an app with `paquets new my-app`.
```

Seul `cargo` est indispensable. Les autres lignes sont informatives. Voir [Ligne de commande](cli.md#paquets-doctor) pour les détails.

## 3. Les outils pour les îlots (facultatif)

Les [îlots](islands.md) sont les parties d'une page qui s'exécutent dans le navigateur, compilées en WebAssembly. Ils demandent deux outils de plus. Inutile de les installer vous-même : le premier `paquets dev` dans un projet avec des îlots installe ce qui manque. Pour les installer à l'avance :

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

La version de `wasm-bindgen-cli` doit être **exactement** celle de la crate `wasm-bindgen` du `Cargo.lock` de votre projet (`0.2.129` au moment où ces lignes sont écrites). Lancé dans un projet, `paquets doctor` vous indique la version dont le projet a besoin.

Si vous ne créez que des pages, des API et des routes WebSocket, passez cette étape : un projet créé avec `paquets new --minimal` n'a pas d'îlots et n'a besoin d'aucun outil WebAssembly.

## 4. Créer votre premier projet

```sh
paquets new mon-app
cd mon-app
paquets dev
```

Ouvrez http://localhost:3000. Le [Démarrage rapide](quick-start.md) prend le relais à partir d'ici.

## Ajouter Paquets à un projet existant

Vous pouvez aussi utiliser Paquets sans la commande, comme n'importe quelle autre crate. Ajoutez-le à `Cargo.toml` :

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }

# Pour les tests (`#[tokio::test]`).
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Ajoutez Paquets à `Cargo.toml` :

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }
```

Remplacez ensuite `src/main.rs` par une première application :

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Accueil").body(h1().text("Bonjour 👋")) });
    app.run()
}
```

et lancez-la avec `cargo run`. C'est tout ce dont un projet a besoin : pas de `#[tokio::main]`, aucune autre dépendance. Pour ajouter des îlots à un tel projet, voir [Îlots](islands.md#préparer-un-projet-à-la-main).

## Features Cargo

Les features activent ou désactivent des parties facultatives de Paquets. Les valeurs par défaut conviennent à la plupart des applications :

| Feature | Par défaut | Ajoute |
|---|---|---|
| `sqlite` | ✓ | les bases SQLite (`DATABASE_URL=sqlite://app.db`), compilées dans le programme : rien à installer |
| `postgres` | ✓ | PostgreSQL avec TLS (Heroku Postgres, Neon, Supabase…) |
| `mysql` | | MySQL et MariaDB, avec TLS |
| `redis` | | Redis (lignes stockées sous forme de documents JSON), avec TLS via `rediss://` |
| `mongodb` | | MongoDB |
| `chrono` | | les colonnes de date et d'heure (`chrono::NaiveDateTime`, `DateTime<Utc>`…) |
| `uuid` | | les colonnes `uuid::Uuid` |
| `json` | | les colonnes JSON (`serde_json::Value`, `Json<T>`) |
| `encryption` | | les bases en mémoire gardées dans un fichier chiffré (voir [Base de données](database.md#une-base-en-mémoire-gardée-dans-un-fichier)) |
| `tls` | | HTTPS dans le serveur lui-même : `app.tls(...)` (voir [Configuration](configuration.md#https-dans-le-serveur)) |
| `http2` | | HTTP/2, négocié avec `tls`, et en clair (h2c) |
| `http3` | | HTTP/3 sur QUIC, à côté de HTTP/1.1 |
| `session` | | les cookies signés et les sessions côté serveur (voir [Sécurité](security.md#comptes-et-sessions)) |
| `templates` | | les templates HTML avec minijinja : `app.views("views")`, `req.render(...)` |
| `multipart` | | les envois de fichiers : `req.multipart()` (voir [Requêtes et réponses](requests-responses.md#envois-de-fichiers)) |
| `compress` | | les réponses en Brotli ou en gzip : `middleware::compress()` (voir [Performances](performance.md#compression)) |
| `tower` | | les couches tower comme middlewares : `middleware::layer(...)` |
| `full` | | tout ce qui précède |
| `client` | | le côté navigateur des îlots (uniquement pour la cible `wasm32-unknown-unknown`) |
| `browser-storage` | | dans les îlots, un store d'état et une base de données gardés dans `localStorage` (voir [Îlots](islands.md#garder-des-données-dans-le-navigateur)) |

Par exemple, pour utiliser MySQL et les dates :

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", features = ["mysql", "chrono"] }
```

Pour un programme plus petit et plus rapide à compiler quand vous n'avez besoin ni de SQLite ni de PostgreSQL, désactivez les valeurs par défaut et ne gardez que ce que vous utilisez. La base de données en mémoire est toujours disponible :

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["postgres"] }
```

## Éditeur

N'importe quel éditeur fait l'affaire. Pour la complétion, les erreurs en direct et « aller à la définition », installez [rust-analyzer](https://rust-analyzer.github.io), disponible pour VS Code, Zed, Neovim, Helix, Emacs et les IDE JetBrains. Paquets n'a ni langage de templates ni macros dans les vues : rust-analyzer comprend donc tout votre code, pages comprises.

## Dépannage

| Problème | Solution |
|---|---|
| `paquets: command not found` | Cargo installe les commandes dans `~/.cargo/bin`. Ouvrez un nouveau terminal, ou ajoutez ce dossier à votre `PATH`. |
| `linker 'cc' not found` | Installez une chaîne de compilation C (voir l'astuce de l'étape 1). |
| `error: package ... requires rustc 1.88` ou une erreur sur l'édition 2024 | Votre Rust est trop ancien : `rustup update`. |
| La première compilation prend plusieurs minutes | C'est normal : Cargo compile chaque dépendance une fois. Les compilations suivantes ne recompilent que votre code. |
| `wasm-bindgen` signale une différence de version ou de schéma | La CLI et la crate ne correspondent pas : `cargo install wasm-bindgen-cli --version <version du Cargo.lock> --locked`. `paquets doctor` affiche les deux versions. |

## Étapes suivantes

- [Démarrage rapide](quick-start.md) : construire une petite application pas à pas.
- [Ligne de commande](cli.md) : toutes les commandes `paquets`.
- [Structure d'un projet](project-structure.md) : ce que crée `paquets new`.
