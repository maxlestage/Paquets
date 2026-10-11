# Performances

Paquets est rapide sans réglage : son serveur, Vitesse, est plus rapide qu'actix-web et axum sur la même machine, son bouclier de sécurité coûte quelques microsecondes par requête, et les pages n'envoient aucun code au navigateur, sauf si elles contiennent des îlots. Cette page donne les chiffres, explique comment le serveur utilise votre processeur, et liste ce qui garde votre propre application rapide.

## Les chiffres

### Le serveur (Vitesse)

Requêtes par seconde sur une VM à 4 vCPU, le serveur épinglé sur 2 cœurs et le générateur de charge (wrk) sur les 2 autres, 128 connexions keep-alive. Entre parenthèses : le temps CPU du serveur par requête (plus c'est bas, mieux c'est).

| Scénario | axum 0.8 | actix-web 4 | **Vitesse** |
|---|---:|---:|---:|
| `GET /` (texte) | 168 165 (11,7 µs) | 274 195 (7,1 µs) | **291 415 (6,1 µs)** |
| `GET /json` | 165 988 (11,9 µs) | 248 987 (7,9 µs) | **313 438 (5,9 µs)** |
| `GET /json` envoyé par un navigateur (12 en-têtes) | 131 710 (15,0 µs) | 179 505 (10,9 µs) | **290 478 (6,8 µs)** |
| `GET /users/:id` (paramètre + JSON) | 151 303 (13,1 µs) | 209 173 (9,4 µs) | **304 323 (6,2 µs)** |
| `POST /echo` (lit et renvoie du JSON) | 105 570 (18,8 µs) | 170 129 (11,7 µs) | **253 941 (7,6 µs)** |
| `GET /` pipeliné ×16 | 213 678 (9,3 µs) | 1 272 510 (1,6 µs) | **2 781 541 (0,64 µs)** |

La méthodologie, les autres frameworks mesurés (Express, Drogon) et la façon de reproduire ces chiffres se trouvent dans [le guide des performances de Vitesse](https://maxlestage.github.io/Vitesse/fr/docs/performance/).

### Le bouclier (Securite)

Toute la chaîne recommandée (identifiant de requête, hygiène, filtre de méthodes, limite de corps, limitation de débit, blocage des robots, pare-feu, CSRF, en-têtes de sécurité, durcissement des cookies) coûte environ **6 µs** par requête.

### Pages et îlots (Active)

| | Rendu serveur d'une page produit | WebAssembly + chargeur pour un compteur (compressé) | Interactif avec 100 ms de latence |
|---|---:|---:|---:|
| **active** | **5,6 µs** | **36,8 ko** | **224 ms** |
| Leptos 0.8 + leptos_meta | 13,4 µs | 60,9 ko | 227 ms |
| Sycamore 0.9 | 33,0 µs | 45,8 ko | 227 ms |
| Dioxus 0.7 | 24,2 µs | 170,4 ko | 339 ms |

Le plus petit bundle de tous, et un rendu serveur 2,4 fois plus rapide que l'autre framework qui fusionne les balises SEO venant des composants (Leptos avec leptos_meta). Un moteur de templates sans fusion SEO, comme maud, rend plus vite (1,4 µs), mais n'envoie rien au navigateur. Les détails sont dans [les benchmarks d'active](https://maxlestage.github.io/Active/fr/docs/benchmarks/).

### Ce que cela signifie pour votre application

Ces benchmarks mesurent chaque paquet seul, sur des handlers triviaux. Dans une vraie application Paquets, une requête passe aussi par le bouclier et par le journal des requêtes, et la plupart des handlers interrogent une base de données. En pratique, **c'est la base de données et votre propre code qui prennent le temps** : la part du framework se compte en microsecondes. Les conseils ci-dessous portent donc sur eux.

## Comment le serveur utilise votre processeur

- Avec la base de données en mémoire ou SQLite, `app.run()` démarre **un thread par cœur de processeur**, chacun avec sa propre boucle d'événements et sa propre socket d'écoute. Le noyau répartit les connexions entre les threads, et une requête ne change jamais de thread. C'est le mode le plus rapide de Vitesse.
- Avec un serveur de base de données (PostgreSQL…), l'application tourne sur le runtime multithread de tokio, partagé avec les connexions à la base, et les threads inactifs prennent du travail aux threads occupés.

Dans les deux modes, un handler qui bloque son thread (un long calcul, une lecture de fichier bloquante, `std::thread::sleep`) retarde toutes les autres requêtes de ce thread. Gardez des handlers asynchrones, et sortez les calculs lourds de la boucle d'événements :

```rust
use paquets::vitesse::tokio::task;

async fn report(_req: Request) -> Result<String> {
    // Un long calcul : l'exécuter sur un thread prévu pour le travail bloquant.
    let total = task::spawn_blocking(|| (1..=50_000_000u64).sum::<u64>()).await?;
    Ok(format!("total : {total}"))
}
```

Le hachage des mots de passe, le traitement d'images et les grosses compressions ont leur place dans `spawn_blocking`.

## Conseils

### Compiler en mode release

Une compilation debug (`cargo run`, `paquets dev`) peut être dix fois plus lente, voire davantage. Ne mesurez et ne déployez que des compilations release : `paquets build`, ou `cargo build --release`. Le `Dockerfile` d'un projet généré compile en mode release, avec l'optimisation à l'édition de liens.

### Base de données

- **Ajoutez des index** sur les colonnes sur lesquelles vous filtrez ou triez : `#[memoire(index)]` (les colonnes uniques sont déjà indexées).
- **Ne lisez que ce que vous affichez** : `.limit(...)` ou `.paginate(...)` sur les listes, `.columns(...)` quand il vous faut une ou deux colonnes, `.count(...)` ou `.exists(...)` plutôt que de charger des lignes pour les compter.
- **Évitez une requête par ligne** (le problème « N+1 »). Chargez les lignes liées en une seule requête avec `is_in` :

```rust
async fn posts_with_authors(req: Request) -> Result<Json<Vec<(String, String)>>> {
    let db = req.db();
    let posts = Post::query().order_by(Post::ID.desc()).limit(20).all(db).await?;
    // Une seule requête pour tous les auteurs, au lieu d'une par article.
    let ids: Vec<i64> = posts.iter().map(|p| p.author_id).collect();
    let authors = Author::query().filter(Author::ID.is_in(ids)).all(db).await?;
    let rows = posts
        .into_iter()
        .map(|post| {
            let name = authors.iter().find(|a| a.id == post.author_id).map_or("?", |a| a.name.as_str());
            (post.title, name.to_string())
        })
        .collect();
    Ok(Json(rows))
}
```

- **Regroupez les écritures** dans une transaction, ou utilisez `insert_many`, `update_many` et `delete_many`.
- Memoire construit le SQL des opérations par clé primaire (`get`, `insert`, `update`, `delete`) à la compilation : elles ne coûtent aucune génération de SQL à l'exécution.

### Pages

- Une page sans îlots n'envoie pas la moindre ligne de JavaScript. N'ajoutez des îlots que là où la page doit réagir dans le navigateur, et gardez leurs props légères : elles font partie du HTML.
- Le thème intégré pèse environ 3,5 ko, en ligne dans chaque page. Pour une grosse feuille de style commune à toutes les pages, placez-la dans `public/` et liez-la avec `.stylesheet(...)` : les navigateurs la gardent en cache.

### Fichiers statiques

Les fichiers de `public/` sont envoyés avec `ETag` et `Last-Modified`, et les navigateurs les revalident (`304 Not Modified`). Pour les fichiers dont le nom change avec le contenu (`app.3f9c2b.css`), autorisez un long cache avec un dossier dédié :

```rust
use std::time::Duration;
use paquets::vitesse::ServeDir;

app.serve_dir("/assets", ServeDir::new("assets").max_age(Duration::from_secs(31_536_000)));
```

### Compression

Vitesse ne compresse pas les réponses. Le texte se compresse très bien (HTML, JSON, CSS, WebAssembly) : placez donc un CDN (Cloudflare, Fastly…) ou un reverse proxy devant l'application si la bande passante compte. Le routeur de Heroku ne compresse pas non plus.

### Journaux

Une ligne de journal par requête coûte peu, mais écrire dans le terminal n'est pas gratuit sous forte charge. `app.quiet()` désactive le journal des requêtes, pour les benchmarks ou quand votre plateforme journalise déjà les requêtes.

## Mesurer

Compilez en mode release, puis chargez l'application avec un outil comme [oha](https://crates.io/crates/oha) ou wrk :

```sh
paquets build
./target/release/mon-app &
oha -z 10s -c 64 http://127.0.0.1:3000/
```

> [!IMPORTANT]
> Toutes les requêtes d'un test de charge viennent d'une seule adresse IP : la limitation de débit du bouclier (20 requêtes par seconde et par IP, pointes de 100) répond donc `429` à la plupart d'entre elles, et c'est le limiteur que vous mesurez. Pour un benchmark sur votre propre machine, retirez la limitation de débit et le journal des requêtes :
>
> ```rust
> app.security(paquets::shield().without("rate_limit").build());
> app.quiet();
> ```
>
> Ne déployez jamais cette configuration : c'est la limitation de débit qui protège votre application contre les inondations de requêtes.

Mesurez ce que font vos utilisateurs : les pages et les routes d'API qui comptent, avec des données réalistes dans la base.

## Étapes suivantes

- [Configuration](configuration.md) : workers, threads et runtime.
- [Base de données](database.md) : requêtes, index et transactions.
- [Déployer avec Docker](deploy-docker.md) : une image de production optimisée.
