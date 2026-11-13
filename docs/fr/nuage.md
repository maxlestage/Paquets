# Nuage DB

Nuage DB est la base de données des projets Paquets : `paquets new` l'utilise depuis la version 0.3.3. Elle garde vos données dans un dossier du projet pendant que vous développez, et sur un serveur Nuage DB en production, avec le même code. Cette page explique son fonctionnement dans Paquets, comment lancer et déployer un serveur, connecter une application, utiliser les branches pour les aperçus et les tests, la console et le tableau de bord.

## Pourquoi Nuage DB

[Nuage DB](https://github.com/maxlestage/Nuage) est la base de données de Memoire, l'ORM de Paquets, par le même auteur :

- **Écrite entièrement en Rust**, comme le reste de votre application : un seul langage du navigateur jusqu'au disque.
- **Elle s'entretient seule** : un journal d'écriture anticipée (write-ahead log) synchronisé sur le disque avant chaque validation, la reprise après un crash au démarrage, des points de contrôle et un compactage automatiques, des sommes de contrôle sur le journal et sur les pages, et des sauvegardes automatiques dans un dossier ou sur un stockage compatible S3, restaurées d'elles-mêmes quand le serveur démarre avec un disque vide.
- **Sur disque, sans limite de mémoire** : tables et index vivent dans un fichier B-tree avec un cache de pages borné ; une recherche par clé primaire ou par index ne lit que les lignes nécessaires.
- **Des branches instantanées** : une copie en copy-on-write de toute la base, faite en un instant quelle que soit sa taille, pour un aperçu ou un test.
- **Un seul serveur, toujours allumé**, qui sert la base, la console `/admin` et le tableau de bord en direct `/app` sur un seul port.
- **Depuis WebAssembly aussi** : le client parle HTTP(S), en natif comme avec le `fetch` du navigateur.

Vos modèles et vos requêtes ne changent pas : Nuage DB exécute les instructions de Memoire avec le moteur de Memoire lui-même, donc `req.db()`, le constructeur de requêtes, les relations, les transactions et les migrations fonctionnent comme avec toutes les autres bases.

## Développement et production

Un projet créé par `paquets new` choisit Nuage DB dans `src/main.rs` :

```rust
fn app() -> App {
    let mut app = App::new();
    // Nuage DB: kept in data/ in development, on a Nuage DB server in production
    // (NUAGE_URL and NUAGE_TOKEN), without changing this code.
    app.nuage("data");
    // ...
    app
}
```

L'environnement décide où vont les données :

| Environnement | Base de données | Ligne de démarrage |
|---|---|---|
| rien (développement) | le mode local de Nuage DB, dans le dossier `data/` | `database: nuage, local (data)` |
| `NUAGE_URL=https://mon-nuage.herokuapp.com`, `NUAGE_TOKEN=…` | un serveur Nuage DB | `database: nuage (https://mon-nuage.herokuapp.com)` |
| `NUAGE_URL=autre-dossier` | le mode local, dans ce dossier | `database: nuage, local (autre-dossier)` |
| `DATABASE_URL=…` sans `NUAGE_URL` | cette base : PostgreSQL, SQLite… (voir [Base de données](database.md#choisir-la-base-de-données)) | `database: postgres`… |

`NUAGE_URL` l'emporte sur `DATABASE_URL`, qui l'emporte sur le dossier de `app.nuage(...)`. Une base choisie dans le code avec `app.database(...)` l'emporte sur tout, et les tests créés avec `app.test()` ont toujours une base neuve en mémoire.

**Le mode local** garde la base dans `data/app.memoire`, avec le moteur qui exécute les instructions dans Nuage DB (celui de Memoire) : une requête donne les mêmes lignes sur votre ordinateur et sur le serveur. Le dossier est créé s'il n'existe pas ; `paquets new` l'inscrit dans `.gitignore` et `.dockerignore`. Toute la base est tenue en mémoire pendant que l'application tourne, ce qui convient au développement ; les branches, la console et le tableau de bord viennent avec un serveur, que vous pouvez aussi lancer sur votre machine (section suivante).

## Lancer un serveur sur votre machine

Le serveur est le programme `nuage` du [dépôt Nuage](https://github.com/maxlestage/Nuage) :

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
NUAGE_TOKEN=dev-token NUAGE_ADMIN_PASSWORD=dev-password cargo run -p nuage --release
# nuage: data is stored in /home/moi/Nuage/nuage-data on this server's disk
# nuage: listening on 0.0.0.0:8080
```

Puis, dans votre projet :

```bash
NUAGE_URL=http://localhost:8080 NUAGE_TOKEN=dev-token paquets dev
#   database: nuage (http://localhost:8080) · security: on (10 guards) · islands: /pkg/my_app.js
```

Ouvrez http://localhost:8080/admin pour la console et http://localhost:8080/app pour le tableau de bord (connexion avec `NUAGE_ADMIN_PASSWORD`).

## Déployer un serveur Nuage DB

Le serveur est un seul programme, configuré par des variables d'environnement. Déployez-le une fois, à côté de vos applications :

| Variable | Rôle |
|---|---|
| `PORT` | le port (défini par Heroku et la plupart des hébergeurs ; `8080` sinon) |
| `NUAGE_TOKEN` | le jeton de l'API de la base : sans lui, l'API refuse toutes les requêtes |
| `NUAGE_ADMIN_PASSWORD` | le mot de passe de `/admin` et `/app` : sans lui, la console est éteinte |
| `NUAGE_DATA_DIR` | le dossier des données (`nuage-data` par défaut) |
| `NUAGE_BACKUP_DIR` | sauvegardes dans un dossier |
| `NUAGE_BACKUP_S3_BUCKET`, `_REGION`, `_ENDPOINT`, `_ACCESS_KEY_ID`, `_SECRET_ACCESS_KEY` | sauvegardes sur un stockage compatible S3 |
| `NUAGE_CACHE_MB` | le cache de pages, en Mio (64 par défaut) |

**Sur Heroku**, la racine du dépôt Nuage se déploie telle quelle (`Procfile`, `RustConfig`, `heroku-build.sh`, et `app.json` pour un bouton **Deploy to Heroku**) :

1. Créez l'application avec le buildpack Rust : `heroku create mon-nuage --buildpack emk/rust`, ou depuis le tableau de bord, avec le bouton du dépôt Nuage.
2. Créez un bucket (AWS S3, Cloudflare R2, Scaleway…) et une clé d'accès limitée à ce bucket.
3. Définissez les config vars :

   ```bash
   heroku config:set NUAGE_TOKEN=$(openssl rand -hex 32) \
     NUAGE_ADMIN_PASSWORD='une longue phrase secrète' \
     NUAGE_BACKUP_S3_BUCKET=mon-bucket NUAGE_BACKUP_S3_REGION=eu-west-3 \
     NUAGE_BACKUP_S3_ACCESS_KEY_ID=... NUAGE_BACKUP_S3_SECRET_ACCESS_KEY=...
   ```

4. Déployez, puis ouvrez `https://mon-nuage.herokuapp.com/admin`.

> [!WARNING]
> Le disque d'un dyno est effacé à chaque redémarrage (au moins une fois par jour). Le dossier des données n'est qu'une copie de travail : **les données vivent dans les sauvegardes**, et chaque démarrage les restaure. Configurez les sauvegardes S3 sur Heroku, et faites tourner **exactement un** dyno Basic (il ne se met jamais en veille) : deux dynos seraient deux bases écrivant dans les mêmes sauvegardes. Les validations faites depuis le dernier segment de journal envoyé (30 secondes par défaut) ne sont perdues que si le dyno s'arrête sans le `SIGTERM` qu'Heroku envoie d'abord.

**Avec Docker**, sur n'importe quel hébergeur avec un volume persistant :

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
docker build -t nuage .
docker run -p 8080:8080 -v nuage-data:/data -e NUAGE_TOKEN=... -e NUAGE_ADMIN_PASSWORD=... nuage
```

Servez le serveur en HTTPS (Heroku et la plupart des hébergeurs le font pour vous) : le jeton voyage dans un en-tête, et le cookie de la console est `Secure`.

## Connecter votre application

Donnez à votre application l'adresse et le jeton du serveur. Sur Heroku : votre application → **Settings** → **Reveal Config Vars** :

| Config var | Valeur |
|---|---|
| `NUAGE_URL` | `https://mon-nuage.herokuapp.com` |
| `NUAGE_TOKEN` | le `NUAGE_TOKEN` du serveur |

L'application redémarre, se connecte, vérifie le jeton et crée les tables de vos modèles. Si le serveur est injoignable ou refuse le jeton, l'application ne démarre pas et dit pourquoi. Le jeton ne fait jamais partie de l'URL, et la ligne de démarrage ne l'affiche jamais.

Paquets garde les connexions HTTP vers le serveur ouvertes d'une requête à l'autre, et utilise TLS pour les adresses `https://`. Avec un serveur, votre application tourne sur un runtime tokio multi-thread, comme avec PostgreSQL.

Pour ouvrir la base vous-même, `Db::nuage` ouvre l'un ou l'autre (l'adresse d'un serveur, ou un dossier), et `Db::connect` accepte aussi l'adresse d'un serveur, avec le jeton de `NUAGE_TOKEN`. Faites-le depuis un `main` asynchrone qui démarre l'application avec `app.serve().await` (voir [Configuration](configuration.md#initialisation-asynchrone)), pour que les connexions vivent sur le runtime de l'application :

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("NUAGE_TOKEN").ok();
    let db = Db::nuage("https://mon-nuage.herokuapp.com", token.as_deref())
        .await
        .map_err(std::io::Error::other)?;
    let mut app = app();
    app.database(db);
    app.serve().await
}
```

## Des branches pour les aperçus et les tests

Une branche est une copie de la base faite en un instant, quelle que soit sa taille : elle partage toutes les pages de sa parente et ne stocke que ses propres changements. Les changements ultérieurs de la base principale ne l'atteignent pas.

**Aperçus.** Définissez `NUAGE_BRANCH` à côté de `NUAGE_URL` sur une application d'aperçu (une review app Heroku, une application de recette) : au démarrage, Paquets crée la branche à partir de la base principale si elle n'existe pas encore, puis exécute toutes les requêtes dessus. L'aperçu travaille sur de vraies données et ne peut pas toucher à celles de la production.

```text
NUAGE_URL=https://mon-nuage.herokuapp.com
NUAGE_TOKEN=…
NUAGE_BRANCH=apercu-42
```

Réinitialisez ou supprimez une branche depuis la console (**Branches**), ou dans le code, avec le client de `Db::Nuage` :

```rust
if let Db::Nuage(nuage) = req.db() {
    nuage.reset_branch("apercu-42").await?; // oublie ses changements
    nuage.delete_branch("apercu-42").await?;
}
```

**Tests.** `app.test()` utilise une base en mémoire : rapide, et le bon choix pour la plupart des tests. Pour tester sur une copie de vraies données, donnez à un test sa propre branche :

```rust
use paquets::orm::NuageDb;

#[tokio::test]
async fn sur_une_copie_de_la_production() -> paquets::orm::Result<()> {
    let (Ok(url), Ok(token)) = (std::env::var("NUAGE_URL"), std::env::var("NUAGE_TOKEN")) else {
        return Ok(()); // pas de serveur : ignoré
    };
    let server = NuageDb::connect(&url, &token).await?;
    let name = format!("test-{}", std::process::id());
    server.create_branch(&name, None).await?;

    let mut app = app();
    app.database(Db::from(server.on_branch(&name)));
    let client = app.test().await;
    assert_eq!(client.get("/").await.status(), 200);

    server.delete_branch(&name).await?;
    Ok(())
}
```

## La console et le tableau de bord

Le serveur que vous avez déployé a deux pages, protégées par `NUAGE_ADMIN_PASSWORD` :

- **`/admin`**, la console : une vue d'ensemble (taille, tables, lignes, branches, temps de fonctionnement, connexions, cache, dernier point de contrôle, sauvegardes), chaque table et ses lignes page par page, une console de requêtes (un petit sous-ensemble de SQL, avec une confirmation avant les modifications), les branches, et le stockage, avec des boutons pour un point de contrôle, une sauvegarde, une vérification d'intégrité et un compactage. Elle est rendue par le serveur et fonctionne sans JavaScript, sur un téléphone aussi.
- **`/app`**, le tableau de bord en direct : requêtes par seconde, latence, connexions, stockage, cache, journal d'écriture, sauvegardes, bases et branches, et un journal des événements en direct, rafraîchi chaque seconde. Il s'installe comme une application sur un téléphone.

`GET /api/health` répond `{"status":"ok"}` aux moniteurs de disponibilité. Avec `NUAGE_PUBLIC_STATUS=true`, `/app` montre une partie publique sans connexion.

## Depuis le navigateur

Le client tourne aussi en WebAssembly, dans un îlot, avec le `fetch` du navigateur. Ajoutez la feature `nuage` au côté navigateur de `Cargo.toml` :

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client", "nuage"] }
```

`paquets::orm::NuageDb` est alors disponible dans `src/lib.rs`, avec les mêmes modèles et les mêmes requêtes.

> [!WARNING]
> Le jeton ouvre toute la base. Ne l'envoyez au navigateur que dans une application pour vous ou votre équipe (un tableau de bord interne derrière une connexion), jamais dans une page publique : les pages publiques lisent et écrivent par l'API de votre serveur, qui garde le jeton sur le serveur.

## Passer un projet existant à Nuage DB

Les projets créés avant la 0.3.3 ne changent pas : sans `NUAGE_URL` et sans `app.nuage(...)`, la base vient toujours de `DATABASE_URL`, et elle est en mémoire sans elle. Pour passer à Nuage DB :

1. Dans `src/main.rs`, après `App::new()`, ajoutez `app.nuage("data");`.
2. Ajoutez `/data` à `.gitignore` et `data` à `.dockerignore`.
3. Dans le `Dockerfile`, avant `USER app`, ajoutez `RUN mkdir data && chown app data`, pour que l'application puisse écrire dans le dossier quand `NUAGE_URL` n'est pas définie.
4. Déployez un serveur Nuage DB, et définissez `NUAGE_URL` et `NUAGE_TOKEN` sur l'application.

Pour copier les données existantes, lancez un programme ponctuel, ou un [seed](database.md#données-au-démarrage), qui lit l'ancienne base et écrit dans la nouvelle :

```rust
let old = Db::connect(&std::env::var("OLD_DATABASE_URL").unwrap()).await?;
let token = std::env::var("NUAGE_TOKEN").ok();
let new = Db::nuage(&std::env::var("NUAGE_URL").unwrap(), token.as_deref()).await?;
new.push::<Todo>().await?;
let todos = Todo::query().all(&old).await?;
Todo::insert_many(&new, &todos).await?;
```

La feature `nuage` est activée par défaut. Un projet qui ne l'utilise pas peut s'en passer avec `default-features = false` (voir [Installation](installation.md#features-cargo)).

## Limites

- **Pas de SQL.** Nuage DB exécute les instructions de Memoire : `db.raw(...)` et les étapes `.sql(...)` des migrations répondent « non pris en charge », comme avec la base en mémoire. Pour du SQL écrit à la main, utilisez PostgreSQL via `DATABASE_URL`.
- **Un seul écrivain à la fois.** Les écritures s'appliquent l'une après l'autre, les lectures en parallèle ; un serveur par base.
- **Le mode local sert au développement** : il tient toute la base en mémoire pendant que l'application tourne, et n'a ni branches, ni console, ni tableau de bord (lancez un serveur pour cela).
- **Les jointures, `GROUP BY` et les agrégats** autres que `COUNT(*)` lisent en mémoire les lignes qu'ils combinent, après les filtres de chaque table.

Le guide complet de Nuage DB, avec ses sauvegardes, ses transactions et tous ses réglages, est dans [la documentation de Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/nuage/) et le [dépôt Nuage](https://github.com/maxlestage/Nuage).

## Étapes suivantes

- [Base de données](database.md) : modèles, requêtes, transactions et les autres bases.
- [Configuration](configuration.md) : toutes les variables d'environnement.
- [Déployer sur Heroku](deploy-heroku-mobile.md) : votre application, depuis votre téléphone.
