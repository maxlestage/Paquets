# Requêtes et réponses

Chaque handler reçoit une `Request` et renvoie une réponse. Cette page montre comment lire tout ce que transporte une requête (paramètres, query string, en-têtes, cookies, corps JSON et formulaires) et tous les types de réponses que vous pouvez renvoyer (texte, JSON, pages, statuts, redirections, cookies, fichiers).

## La requête

Un handler reçoit la requête par valeur : `|req: Request| async move { ... }` ou `async fn handler(req: Request)`. Presque toutes les méthodes prennent `&self` : vous pouvez donc lire un paramètre, puis le corps, puis un en-tête, dans l'ordre que vous voulez.

| Méthode | Renvoie | Exemple |
|---|---|---|
| `req.method()` | `&Method` | `GET` |
| `req.path()` | `&str` | `/users/42` |
| `req.param("id")` | `Option<&str>` | le segment `:id` de la route |
| `req.param_as::<T>("id")` | `Result<T, _>` | un paramètre typé, `400` s'il est invalide |
| `req.query("q")` | `Option<Cow<str>>` | `?q=rust` |
| `req.query_as::<T>()` | `Result<T, _>` | toute la query string dans une struct, `400` si elle est invalide |
| `req.header("user-agent")` | `Option<&str>` | un en-tête (le nom est insensible à la casse) |
| `req.cookie("session")` | `Option<&str>` | un cookie |
| `req.json::<T>().await` | `Result<T, _>` | le corps en JSON, `400` s'il est invalide |
| `req.form::<T>().await` | `Result<T, _>` | un formulaire HTML, `400` s'il est invalide |
| `req.text().await` | `Result<String, _>` | le corps en texte |
| `req.bytes().await` | `Result<Bytes, _>` | le corps brut |
| `req.db()` | `&'static Db` | la base de données (voir [Base de données](database.md)) |
| `req.state::<T>()` | `&T` | l'état partagé (voir [Configuration](configuration.md#état-partagé)) |

Chaque méthode qui peut échouer renvoie une erreur que `?` transforme en bon statut : un handler ne décrit donc que le chemin du succès.

## Query string

Pour une ou deux valeurs, `req.query(nom)` suffit. Il renvoie la valeur décodée, ou `None` :

```rust
app.get("/search", |req: Request| async move {
    let q = req.query("q").unwrap_or_default();
    let page: u32 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    format!("recherche de {q:?}, page {page}")
});
```

`GET /search?q=caf%C3%A9&page=2` répond `recherche de "café", page 2`.

Pour davantage de valeurs, décrivez la query string avec une struct `#[data]` et lisez-la en un seul appel avec `req.query_as()`. Utilisez `Option` pour les valeurs facultatives et `#[serde(default)]` pour les valeurs qui ont une valeur par défaut :

```rust
#[data]
struct Search {
    q: String,
    page: Option<u32>,
    #[serde(default)]
    exact: bool,
}

async fn search(req: Request) -> Result<String> {
    let s: Search = req.query_as()?; // 400 si `q` manque ou si `page` n'est pas un nombre
    Ok(format!("{} (page {}, exact : {})", s.q, s.page.unwrap_or(1), s.exact))
}
```

`#[data]` dérive `Clone`, `Debug`, `Serialize` et `Deserialize` pour la struct, avec la crate `serde` réexportée par Paquets : aucune autre dépendance n'est nécessaire.

## En-têtes et cookies

```rust
app.get("/whoami", |req: Request| async move {
    let agent = req.header("user-agent").unwrap_or("inconnu");
    let visits = req.cookie("visits").unwrap_or("0");
    format!("Vous utilisez {agent} et êtes déjà venu {visits} fois.")
});
```

- `req.header(nom)` renvoie la première valeur d'un en-tête, sous forme de texte. `req.headers()` les renvoie toutes.
- `req.cookie(nom)` lit un cookie envoyé par le navigateur, tel quel : il n'est ni décodé ni signé.

## L'adresse IP du client

Le bouclier de sécurité trouve la véritable adresse du client, même derrière le routeur de Heroku, et l'attache à la requête. Lisez-la avec :

```rust
use paquets::securite::ClientIp;

async fn ip(req: Request) -> String {
    match req.extensions().get::<ClientIp>() {
        Some(ClientIp(ip)) => format!("Votre IP : {ip}"),
        None => "inconnue".to_string(),
    }
}
```

`req.ip()` existe aussi, mais renvoie l'adresse de la connexion directe : derrière un proxy ou le routeur de Heroku, c'est l'adresse du proxy. Préférez `ClientIp`. Voir [Sécurité](security.md#lip-du-client-derrière-un-proxy).

## Corps JSON

Décrivez le corps attendu avec une struct `#[data]` (ou utilisez un `#[model]`), et lisez-le avec `req.json()` :

```rust
#[data]
struct NewUser {
    name: String,
    email: String,
    newsletter: Option<bool>,
}

async fn create_user(req: Request) -> Result<(StatusCode, Json<paquets::serde_json::Value>)> {
    let user: NewUser = req.json().await?;
    if !user.email.contains('@') {
        return Err(Error::unprocessable("l'adresse e-mail n'est pas valide"));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "name": user.name, "newsletter": user.newsletter.unwrap_or(false) })),
    ))
}
```

Si le corps n'est pas du JSON valide, ou si un champ manque ou n'a pas le bon type, `req.json()` échoue et le client reçoit une `400` qui explique pourquoi. Envoyer `{"name":"Ada"}` à ce handler donne :

```json
{"error":"invalid JSON: missing field `email` at line 1 column 14"}
```

Les champs inconnus sont ignorés. Pour du JSON sans forme fixe, demandez une `serde_json::Value` (réexportée sous `paquets::serde_json`, et `json!` est dans le prélude).

## Formulaires

Les formulaires HTML (`application/x-www-form-urlencoded`) fonctionnent de la même façon, avec `req.form()` :

```rust
#[data]
struct Login {
    email: String,
    password: String,
    // Une case non cochée n'est pas envoyée du tout : `false` par défaut.
    #[serde(default)]
    remember: bool,
}

async fn login(req: Request) -> Result<Redirect> {
    let form: Login = req.form().await?;
    if form.email.is_empty() || form.password.is_empty() {
        return Err(Error::bad_request("l'e-mail et le mot de passe sont obligatoires"));
    }
    // ... vérifier le mot de passe, ouvrir une session ...
    Ok(Redirect::see_other("/dashboard"))
}
```

### Envois de fichiers

Avec la feature `multipart`, `req.multipart()?` lit un formulaire envoyé avec `enctype="multipart/form-data"`, un champ à la fois ; il répond `415` quand la requête est d'un autre type. Les limites de corps (2 Mio par défaut) s'appliquent aussi aux envois : pour accepter des fichiers plus gros, relevez les deux (voir [Taille du corps](#taille-du-corps)). Voir [le guide des envois de fichiers de Vitesse](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/requests/#envois-de-fichiers-multipart).

Sans la feature, lisez le corps brut avec `req.take_body()` et passez-le à une crate multipart.

## Taille du corps

Deux limites protègent le serveur contre les corps énormes :

| Limite | Par défaut | Vérifiée par | Réponse |
|---|---|---|---|
| La limite de corps du bouclier | 2 Mio | l'en-tête `Content-Length`, avant que votre code ne s'exécute | `413` |
| La limite de lecture | 2 Mio | `req.json()`, `req.form()`, `req.text()` et `req.bytes()` | `413 {"error":"Payload Too Large"}` |

Par défaut, un handler lit donc au plus 2 Mio. Pour accepter des corps plus gros, relevez les deux limites :

```rust
use paquets::securite::prelude::*;

let mut app = App::new();
app.body_limit(10 * 1024 * 1024); // lire jusqu'à 10 Mio
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

`paquets::shield()` est le bouclier par défaut, prêt à être modifié : voir [Sécurité](security.md#modifier-le-bouclier).

## Réponses

Un handler n'écrit pas dans un objet réponse : il **renvoie** sa réponse. Tout ce qui implémente `IntoResponse` convient :

| Valeur renvoyée | Statut | Content-Type |
|---|---|---|
| `&'static str`, `String` | `200` | `text/plain; charset=utf-8` |
| `Json(valeur)` (n'importe quelle valeur `Serialize`) | `200` | `application/json` |
| `json!({ ... })` | `200` | `application/json` |
| `Page` | `200`, ou celui défini avec `.status(...)` | `text/html; charset=utf-8` |
| `Html(corps)` | `200` | `text/html; charset=utf-8` |
| `StatusCode::NO_CONTENT` | ce statut | le libellé du statut, en texte |
| `(statut, corps)` | `statut` | celui de `corps` |
| `Redirect::to("/")` | `302` | aucun, avec un en-tête `Location` |
| `Option<T>` | celui de `T`, ou `404` pour `None` | |
| `Result<T>` | celui de `T`, ou le statut de l'erreur | |
| `Response` | ce que vous avez construit | |

```rust
app.get("/text", |_| async { "Bonjour !" });
app.get("/user", |_| async { Json(json!({ "id": 1, "name": "Ada" })) });
app.get("/stats", |_| async { json!({ "users": 42, "online": true }) });
app.post("/jobs", |_| async { (StatusCode::ACCEPTED, "en file d'attente") });
app.post("/users", |_| async { (201, Json(json!({ "id": 2 }))) });
app.delete("/cache", |_| async { StatusCode::NO_CONTENT });
app.get("/old-page", |_| async { Redirect::permanent("/new-page") });
```

### Page ou Html ?

Renvoyez une [`Page`](pages.md) pour du HTML : c'est un document complet avec les balises SEO, le thème, et une Content-Security-Policy qui autorise les styles et les scripts propres à la page. `Html(...)` envoie une chaîne HTML telle quelle, avec la politique par défaut du bouclier, qui bloque les balises `<style>` et `<script>` en ligne qu'elle ne connaît pas. N'utilisez `Html` que pour des fragments, ou pour du HTML qui charge ses ressources depuis des fichiers.

### Redirections

| Helper | Statut | À utiliser |
|---|---|---|
| `Redirect::to(url)` | `302 Found` | pour une redirection temporaire |
| `Redirect::see_other(url)` | `303 See Other` | après le `POST` d'un formulaire |
| `Redirect::temporary(url)` | `307 Temporary Redirect` | pour conserver la méthode et le corps |
| `Redirect::permanent(url)` | `301 Moved Permanently` | pour une page qui a déménagé pour de bon |

### En-têtes et cookies : le builder res

Quand vous devez régler plusieurs choses à la fois, construisez une `Response`. Le module `res` fournit des points d'entrée dans le style d'Express, et toutes les méthodes s'enchaînent :

```rust
app.get("/custom", |_| async {
    res::status(202)
        .header("x-version", "1.0")
        .cookie(Cookie::new("seen", "1"))
        .json(json!({ "ok": true }))
});

app.post("/logout", |_| async { res::redirect("/").clear_cookie("session") });
```

| Méthode | Effet |
|---|---|
| `res::status(code)` / `.status(code)` | le statut |
| `res::json(v)` / `.json(v)`, `.text(s)`, `.html(s)` | le corps, avec son `Content-Type` |
| `.header(nom, valeur)` | définit un en-tête |
| `.append_header(nom, valeur)` | ajoute une valeur de plus à un en-tête |
| `.cookie(Cookie::new(nom, valeur))` | définit un cookie (`.max_age(...)`, `.path(...)`…) |
| `.clear_cookie(nom)` | supprime un cookie |
| `res::redirect(url)` | une redirection `302` |

Le guard des cookies du bouclier ajoute `Secure`, `HttpOnly` et `SameSite=Lax` à chaque cookie qui ne les définit pas : les cookies sont donc sûrs, même en cas d'oubli. Les navigateurs acceptent les cookies `Secure` sur `http://localhost` pendant le développement.

### Fichiers

```rust
app.get("/terms", |_| async { res::file("legal/terms.pdf").await });
app.get("/invoice", |_| async { res::download("files/invoice-42.pdf", "invoice.pdf").await });
```

`res::file` devine le `Content-Type` d'après l'extension, envoie les gros fichiers en streaming et répond `404` si le fichier n'existe pas. `res::download` demande en plus au navigateur d'enregistrer le fichier sous le nom donné. Pour un dossier entier, utilisez `public/` ou `app.static_dir` (voir [Routes](routing.md#fichiers-statiques)).

### Vos propres types de réponse

Implémentez `IntoResponse` pour renvoyer directement vos propres types :

```rust
struct Csv(String);

impl IntoResponse for Csv {
    fn into_response(self) -> Response {
        res::send(self.0).content_type("text/csv; charset=utf-8")
    }
}

app.get("/export.csv", |_| async { Csv("id,name\n1,Ada\n".into()) });
```

## Pour aller plus loin

- [Vitesse : Lire la requête](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/requests/) et [Répondre](https://vitesse-f16135654c3d.herokuapp.com/fr/docs/responses/) : toutes les méthodes, les corps en streaming, les Server-Sent Events.
- [Erreurs](errors.md) : ce que fait `?` de chaque type d'erreur.

## Étapes suivantes

- [Pages](pages.md) : des pages HTML avec des balises SEO et un thème.
- [Base de données](database.md) : lire et écrire des données depuis vos handlers.
