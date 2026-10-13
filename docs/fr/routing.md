# Routes

Le routage décide quel code répond à une requête, d'après sa méthode (`GET`, `POST`…) et son chemin (`/users/42`). Paquets utilise le routeur de [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/), avec le vocabulaire d'Express.js : `app.get(chemin, handler)`, des paramètres écrits `:id`, des middlewares avec `next`. Cette page couvre les routes, les paramètres, les routeurs, les middlewares, la page 404 et les fichiers statiques.

## Vos premières routes

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async { "Accueil" });
    app.get("/about", |_| async { Page::new("À propos").body(h1().text("Qui sommes-nous ?")) });
    app.post("/contact", |_| async { (201, "Message reçu") });

    app.run()
}
```

Chaque appel prend un **chemin** et un **handler**. Un handler est une fonction ou une closure `async` qui reçoit la requête et renvoie quelque chose qui devient une réponse : du texte, du JSON, une page, un statut, une redirection… (voir [Requêtes et réponses](requests-responses.md)).

## Trois façons d'écrire un handler

```rust
// 1. Une closure qui n'a pas besoin de la requête : `|_|`.
app.get("/", |_| async { "Accueil" });

// 2. Une closure qui utilise la requête : précisez son type, et écrivez `async move`.
app.get("/hello/:name", |req: Request| async move {
    format!("Bonjour {} !", req.param("name").unwrap_or("toi"))
});

// 3. Une fonction nommée, le meilleur choix dès que le code dépasse quelques lignes.
async fn list_users(_req: Request) -> Json<Vec<&'static str>> {
    Json(vec!["ada", "grace"])
}
app.get("/users", list_users);
```

`async move` donne la propriété de la requête à la future de la closure, ce qui lui permet de lire la requête après un `.await`. Si vous oubliez `move`, le compilateur signale une erreur d'emprunt.

## Méthodes HTTP

| Méthode | Correspond à |
|---|---|
| `app.get(chemin, handler)` | `GET`, et aussi `HEAD` |
| `app.post(chemin, handler)` | `POST` |
| `app.put(chemin, handler)` | `PUT` |
| `app.patch(chemin, handler)` | `PATCH` |
| `app.delete(chemin, handler)` | `DELETE` |
| `app.head(chemin, handler)` | `HEAD` |
| `app.options(chemin, handler)` | `OPTIONS` |
| `app.all(chemin, handler)` | toutes les méthodes |
| `app.route(méthode, chemin, handler)` | la `Method` passée, y compris les méthodes non standard |
| `app.ws(chemin, handler)` | une connexion WebSocket (voir [WebSocket](websocket.md)) |
| `app.api::<Model>(chemin)` | six routes REST pour un modèle (voir [API REST](rest-api.md)) |

Les méthodes de routage renvoient l'application en `&mut` : vous pouvez donc les enchaîner :

```rust
app.get("/articles", list_articles)
    .post("/articles", create_article)
    .get("/articles/:id", show_article)
    .delete("/articles/:id", delete_article);
```

## Paramètres de chemin

Un segment qui commence par `:` capture cette partie du chemin. `req.param(nom)` la renvoie sous forme d'`Option<&str>` :

```rust
app.get("/users/:id/posts/:post", |req: Request| async move {
    let user = req.param("id").unwrap_or_default();
    let post = req.param("post").unwrap_or_default();
    format!("article {post} de l'utilisateur {user}")
});
```

`GET /users/42/posts/7` répond `article 7 de l'utilisateur 42`. Les valeurs sont décodées (`/users/Fran%C3%A7ois` donne `François`), et un paramètre couvre toujours un segment entier : il ne correspond jamais à un segment vide ni à un `/`.

### Paramètres typés

`req.param_as::<T>(nom)` convertit un paramètre en nombre, en `bool` ou en n'importe quel type qui implémente `FromStr`. Quand la conversion échoue, il renvoie une erreur qui devient une `400 Bad Request` : un `?` suffit donc :

```rust
async fn show_user(req: Request) -> Result<String> {
    let id: i64 = req.param_as("id")?; // 400 si `id` n'est pas un nombre
    Ok(format!("utilisateur n°{id}"))
}

app.get("/users/:id", show_user);
```

```text
GET /users/42   → 200 utilisateur n°42
GET /users/abc  → 400 {"error":"invalid parameter 'id': 'abc'"}
```

## Jokers

Un segment `*nom` capture tout le reste du chemin, barres obliques comprises. Ce doit être le dernier segment :

```rust
app.get("/files/*path", |req: Request| async move {
    format!("vous avez demandé {}", req.param("path").unwrap_or_default())
});
```

`/files/2024/rapport.pdf` donne `path = "2024/rapport.pdf"`, et `/files` donne une chaîne vide.

## Comment une route est choisie

- **La route la plus précise l'emporte**, quel que soit l'ordre de déclaration : un segment statique bat un paramètre, qui bat un joker. Avec `/users/new` et `/users/:id`, une requête vers `/users/new` va à la première.
- **La barre oblique finale est ignorée** : `/about/` correspond à `/about`.
- **Les chemins sont sensibles à la casse** : `/About` ne correspond pas à `/about`.
- **La query string est ignorée** : `/search?q=rust` correspond à `/search` (lisez-la avec `req.query("q")`, voir [Requêtes et réponses](requests-responses.md#query-string)).

Les erreurs dans la déclaration des routes sont détectées **au démarrage de l'application**, avec un message clair : un chemin qui ne commence pas par `/`, un joker qui n'est pas en dernier, ou la même méthode déclarée deux fois sur le même chemin. Par exemple, `app.api::<Todo>("/api/todos")` suivi de votre propre `app.get("/api/todos", ...)` panique avec `duplicate route: GET /api/todos is already defined`.

Certains comportements HTTP sont fournis d'office :

- une requête `HEAD` utilise la route `GET` et reçoit les en-têtes sans le corps ;
- un chemin qui existe, mais pas pour cette méthode, reçoit `405 Method Not Allowed` avec un en-tête `Allow` qui liste les méthodes acceptées ;
- une requête `OPTIONS` reçoit `204 No Content` avec le même en-tête `Allow`.

## Routeurs : regrouper des routes

Un `Router` regroupe des routes sous un préfixe, comme `express.Router()` :

```rust
let mut admin = Router::new();
admin.get("/", |_| async { "Tableau de bord" });
admin.get("/users", |req: Request| async move {
    Ok::<_, Error>(Json(User::all(req.db()).await?))
});

app.mount("/admin", admin); // GET /admin et GET /admin/users
```

Dans un routeur, les handlers reçoivent la même requête que partout ailleurs : `req.db()`, `req.state()` et le bouclier de sécurité fonctionnent tous. Un routeur peut avoir ses propres middlewares (voir plus bas), ce qui en fait le moyen naturel de protéger une section du site.

## Middlewares

Un middleware s'exécute autour des handlers. Il reçoit la requête et `next`, la suite de la chaîne, et renvoie une réponse. Il peut agir avant le handler, après lui, ou répondre lui-même :

```rust
use std::time::Instant;

app.middleware(|req: Request, next: Next| async move {
    let start = Instant::now();
    let res = next.run(req).await; // la suite de la chaîne, puis le handler
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    res.header("server-timing", format!("app;dur={ms:.2}"))
});
```

Pour arrêter une requête, renvoyez une réponse sans appeler `next.run` :

```rust
async fn require_admin(req: Request, next: Next) -> Response {
    if req.header("x-admin-key") != Some("let-me-in") {
        return Error::forbidden("réservé aux administrateurs").into_response();
    }
    next.run(req).await
}
```

Un middleware peut se placer à trois endroits :

| Où | Code | S'exécute pour |
|---|---|---|
| Toute l'application | `app.middleware(m)` | chaque requête, 404 comprises, quel que soit l'ordre de déclaration |
| Un routeur | `router.middleware(m)` | chaque requête sous le préfixe du routeur |
| Une route | `app.get("/stats", stats.with(m))` | cette route uniquement |

```rust
let mut admin = Router::new();
admin.middleware(require_admin);
admin.get("/", |_| async { "Tableau de bord" });
app.mount("/admin", admin);

// Une seule route. `.with` vient du trait `HandlerExt`, inclus dans le prélude.
app.get("/stats", stats.with(require_admin));
```

Pour appeler `.with` sur une closure, mettez la closure entre parenthèses : `(|_| async { "ok" }).with(require_admin)`.

### L'ordre de toute la chaîne

Pour chaque requête, Paquets exécute ces couches, dans cet ordre :

1. le journal des requêtes (une ligne par requête ; `app.quiet()` le désactive) ;
2. le bouclier de sécurité (voir [Sécurité](security.md)) : une requête bloquée s'arrête ici, avant la moindre ligne de votre code ;
3. vos middlewares globaux, dans l'ordre où vous les avez ajoutés ;
4. les fichiers statiques de `public/` ;
5. le routage, puis les middlewares du routeur, ceux de la route (`.with`) et le handler.

La réponse repasse par les mêmes couches dans l'ordre inverse : le bouclier ajoute donc ses en-têtes de sécurité à chaque réponse, y compris les vôtres.

## Page 404 personnalisée

Quand aucune route ne correspond, Paquets répond `404` avec `{"error":"Cannot GET /path"}`. `app.fallback` remplace cette réponse, par exemple par une page :

```rust
app.fallback(|req: Request| async move {
    Page::new("Page introuvable")
        .status(404)
        .body(h1().text("Page introuvable"))
        .body(p().text(format!("Il n'y a rien à l'adresse {}.", req.path())))
        .body(a().href("/").text("Retour à l'accueil"))
});
```

N'oubliez pas `.status(404)` : une `Page` est envoyée avec `200 OK` par défaut. Le fallback n'est pas appelé pour une `405` (le chemin existe, avec une autre méthode). Pour changer d'un coup toutes les réponses d'erreur, voir [Erreurs](errors.md#changer-le-format-de-toutes-les-erreurs).

## Fichiers statiques

Les fichiers du dossier `public/` sont servis à la racine du site, sans une ligne de code. Pour servir un autre dossier sous un préfixe, utilisez `static_dir` :

```rust
app.static_dir("/downloads", "files"); // files/rapport.pdf → /downloads/rapport.pdf
```

Les chemins sont relatifs au dossier depuis lequel le serveur est lancé. Les fichiers statiques reçoivent les bons en-têtes `Content-Type`, `ETag` et `Last-Modified`, les réponses `304 Not Modified` et les requêtes partielles (vidéo). Les fichiers cachés (`.env`, `.git/`) et les chemins avec `..` sont refusés. Plus d'options dans [le guide des fichiers statiques de Vitesse](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/static-files/).

## Pour aller plus loin

La documentation de Vitesse approfondit le routage : [routage](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/routing/), [middlewares](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/middleware/) (y compris les middlewares fournis `logger`, `cors` et `timeout`) et [routeurs](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/routers/). Tout ce qui s'y trouve fonctionne dans Paquets : `App` donne accès à toutes les méthodes de l'application de Vitesse.

## Étapes suivantes

- [Requêtes et réponses](requests-responses.md) : lire les paramètres, la query string et le corps, et construire n'importe quelle réponse.
- [Pages](pages.md) : renvoyer des pages HTML complètes.
- [Sécurité](security.md) : ce que fait le bouclier avant que vos middlewares ne s'exécutent.
