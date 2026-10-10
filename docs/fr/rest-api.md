# API REST

`app.api::<Model>(chemin)` donne à un modèle une API JSON complète en une ligne : lister, lire, créer, remplacer, modifier et supprimer, avec la pagination, les bons codes de statut et des erreurs claires. Cette page décrit chaque route, montre comment les appeler, et comment protéger ou étendre l'API.

## Une ligne

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
    app.api::<Todo>("/api/todos");
    app.run()
}
```

Cette ligne crée la table `todo` au démarrage (si elle n'existe pas encore) et ajoute six routes :

| Requête | Effet | Succès |
|---|---|---|
| `GET /api/todos` | liste les lignes, dans l'ordre des clés, 100 à la fois | `200`, tableau JSON, total dans `X-Total-Count` |
| `GET /api/todos/:id` | lit une ligne | `200`, objet JSON |
| `POST /api/todos` | crée une ligne à partir du corps JSON | `201 Created`, la ligne avec son `id` généré |
| `PUT /api/todos/:id` | remplace toute la ligne | `200`, la ligne |
| `PATCH /api/todos/:id` | ne modifie que les champs présents dans le corps | `200`, la ligne |
| `DELETE /api/todos/:id` | supprime la ligne | `204 No Content` |

## Appeler l'API

Les exemples utilisent `curl` avec un serveur qui tourne sur le port 3000.

### Créer

```sh
curl -X POST http://localhost:3000/api/todos \
  -H 'content-type: application/json' \
  -d '{"title": "Lire la doc", "done": false}'
```

```http
HTTP/1.1 201 Created
content-type: application/json

{"id":1,"title":"Lire la doc","done":false}
```

L'`id` est généré par la base de données. Si le corps contient un `id`, il est ignoré. Tous les autres champs sont obligatoires, sauf s'il s'agit d'une `Option` ou s'ils ont `#[serde(default)]` (voir [Base de données](database.md#champs-facultatifs-en-json)).

### Lister

```sh
curl -i 'http://localhost:3000/api/todos?limit=20&offset=40'
```

```http
HTTP/1.1 200 OK
content-type: application/json
x-total-count: 57

[{"id":41,"title":"...","done":false}, ...]
```

| Paramètre de la query string | Par défaut | Signification |
|---|---|---|
| `limit` | `100` | le nombre de lignes à renvoyer, entre 1 et 1000 (les valeurs plus grandes sont ramenées à 1000) |
| `offset` | `0` | le nombre de lignes à sauter |

Les lignes sont toujours triées par clé primaire. L'en-tête `X-Total-Count` contient le nombre total de lignes : un client peut ainsi calculer le nombre de pages, `ceil(total / limit)`.

### Lire une ligne

```sh
curl http://localhost:3000/api/todos/1
```

```json
{"id":1,"title":"Lire la doc","done":false}
```

### Remplacer

`PUT` envoie la ligne entière. L'`id` vient de l'URL :

```sh
curl -X PUT http://localhost:3000/api/todos/1 \
  -H 'content-type: application/json' \
  -d '{"title": "Lire toute la doc", "done": true}'
```

### Modifier certains champs

`PATCH` n'envoie que les champs à modifier. Les autres gardent leur valeur :

```sh
curl -X PATCH http://localhost:3000/api/todos/1 \
  -H 'content-type: application/json' \
  -d '{"done": true}'
```

```json
{"id":1,"title":"Lire toute la doc","done":true}
```

### Supprimer

```sh
curl -X DELETE http://localhost:3000/api/todos/1   # 204 No Content
```

## Erreurs

Chaque erreur est un objet JSON avec un message `error` :

| Situation | Statut | Corps |
|---|---|---|
| L'`id` de l'URL n'est pas valide pour le type de la clé (`/api/todos/abc`) | `400` | ``{"error":"invalid id `abc`"}`` |
| `limit` ou `offset` n'est pas un entier positif | `400` | ``{"error":"`limit` must be a positive integer"}`` |
| JSON invalide, champ manquant, mauvais type | `400` | ``{"error":"invalid JSON: missing field `done` at line 1 column 23"}`` (pour `{"title":"Lire la doc"}`) |
| `PATCH` avec un corps qui n'est pas un objet JSON | `400` | `{"error":"expected a JSON object"}` |
| Aucune ligne avec cet `id` (`GET`, `PUT`, `PATCH`, `DELETE`) | `404` | `{"error":"record not found"}` |
| Une colonne `unique` a déjà cette valeur | `409` | ``{"error":"unique constraint violated on `member.email`"}`` |
| Un problème côté serveur (base de données injoignable…) | `500` | `{"error":"Internal Server Error"}`, les détails dans les journaux |

## Clés

La clé primaire peut être de n'importe quel type qui se lit depuis une URL :

- un `id` entier (le cas habituel), généré par la base de données ;
- une `String` choisie par le client, marquée `#[memoire(primary_key)]` : `/api/tags/rust` ;
- un `uuid::Uuid` (feature `uuid`).

```rust
#[model(table = "tags")]
struct Tag {
    #[memoire(primary_key)]
    slug: String,
    label: String,
}

app.api::<Tag>("/api/tags"); // POST {"slug": "rust", "label": "Rust"}, puis GET /api/tags/rust
```

Une clé chaîne ou UUID n'est pas générée : le client doit l'envoyer dans le corps des `POST`.

## Plusieurs API

Appelez `app.api` une fois par modèle. Chacun reçoit sa propre table et ses propres routes :

```rust
app.api::<Todo>("/api/todos");
app.api::<Tag>("/api/tags");
app.api::<Member>("/api/members");
```

Vous ne pouvez pas déclarer votre propre route avec la même méthode et le même chemin qu'une route de `app.api` (par exemple votre propre `GET /api/todos`) : l'application panique au démarrage avec `duplicate route`. Les autres chemins sont libres, comme `GET /api/todos/stats`.

## Protéger l'API

Les routes de `app.api` sont publiques : n'importe qui peut créer ou supprimer des lignes. C'est très bien pour un prototype, rarement pour la production. Le bouclier bloque déjà les attaques, les inondations de requêtes et les formulaires venus d'autres sites, mais il ne sait pas qui sont vos utilisateurs. Ajoutez un middleware qui vérifie, pour chaque requête qui modifie des données, quelque chose que seuls vos clients possèdent :

```rust
async fn require_token(req: Request, next: Next) -> Response {
    let writes = req.method() != Method::GET && req.method() != Method::HEAD;
    if req.path().starts_with("/api/") && writes {
        let expected = std::env::var("API_TOKEN").unwrap_or_default();
        let sent = req.header("authorization").and_then(|h| h.strip_prefix("Bearer "));
        if expected.is_empty() || sent != Some(expected.as_str()) {
            return Error::unauthorized("jeton absent ou incorrect").into_response();
        }
    }
    next.run(req).await
}

app.middleware(require_token);
app.api::<Todo>("/api/todos");
```

La lecture reste publique, et l'écriture demande `Authorization: Bearer <jeton>`, où le jeton est la variable d'environnement `API_TOKEN` (une config var sur Heroku). Pour des comptes et des sessions, voir [Sécurité](security.md#authentification).

## Quand écrire vos propres routes

`app.api` est un CRUD générique. Il ne valide pas les valeurs au-delà de leur type, ne filtre pas les listes, ne masque pas de champs et ne vérifie pas à qui appartient une ligne. Dès que vous avez besoin de règles, écrivez les routes vous-même : quelques lignes suffisent avec l'API de la [base de données](database.md).

```rust
#[data]
struct NewTodo {
    title: String,
}

async fn create_todo(req: Request) -> Result<(StatusCode, Json<Todo>)> {
    let new: NewTodo = req.json().await?;
    let title = new.title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err(Error::unprocessable("le titre doit faire entre 1 et 200 caractères"));
    }
    let mut todo = Todo { id: 0, title: title.to_string(), done: false };
    todo.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn search_todos(req: Request) -> Result<Json<Vec<Todo>>> {
    let q = req.query("q").unwrap_or_default().to_string();
    let todos = Todo::query()
        .filter(Todo::TITLE.icontains(q))
        .order_by(Todo::ID.asc())
        .limit(50)
        .all(req.db())
        .await?;
    Ok(Json(todos))
}

app.model::<Todo>(); // la table, sans les routes génériques
app.post("/api/todos", create_todo);
app.get("/api/todos/search", search_todos);
```

## Appeler l'API depuis un navigateur

Une page de votre propre site peut appeler l'API avec `fetch` : la requête vient de la même origine, le bouclier la laisse donc passer. Un front-end hébergé sur une **autre** origine (un autre domaine, ou un serveur de développement sur un autre port) demande deux changements dans le bouclier : CORS, pour que le navigateur laisse la page lire les réponses, et une origine de confiance pour le guard CSRF, pour que ses requêtes `POST`, `PUT`, `PATCH` et `DELETE` soient acceptées :

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .guard(Cors::new().allow_origin("https://app.example.com"))
        .guard(Csrf::new().trusted_origin("https://app.example.com"))
        .build(),
);
```

Voir [Sécurité](security.md#modifier-le-bouclier).

## Tester l'API

```rust
#[tokio::test]
async fn creates_and_lists_todos() {
    let mut app = App::new();
    app.api::<Todo>("/api/todos");
    let client = app.test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Lire la doc", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    let created: Todo = res.json();
    assert_eq!(created.id, 1);

    let res = client.get("/api/todos").await;
    assert_eq!(res.header("x-total-count"), Some("1"));
    assert_eq!(res.json::<Vec<Todo>>().len(), 1);

    assert_eq!(client.get("/api/todos/99").await.status(), 404);
}
```

Voir [Tests](testing.md).

## Étapes suivantes

- [Base de données](database.md) : modèles, requêtes et transactions.
- [Sécurité](security.md) : le bouclier, l'authentification, CORS.
- [Erreurs](errors.md) : tous les statuts, et comment changer le format des erreurs.
