# Tests

`app.test().await` transforme votre application en client de test : les requêtes vont directement à l'application, en passant par le bouclier, les middlewares et vos handlers, sans ouvrir de port. La base de données est en mémoire et vide pour chaque test : les tests sont donc rapides, indépendants, et peuvent s'exécuter en parallèle. Cette page montre comment les écrire.

## Mise en place

Les tests sont des fonctions `async` marquées `#[tokio::test]`. Ils ont besoin de tokio comme dépendance de développement, ce que les projets créés avec `paquets new` ont déjà :

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Votre application elle-même n'a pas besoin de tokio : `App::run()` gère son propre runtime.

## Rendre l'application testable

Les tests doivent construire la même application que `main`, sans démarrer le serveur. Placez l'application dans une fonction :

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Accueil").body(h1().text("Bonjour")) });
    app.api::<Todo>("/api/todos");
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

## Votre premier test

Ajoutez un module `tests` à la fin de `src/main.rs` :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_page() {
        let client = app().test().await;

        let res = client.get("/").await;

        assert_eq!(res.status(), 200);
        assert!(res.text().contains("<h1>Bonjour</h1>"));
    }
}
```

Lancez-le :

```sh
cargo test                 # tous les tests
cargo test home_page       # les tests dont le nom contient « home_page »
cargo test -- --nocapture  # affiche aussi ce que les tests impriment
```

`app().test().await` prépare l'application exactement comme le ferait `run()` : il ouvre la base de données, crée les tables, exécute les hooks `on_start`, et branche le bouclier et les fichiers statiques. Seuls le journal des requêtes et le réseau sont laissés de côté.

## Envoyer des requêtes

Commencez une requête depuis le client, enchaînez ce dont vous avez besoin, et envoyez-la avec `.await` :

| Méthode | Requête |
|---|---|
| `client.get(uri)` | `GET` |
| `client.post(uri)` | `POST` |
| `client.put(uri)` | `PUT` |
| `client.patch(uri)` | `PATCH` |
| `client.delete(uri)` | `DELETE` |
| `client.request(Method::OPTIONS, uri)` | n'importe quelle autre méthode |

| Ensuite | Effet |
|---|---|
| `.header(nom, valeur)` | ajoute un en-tête |
| `.json(&valeur)` | un corps JSON, avec `content-type: application/json` |
| `.form(&valeur)` | un corps de formulaire, comme `&[("email", "ada@example.com")]` |
| `.body(données)` | un corps brut |

```rust
let res = client
    .post("/api/todos")
    .json(&json!({ "title": "Écrire des tests", "done": false }))
    .await;

let res = client.get("/search?q=caf%C3%A9").await; // la query string fait partie de l'URI, encodée
let res = client.post("/login").form(&[("email", "ada@example.com"), ("password", "secret")]).await;
```

## Lire les réponses

| Méthode | Renvoie |
|---|---|
| `res.status()` | le statut, comparable à un nombre : `assert_eq!(res.status(), 404)` |
| `res.header(nom)` | un en-tête, sous forme d'`Option<&str>` |
| `res.headers()` | tous les en-têtes |
| `res.text()` | le corps sous forme de `String` |
| `res.json::<T>()` | le corps décodé depuis le JSON (panique en affichant le corps s'il n'est pas valide) |
| `res.bytes()` | le corps brut |

```rust
#[tokio::test]
async fn creates_a_todo() {
    let client = app().test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Écrire des tests", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    let todo: Todo = res.json(); // typé
    assert_eq!(todo.title, "Écrire des tests");

    let body: paquets::serde_json::Value = client.get("/api/todos/1").await.json(); // non typé
    assert_eq!(body["done"], false);
}
```

## La base de données dans les tests

Chaque appel à `app().test()` reçoit une nouvelle base de données en mémoire, vide, quoi que dise `DATABASE_URL`. Les tests ne voient jamais les données des autres tests, ni les vôtres.

Pour partir de données connues, ajoutez un hook `on_start` dans le test :

```rust
#[tokio::test]
async fn lists_are_paginated() {
    let mut app = app();
    app.on_start(|db| async move {
        for title in ["Un", "Deux", "Trois"] {
            Todo { id: 0, title: title.into(), done: false }.insert(db).await?;
        }
        Ok(())
    });
    let client = app.test().await;

    let res = client.get("/api/todos?limit=2").await;
    assert_eq!(res.header("x-total-count"), Some("3"));
    assert_eq!(res.json::<Vec<Todo>>().len(), 2);
}
```

Pour tester avec un vrai moteur SQL (pour du SQL brut ou des migrations propres au SQL), choisissez-le dans le test. Une base SQLite en mémoire est, elle aussi, créée à neuf pour chaque test :

```rust
let mut app = app();
app.database("sqlite::memory:");
let client = app.test().await;
```

## Tester les pages

Une page, c'est du HTML : vérifiez le statut, le type de contenu et le texte attendu.

```rust
#[tokio::test]
async fn the_home_page_has_its_seo_tags() {
    let client = app().test().await;
    let res = client.get("/").await;

    assert_eq!(res.status(), 200);
    assert!(res.header("content-type").unwrap().starts_with("text/html"));
    let html = res.text();
    assert!(html.contains("<title>Accueil</title>"));
    assert!(html.contains("<h1>Bonjour</h1>"));
}
```

## Tester la sécurité

Le bouclier fonctionne aussi dans les tests : vous pouvez donc vérifier que les attaques sont refusées :

```rust
#[tokio::test]
async fn attacks_are_blocked() {
    let client = app().test().await;

    // Une injection SQL dans la query string : bloquée par le pare-feu.
    let res = client.get("/?q=1%27%20OR%20%271%27=%271").await;
    assert_eq!(res.status(), 403);

    // Un outil d'attaque connu : bloqué par le blocage des robots.
    let res = client.get("/").header("user-agent", "sqlmap/1.7").await;
    assert_eq!(res.status(), 403);

    // Une requête envoyée depuis un autre site : bloquée par le guard CSRF.
    let res = client
        .post("/api/todos")
        .header("sec-fetch-site", "cross-site")
        .json(&json!({ "title": "x", "done": false }))
        .await;
    assert_eq!(res.status(), 403);
}
```

> [!WARNING]
> La limitation de débit fonctionne aussi dans les tests : toutes les requêtes d'un test viennent du même client, donc après 100 requêtes rapprochées, les suivantes reçoivent `429 Too Many Requests`. Pour un test qui envoie beaucoup de requêtes, retirez la limitation de débit dans ce test :
>
> ```rust
> let mut app = app();
> app.security(paquets::shield().without("rate_limit").build());
> let client = app.test().await;
> ```

## Tester les erreurs et les middlewares

Les erreurs sont des réponses ordinaires : vérifiez le statut et le message.

```rust
#[tokio::test]
async fn bad_input_is_refused() {
    let client = app().test().await;

    let res = client.post("/api/todos").body("{oops").await; // pas du JSON
    assert_eq!(res.status(), 400);

    assert_eq!(client.get("/api/todos/abc").await.status(), 400); // pas un nombre
    assert_eq!(client.get("/api/todos/999").await.status(), 404); // ligne inexistante
}
```

Pour un middleware, vérifiez les deux chemins : la requête qu'il laisse passer et celle qu'il arrête (par exemple, avec et sans l'en-tête `Authorization`).

## Les tests du dossier tests

Le module `tests` de `src/main.rs` est l'endroit le plus simple pour les tests. Si vous préférez des tests d'intégration dans un dossier `tests/`, sachez qu'ils ne peuvent utiliser que votre bibliothèque (`src/lib.rs`), pas `src/main.rs`. Déplacez `app()` dans la bibliothèque, en la tenant à l'écart de la compilation pour le navigateur quand le projet a des îlots :

```rust
// src/lib.rs
#[cfg(not(target_arch = "wasm32"))]
pub fn app() -> paquets::App {
    let mut app = paquets::App::new();
    // ... les routes ...
    app
}
```

```rust
// src/main.rs
fn main() -> std::io::Result<()> {
    mon_app::app().run()
}
```

```rust
// tests/api.rs
use paquets::prelude::*;

#[tokio::test]
async fn home_page() {
    let client = mon_app::app().test().await;
    assert_eq!(client.get("/").await.status(), 200);
}
```

Les modèles utilisés par `app()` passent alors eux aussi dans la bibliothèque, derrière le même `#[cfg(not(target_arch = "wasm32"))]`.

## Ce que le client de test ne couvre pas

- **Le réseau** : les en-têtes écrits par le moteur HTTP (`content-length`, `date`) ne figurent pas dans les réponses de test.
- **Les conversations WebSocket** : le client de test ne voit que la poignée de main. Voir [WebSocket](websocket.md#tests) pour tester sur un vrai port.
- **Le côté navigateur des îlots** : testez le HTML serveur d'un îlot avec le client de test, et sa logique en appelant directement vos fonctions Rust.

## En intégration continue

Le workflow `ci.yml` d'un projet généré lance `cargo test` à chaque push, avec `cargo fmt --check` et Clippy, et ne fusionne la branche dans `master` que quand tout est vert. Voir [Automatisation](automation.md).

## Étapes suivantes

- [Erreurs](errors.md) : à quoi ressemble chaque erreur, pour la tester.
- [Automatisation](automation.md) : les tests à chaque push.
- [Le guide des tests de Vitesse](https://maxlestage.github.io/Vitesse/fr/docs/testing/) : d'autres exemples avec le même client de test.
