# Base de données

Paquets enregistre les données avec [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/), un ORM typé : une struct Rust décrit une table, et les requêtes s'écrivent en Rust, vérifiées par le compilateur. Le même code fonctionne en mémoire, sur SQLite, PostgreSQL, MySQL/MariaDB, Redis ou MongoDB : la base de données est choisie par une URL. Cette page couvre les modèles, le choix de la base, la lecture et l'écriture, les transactions, les changements de schéma et le SQL brut.

## Modèles

Un modèle est une struct marquée avec `#[model]` :

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,      // la clé primaire, générée par la base de données
    title: String,
    done: bool,
}
```

`#[model]` fait de la struct une table de base de données **et** une valeur JSON. Il ajoute `#[derive(Clone, Debug, Serialize, Deserialize, Model)]` (les derives que vous avez déjà écrits sont conservés) et fait pointer le code généré vers les crates réexportées par Paquets : vous n'avez besoin d'aucune autre dépendance.

| Règle | Détails |
|---|---|
| Nom de la table | le nom de la struct en `snake_case`, sans pluriel : `Todo` → `todo`, `BlogPost` → `blog_post`. Choisissez un autre nom avec `#[model(table = "todos")]`. |
| Clé primaire | le champ nommé `id`. Un `id` entier (`i64`, `i32` ou `i16`) est généré par la base de données à l'insertion d'une ligne. |
| `id` en JSON | un `id` entier peut être omis dans les corps JSON : les clients qui créent une ligne ne le connaissent pas encore. |
| Colonnes | une par champ, avec le même nom. `Option<T>` rend une colonne nullable ; toutes les autres colonnes sont `NOT NULL`. |
| Colonnes typées | chaque champ reçoit une constante en `UPPER_SNAKE_CASE`, utilisée dans les requêtes : `Todo::TITLE`, `Todo::DONE`. |

### Types de colonnes

| Type Rust | Remarques |
|---|---|
| `bool` | |
| `i16`, `i32`, `i64` (et `i8`, `u8`, `u16`, `u32`) | `u64`, `usize` et `i128` ne sont pas pris en charge : utilisez `i64` |
| `f32`, `f64` | |
| `String` | |
| `Vec<u8>` | données binaires |
| `Option<T>` | n'importe lequel des types ci-dessus, nullable |
| `chrono::NaiveDateTime`, `DateTime<Utc>`, `NaiveDate`, `NaiveTime` | feature `chrono` |
| `uuid::Uuid` | feature `uuid` |
| `serde_json::Value`, `Json<T>` | feature `json` |

### Attributs de colonne

Les attributs de Memoire s'appliquent aux champs, dans `#[memoire(...)]` :

```rust
#[model(table = "users")]
struct User {
    id: i64,
    #[memoire(unique)]
    email: String,
    #[memoire(index)]
    name: String,
    #[memoire(default = "0")]
    visits: i32,
    bio: Option<String>,
}
```

| Attribut | Effet |
|---|---|
| `#[memoire(unique)]` | une deuxième ligne avec la même valeur est refusée : `409 Conflict` dans un handler |
| `#[memoire(index)]` | un index, pour les colonnes sur lesquelles vous filtrez ou triez souvent |
| `#[memoire(column = "e_mail")]` | un autre nom pour la colonne |
| `#[memoire(default = "0")]` | une valeur par défaut SQL, utilisée quand vous ajoutez la colonne à une table existante |
| `#[memoire(references = "users(id)", on_delete = "cascade")]` | une clé étrangère |
| `#[memoire(primary_key)]` | un autre champ comme clé primaire, par exemple un `slug: String` |
| `#[memoire(fulltext)]` | prépare une colonne de texte à la [recherche plein texte](#requêtes) avec `matches` : un index GIN sur PostgreSQL, un index de mots en mémoire |
| `#[memoire(skip)]` | un champ qui n'est pas enregistré (son type doit implémenter `Default`) |

Sur la struct, `#[memoire(unique(a, b))]` et `#[memoire(index(a, b))]` portent sur plusieurs colonnes à la fois, et peuvent être répétés. `unique(user_id, team_id)` refuse une deuxième ligne avec la même paire (`409 Conflict`), alors que chaque colonne seule peut se répéter :

```rust
#[model(table = "memberships")]
#[memoire(unique(user_id, team_id), index(team_id, role))]
struct Membership {
    id: i64,
    user_id: i64,
    team_id: i64,
    role: String,
}
```

### Champs facultatifs en JSON

Un champ `bool` ou `String` est obligatoire dans les corps JSON. Pour permettre aux clients de l'omettre, ajoutez `#[serde(default)]` (le champ vaut alors `false`, `0` ou une chaîne vide), ou faites-en une `Option` :

```rust
#[model]
struct Task {
    id: i64,
    title: String,
    #[serde(default)]
    done: bool,          // peut être omis : false
    due: Option<String>, // peut être omis ou null : NULL dans la base
}
```

## Choisir la base de données

La base de données est choisie par la variable d'environnement `DATABASE_URL` au démarrage de l'application :

| `DATABASE_URL` | Base de données | Feature Cargo |
|---|---|---|
| *(non définie)* ou `memory` | en mémoire, vidée à l'arrêt du serveur | toujours disponible |
| `memory://data/app.memoire` | en mémoire, gardée dans un fichier et relue au démarrage | toujours disponible |
| `sqlite://app.db`, `app.db`, `sqlite::memory:` | un fichier SQLite, créé (avec son dossier) s'il n'existe pas | `sqlite` (par défaut) |
| `postgres://user:password@host:5432/name` | PostgreSQL, avec TLS quand le serveur le propose | `postgres` (par défaut) |
| `mysql://…`, `mariadb://…` | MySQL ou MariaDB | `mysql` |
| `redis://…`, `rediss://…` | Redis | `redis` |
| `mongodb://…`, `mongodb+srv://…` | MongoDB | `mongodb` |

```sh
DATABASE_URL=sqlite://app.db paquets dev
```

La ligne de démarrage vous indique celle qui est utilisée : `database: sqlite`. Sur Heroku, ajouter l'add-on Heroku Postgres définit `DATABASE_URL` pour vous : rien à changer dans le code.

Vous pouvez aussi choisir la base de données dans le code. Elle l'emporte alors sur `DATABASE_URL` :

```rust
let mut app = App::new();
app.database("sqlite://app.db"); // un fichier là où le programme s'exécute
// ou : app.database(Db::memory());
```

Si l'URL est fausse ou que le serveur est injoignable, l'application ne démarre pas, et dit pourquoi. Une URL qui a besoin d'une feature que vous n'avez pas activée vous indique même quoi écrire dans `Cargo.toml` :

```text
📦 Paquets could not start: database: operation not supported by this backend: `mysql` URLs need the `mysql` feature: paquets = { git = "https://github.com/maxlestage/Paquets", features = ["mysql"] }
```

> [!NOTE]
> Les tests créés avec `app.test()` utilisent toujours une base de données neuve en mémoire, sauf si le test appelle lui-même `app.database(...)`. `DATABASE_URL` y est ignorée : les tests ne touchent donc jamais à vos données.

### Une base en mémoire gardée dans un fichier

`DATABASE_URL=memory://data/app.memoire` garde la rapidité du moteur en mémoire (données, index et transactions en mémoire) et écrit en plus chaque modification validée dans le fichier, qui est relu quand l'application redémarre. Le fichier (et son dossier) est créé s'il n'existe pas, et compacté de temps en temps. Les tables reviennent quand leur modèle est utilisé, en retrouvant leurs colonnes par leur nom.

Avec la feature `encryption`, le fichier peut être chiffré avec un mot de passe (AES-256-GCM, une clé dérivée avec Argon2id). Ouvrez-le vous-même et donnez-le à l'application :

```rust
use paquets::orm::{MemoryDb, StorageOptions};

let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD");
let db = MemoryDb::open_with("data/app.memoire", StorageOptions::new().password(password))
    .map_err(std::io::Error::other)?;
app.database(Db::from(db));
```

Comme SQLite, cette base a besoin d'un disque persistant : pas sur Heroku. Voir [le moteur en mémoire de Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/memory/) pour la durabilité (`StorageOptions::sync`) et le compactage.

## Les tables sont créées au démarrage

Au démarrage, l'application crée la table de chaque modèle enregistré avec `app.api::<M>(...)` (voir [API REST](rest-api.md)) ou `app.model::<M>()`, **si la table n'existe pas encore**. Utilisez `app.model` pour les modèles dont vous ne vous servez que dans vos propres handlers :

```rust
let mut app = App::new();
app.api::<Todo>("/api/todos"); // table + routes REST
app.model::<User>();            // table seulement
```

Une table existante n'est jamais modifiée : quand vous ajoutez plus tard un champ à un modèle, voir [Modifier une table plus tard](#modifier-une-table-plus-tard).

## Lire des données

Dans un handler, `req.db()` renvoie la base de données. Chaque méthode de modèle la prend en premier argument :

```rust
async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si ce n'est pas un nombre
    let todo = Todo::get(req.db(), id).await?; // 404 si cette ligne n'existe pas
    Ok(Json(todo))
}
```

| Code | Renvoie |
|---|---|
| `Todo::all(db).await?` | `Vec<Todo>` : toutes les lignes |
| `Todo::get(db, id).await?` | `Todo`, ou une erreur « introuvable » (`404` dans un handler) |
| `Todo::find(db, id).await?` | `Option<Todo>` |
| `Todo::find_many(db, [1, 2, 3]).await?` | `Vec<Todo>` : les lignes de ces clés, en une seule requête |
| `Todo::count(db).await?` | `u64` |
| `todo.reload(db).await?` | rafraîchit une struct depuis la base de données |

### Requêtes

`Todo::query()` commence une requête. Enchaînez les filtres, le tri et les limites, puis exécutez-la :

```rust
async fn open_todos(req: Request) -> Result<Json<Vec<Todo>>> {
    let todos = Todo::query()
        .filter(Todo::DONE.eq(false))
        .filter(Todo::TITLE.contains("docs"))
        .order_by(Todo::ID.desc())
        .limit(20)
        .all(req.db())
        .await?;
    Ok(Json(todos))
}
```

Les constantes de colonne sont typées : `Todo::DONE.eq("yes")` ne compile pas, car `done` est un `bool`.

| Filtre | Code |
|---|---|
| égal, différent | `Todo::DONE.eq(false)`, `.ne(false)` |
| comparaisons | `User::VISITS.gt(10)`, `.ge(10)`, `.lt(10)`, `.le(10)` |
| et, ou, non | `a & b`, `a \| b`, `!a`, ou plusieurs appels à `.filter(...)` (et) |
| dans une liste | `Todo::ID.is_in([1, 2, 3])` |
| entre | `User::VISITS.between(1, 10)` |
| null | `User::BIO.is_null()`, `.is_not_null()` |
| texte | `.contains("x")`, `.starts_with("x")`, `.ends_with("x")`, `.icontains("x")` (sans tenir compte de la casse), `.like("%x%")` |
| recherche plein texte | `Book::TITLE.matches("rust orm")` : les lignes dont le texte contient tous les mots, en mots entiers, sans tenir compte de la casse, sur toutes les bases |

| Exécuter avec | Renvoie |
|---|---|
| `.all(db)` | `Vec<Todo>` |
| `.first(db)` | `Option<Todo>` |
| `.one(db)` | `Todo`, ou « introuvable » |
| `.count(db)` | `u64` |
| `.exists(db)` | `bool` |
| `.paginate(db, page, per_page)` | une page de résultats : `items`, `page`, `per_page` et `total` |
| `.columns(Todo::TITLE).all(db)` | `Vec<String>` : seulement cette colonne |

Tri et pagination : `.order_by(Todo::TITLE)` (croissant), `.order_by(Todo::ID.desc())`, `.limit(20)`, `.offset(40)`, `.page(3, 20)`.

Une liste paginée dans une page :

```rust
async fn todo_list(req: Request) -> Result<Page> {
    let page: u64 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let result = Todo::query().order_by(Todo::ID.asc()).paginate(req.db(), page, 20).await?;
    Ok(Page::new(format!("Tâches, page {page}")).body(
        section()
            .child(p().class("muted").text(format!("{} tâches au total", result.total)))
            .child(ul().children(result.items.into_iter().map(|t| li().text(t.title)))),
    ))
}
```

La valeur renvoyée par `paginate` est le type `Page` propre à Memoire (`paquets::orm::Page`), pas la `Page` HTML : vous n'utilisez que ses champs.

## Lignes liées

Une colonne déclarée avec `references` relie deux modèles, et Memoire y lit le lien : il n'y a rien d'autre à déclarer.

```rust
#[model(table = "users")]
struct User {
    id: i64,
    name: String,
}

#[model(table = "posts")]
struct Post {
    id: i64,
    #[memoire(references = "users(id)", index)]
    author_id: i64,
    title: String,
}
```

| Code | Renvoie |
|---|---|
| `Post::belonging_to(&user)` | une requête sur les articles d'un utilisateur, à filtrer, trier ou compter comme n'importe quelle autre |
| `post.find_parent::<User, _>(db).await?` | `Option<User>` : l'auteur d'un article |
| `users.load_children::<Post>(db).await?` | `Vec<Vec<Post>>` : les articles de chaque utilisateur, dans le même ordre que `users` |
| `posts.load_parent::<User>(db).await?` | `Vec<Option<User>>` : l'auteur de chaque article |
| `posts.load_via::<PostTag, Tag>(db).await?` | plusieurs-à-plusieurs, à travers un modèle qui référence les deux |

`load_children`, `load_parent` et `load_via` (le trait `LoadRelated`, dans le prélude) font une requête par niveau, quel que soit le nombre de lignes de départ, au lieu d'une requête par ligne :

```rust
async fn authors(req: Request) -> Result<Page> {
    let users = User::query().order_by(User::NAME.asc()).all(req.db()).await?;
    let posts = users.load_children::<Post>(req.db()).await?; // une seule requête pour tous les articles
    Ok(Page::new("Auteurs").body(ul().children(users.into_iter().zip(posts).map(
        |(user, posts)| li().text(format!("{} : {} articles", user.name, posts.len())),
    ))))
}
```

## Écrire des données

```rust
// Insérer : la struct doit être `mut`, l'id généré y est réécrit.
let mut todo = Todo { id: 0, title: "Écrire la doc".into(), done: false };
todo.insert(db).await?;
println!("nouvel id : {}", todo.id);

// Mettre à jour toutes les colonnes de la ligne qui a cet id (« introuvable » si elle n'existe pas).
todo.done = true;
todo.update(db).await?;

// Supprimer : `true` si une ligne a été supprimée.
let deleted: bool = Todo::delete_by_key(db, todo.id).await?;
```

| Code | Effet |
|---|---|
| `row.insert(db)` | insère la ligne et y réécrit l'`id` généré |
| `Todo::insert_many(db, &rows)` | insère plusieurs lignes, renvoie leur nombre |
| `row.upsert(db)` | insère, ou remplace la ligne qui a la même clé |
| `row.update(db)` | enregistre toutes les colonnes de la ligne |
| `row.update_only(db, (Todo::TITLE, Todo::DONE))` | enregistre seulement ces colonnes |
| `row.delete(db)` | supprime la ligne, renvoie un `bool` |
| `Todo::delete_by_key(db, id)` | supprime par clé, renvoie un `bool` |
| `Todo::update_many().set(Todo::DONE, true).filter(...).execute(db)` | met à jour plusieurs lignes, renvoie leur nombre |
| `Todo::delete_many().filter(...).execute(db)` | supprime plusieurs lignes, renvoie leur nombre |

`update_many` propose aussi `.increment(colonne, n)`, `.decrement(colonne, n)` et `.set_null(colonne)`.

## Transactions

Une transaction regroupe plusieurs modifications : soit elles sont toutes enregistrées, soit aucune. Commencez-en une avec `db.begin()`, passez `&tx` à la place de la base de données, puis appelez `commit` :

```rust
#[model]
struct Account {
    id: i64,
    balance: i64,
}

#[data]
struct Transfer {
    from: i64,
    to: i64,
    amount: i64,
}

async fn transfer(req: Request) -> Result<StatusCode> {
    let t: Transfer = req.json().await?;
    let tx = req.db().begin().await?;
    Account::update_many()
        .decrement(Account::BALANCE, t.amount)
        .filter(Account::ID.eq(t.from))
        .execute(&tx)
        .await?;
    Account::update_many()
        .increment(Account::BALANCE, t.amount)
        .filter(Account::ID.eq(t.to))
        .execute(&tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
```

Si quoi que ce soit échoue avant `commit` (un `?` qui sort plus tôt), la transaction est abandonnée et toutes les modifications sont annulées.

## Données au démarrage

`app.on_start` s'exécute une seule fois, après l'ouverture de la base de données et la création des tables, avant la première requête. Servez-vous-en pour ajouter des données initiales :

```rust
app.on_start(|db| async move {
    if Todo::count(db).await? == 0 {
        Todo { id: 0, title: "Essayer Paquets".into(), done: false }.insert(db).await?;
    }
    Ok(())
});
```

Si le hook renvoie une erreur, l'application ne démarre pas, et le message commence par `startup:`. Vous pouvez appeler `on_start` plusieurs fois : les hooks s'exécutent dans l'ordre.

Un hook peut aussi lancer un travail de fond avec `tokio::spawn`, comme une purge toutes les minutes : la tâche tourne aussi longtemps que le serveur, quelle que soit la base.

```rust
app.on_start(|db| async move {
    tokio::spawn(async move {
        let mut minute = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            minute.tick().await;
            let _ = Todo::delete_many().filter(Todo::DONE.eq(true)).execute(db).await;
        }
    });
    Ok(())
});
```

## Modifier une table plus tard

Les tables sont créées quand elles manquent, mais **jamais modifiées**. Si vous ajoutez un champ à un modèle dont la table existe déjà (dans un fichier SQLite ou sur Heroku Postgres), la colonne manque, et les requêtes sur ce modèle échouent avec une `500`. Avec la base de données en mémoire (développement, tests), il n'y a rien à faire : la table est recréée à chaque démarrage.

Pour une base existante, utilisez une **migration** : une modification numérotée, enregistrée dans la base de données pour ne s'exécuter qu'une seule fois. Le `Migrator` de Memoire s'en charge, et `on_start` est l'endroit où le lancer. Cet exemple ajoute une colonne `priority` à la table `todo` sur PostgreSQL :

```rust
use paquets::orm::{Dialect, Migration, Migrator};

app.on_start(|db| async move {
    Migrator::new()
        .add(Migration::new(1, "add_todo_priority").sql_for(
            Dialect::Postgres,
            "ALTER TABLE todo ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 0",
        ))
        .run(db)
        .await?;
    Ok(())
});
```

Ajoutez ensuite `priority: i32` à la struct. Le fonctionnement, et les règles à suivre :

- `sql_for(Dialect::Postgres, ...)` ne s'exécute que sur PostgreSQL ; il est ignoré sur la base de données en mémoire, et vos tests continuent donc de fonctionner. Il existe aussi `Dialect::Sqlite` et `Dialect::MySql`, et `.sql(...)` pour toutes les bases SQL.
- `IF NOT EXISTS` rend la modification sûre sur une base toute neuve aussi : la table vient alors d'être créée à partir de la struct actuelle et a déjà la colonne.
- Ne modifiez jamais une migration qui s'est déjà exécutée quelque part : ajoutez-en une nouvelle avec le numéro suivant.
- N'appelez pas `create_table::<M>()` dans ces migrations pour un modèle enregistré avec `app.api` ou `app.model` : sa table a déjà été créée avant l'exécution de `on_start`, et la migration échouerait avec « table already exists ».

> [!NOTE]
> SQLite et MySQL ne prennent pas en charge `ADD COLUMN IF NOT EXISTS`. Pour eux, lancez vous-même les migrations **avant** que l'application ne crée ses tables : ouvrez la base de données avec `Db::connect(&url).await?`, lancez le `Migrator` (avec des étapes `create_table` construites à partir de copies figées de vos structs), puis confiez la base à l'application avec `app.database(db)` et démarrez-la avec `app.serve().await` dans `#[tokio::main]`. Le [guide des migrations](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/migrations/) de Memoire explique en détail les structs figées.

## SQL brut

Quand le constructeur de requêtes ne suffit pas, écrivez du SQL. Les paramètres s'écrivent `?` sur SQLite et MySQL, `$1`, `$2`… sur PostgreSQL. Les lignes sont décodées par position, en tuples ou en structs :

```rust
use paquets::orm::params;

async fn top_visitors(req: Request) -> Result<Json<Vec<(i64, String)>>> {
    let rows: Vec<(i64, String)> = req
        .db()
        .raw("SELECT id, name FROM users WHERE visits > ? ORDER BY visits DESC", &params![10])
        .await?;
    Ok(Json(rows))
}
```

`db.raw_execute(sql, &params![...])` exécute une instruction qui ne renvoie pas de lignes et donne le nombre de lignes touchées. Le SQL brut fonctionne sur SQLite, PostgreSQL et MySQL ; la base de données en mémoire, Redis et MongoDB renvoient une erreur « non pris en charge ».

## Erreurs

Les erreurs de base de données deviennent des statuts HTTP quand elles atteignent un handler par `?` :

| Erreur | Statut |
|---|---|
| ligne inexistante (`get`, `one`, `update`) | `404 Not Found` : `{"error":"record not found"}` |
| valeur en double dans une colonne `unique` | `409 Conflict` : ``{"error":"unique constraint violated on `user.email`"}`` |
| autre contrainte (clé étrangère…), conflit de transaction | `409 Conflict` |
| `NULL` dans un champ qui n'est pas une `Option`, valeur invalide | `400 Bad Request` |
| tout le reste (connexion perdue, table ou colonne manquante…) | `500 Internal Server Error` ; les détails partent dans les journaux du serveur, jamais vers le client |

Pour réagir vous-même à une erreur, examinez-la avant le `?` : `Todo::find` renvoie `None` au lieu d'une erreur, et les erreurs de Memoire ont `is_not_found()`, `is_unique_violation()` et `is_conflict()`. Voir [Erreurs](errors.md).

## Quelle base de données choisir ?

| Situation | Choix |
|---|---|
| Apprentissage, prototypes, tests | en mémoire (rien à définir) |
| Un seul serveur avec un disque persistant (un VPS, un Raspberry Pi, un volume Docker) | SQLite : un seul fichier, rien à installer, très rapide ; ou `memory://app.memoire`, le moteur en mémoire gardé dans un fichier |
| Heroku, ou plusieurs serveurs | PostgreSQL |
| Une base de données existante | celle que vous avez : MySQL/MariaDB, Redis ou MongoDB |

> [!WARNING]
> N'utilisez pas SQLite sur Heroku : le système de fichiers d'un dyno est temporaire, et le fichier est perdu à chaque déploiement et à chaque redémarrage (au moins une fois par jour). Ajoutez plutôt l'add-on Heroku Postgres (voir [Déployer sur Heroku](deploy-heroku-mobile.md#2-ajouter-une-base-de-données)).

Avec PostgreSQL, le pool ouvre jusqu'à 4 connexions par cœur de processeur quand le trafic le demande. Les petites offres hébergées limitent le nombre de connexions (souvent à 20) : pour choisir vous-même la taille du pool, ouvrez-le avec Memoire (`PgPool::connect_with(&url, 10)`) et passez-le avec `app.database(Db::from(pool))` depuis un `main` asynchrone (voir [Configuration](configuration.md#initialisation-asynchrone)).

## Pour aller plus loin

La documentation de Memoire va plus loin : [modèles](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/models/), [requêtes](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/queries/) (jointures, agrégats, group by), [relations](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/relations/), [transactions](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/transactions/), [migrations](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/migrations/), [notifications de changements](https://memoire-7f4525f1fdaf.herokuapp.com/fr/docs/events/) et une page par base de données. Tout fonctionne avec `req.db()` : elle accepte toutes les méthodes de modèle et toutes les requêtes. Dans Paquets, Memoire est réexporté sous `paquets::orm`.

## Étapes suivantes

- [API REST](rest-api.md) : une API JSON complète pour un modèle, en une ligne.
- [Tests](testing.md) : tester vos requêtes avec une base de données en mémoire.
- [Erreurs](errors.md) : ce que fait `?` des erreurs de base de données.
