# Erreurs

Dans Paquets, les erreurs sont des valeurs ordinaires. Un handler renvoie un `Result<T>`, l'opérateur `?` l'arrête à la première erreur, et l'erreur devient une réponse HTTP avec le bon statut. Cette page explique ce que fait `?` de chaque type d'erreur, comment créer les vôtres, et comment changer l'apparence des réponses d'erreur.

## Result et l'opérateur ?

`paquets::Result<T>` (dans le prélude sous le nom `Result<T>`) est un `Result<T, paquets::Error>`. Faites-le renvoyer par vos handlers, et utilisez `?` sur tout ce qui peut échouer :

```rust
async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si `id` n'est pas un nombre
    let todo = Todo::get(req.db(), id).await?; // 404 si cette ligne n'existe pas
    Ok(Json(todo))
}
```

`?` convertit chaque erreur en `paquets::Error` munie d'un statut :

| Erreur | Statut | Corps envoyé au client |
|---|---|---|
| paramètre invalide (`req.param_as`) | `400` | ``{"error":"invalid parameter 'id': 'abc'"}`` |
| query string invalide (`req.query_as`) | `400` | `{"error":"invalid query string: ..."}` |
| JSON ou formulaire invalide (`req.json`, `req.form`) | `400` | `{"error":"invalid JSON: ..."}` |
| corps trop volumineux | `413` | `{"error":"Payload Too Large"}` |
| ligne inexistante (`get`, `one`, `update`) | `404` | `{"error":"record not found"}` |
| valeur en double dans une colonne `unique`, autre contrainte, conflit de transaction | `409` | ``{"error":"unique constraint violated on `user.email`"}`` |
| `NULL` dans un champ qui n'est pas une `Option`, valeur invalide venant de la base | `400` | le message de la base de données |
| toute autre erreur (E/S, analyse, connexion perdue…) | `500` | `{"error":"Internal Server Error"}` |

Pour une `500`, les détails ne sont **jamais** envoyés au client (ils pourraient révéler le fonctionnement de votre serveur), mais ils sont écrits sur la sortie d'erreur du serveur :

```text
[vitesse] error 500: invalid digit found in string
```

Sur Heroku, cette ligne figure dans les journaux de l'application.

## Créer des erreurs

| Constructeur | Statut |
|---|---|
| `Error::bad_request(msg)` | `400 Bad Request` |
| `Error::unauthorized(msg)` | `401 Unauthorized` |
| `Error::forbidden(msg)` | `403 Forbidden` |
| `Error::not_found(msg)` | `404 Not Found` |
| `Error::conflict(msg)` | `409 Conflict` |
| `Error::unprocessable(msg)` | `422 Unprocessable Content` |
| `Error::internal(msg)` | `500 Internal Server Error` : le message est envoyé lui aussi, gardez-le donc générique |
| `Error::new(statut, msg)` | n'importe quel statut : `Error::new(418, "Je suis une théière")` |

Le message est envoyé au client sous la forme `{"error": "..."}`. Lisez les composants d'une erreur avec `err.status()` et `err.message()`.

```rust
#[data]
struct NewTodo {
    title: String,
}

async fn create(req: Request) -> Result<(StatusCode, Json<Todo>)> {
    let new: NewTodo = req.json().await?;
    if new.title.trim().is_empty() {
        return Err(Error::unprocessable("le titre ne peut pas être vide"));
    }
    let mut todo = Todo { id: 0, title: new.title, done: false };
    todo.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}
```

### À partir d'une Option

Transformez un `None` en erreur avec `ok_or_else` :

```rust
async fn show_member(req: Request) -> Result<Json<Member>> {
    let id: i64 = req.param_as("id")?;
    let member = Member::find(req.db(), id)
        .await?
        .ok_or_else(|| Error::not_found(format!("membre {id} introuvable")))?;
    Ok(Json(member))
}
```

Un handler peut aussi renvoyer directement une `Option<T>` : `None` devient `404 {"error":"Not Found"}`.

### Dans les closures

Dans une closure, Rust ne peut pas deviner le type d'erreur de `?`. Précisez-le sur le `Ok` final :

```rust
app.get("/double/:n", |req: Request| async move {
    let n: i64 = req.param_as("n")?;
    Ok::<_, Error>(format!("{}", n * 2))
});
```

## Choisir votre propre message pour une erreur de base de données

`?` donne aux erreurs de base de données un message générique. Pour en donner un meilleur, examinez l'erreur avant le `?`. Les erreurs de memoire ont `is_not_found()`, `is_unique_violation()` et `is_conflict()` :

```rust
#[model]
struct Member {
    id: i64,
    #[memoire(unique)]
    email: String,
}

#[data]
struct SignUp {
    email: String,
}

async fn sign_up(req: Request) -> Result<(StatusCode, Json<Member>)> {
    let form: SignUp = req.json().await?;
    let mut member = Member { id: 0, email: form.email };
    match member.insert(req.db()).await {
        Ok(_) => Ok((StatusCode::CREATED, Json(member))),
        Err(e) if e.is_unique_violation() => Err(Error::conflict("cet e-mail est déjà inscrit")),
        Err(e) => Err(e.into()),
    }
}
```

## Vos propres types d'erreur

Les erreurs métier méritent souvent leur propre type. Convertissez-le dans le type d'erreur de vitesse, et `?` fait le reste :

```rust
#[derive(Debug)]
enum ShopError {
    OutOfStock(i64),
    InvalidQuantity,
}

impl From<ShopError> for paquets::vitesse::Error {
    fn from(err: ShopError) -> Self {
        match err {
            ShopError::OutOfStock(id) => Self::conflict(format!("l'article {id} est en rupture de stock")),
            ShopError::InvalidQuantity => Self::unprocessable("quantité invalide"),
        }
    }
}

// Votre logique métier.
fn reserve(id: i64, quantity: i64) -> Result<(), ShopError> {
    if quantity < 1 {
        return Err(ShopError::InvalidQuantity);
    }
    Err(ShopError::OutOfStock(id))
}

async fn order(req: Request) -> Result<&'static str> {
    let id: i64 = req.param_as("id")?;
    reserve(id, 1)?; // ShopError → 409 ou 422
    Ok("réservé")
}
```

`Result<T>` utilise `paquets::Error` par défaut, et `Result<T, ShopError>` convient pour vos propres fonctions.

### Garder la cause dans les journaux

Pour envoyer un message clair au client tout en gardant l'erreur d'origine dans les journaux, attachez-la avec `with_source` :

```rust
async fn settings(_req: Request) -> Result<String> {
    let text = std::fs::read_to_string("settings.toml")
        .map_err(|e| Error::internal("réglages indisponibles").with_source(e))?;
    Ok(text)
}
```

Le client reçoit `500 {"error":"réglages indisponibles"}`, et les journaux reçoivent la cause : `[vitesse] error 500: No such file or directory (os error 2)`.

## Changer le format de toutes les erreurs

Par défaut, les erreurs sont envoyées sous la forme `{"error": "message"}`. `app.on_error` remplace ce format pour toute l'application :

```rust
app.on_error(|err: Error| {
    res::status(err.status()).json(json!({
        "error": { "status": err.status().as_u16(), "message": err.message() }
    }))
});
```

```json
{"error":{"message":"membre 9 introuvable","status":404}}
```

- La fonction reçoit une `paquets::Error` (`Error` dans le prélude) : `err.status()`, `err.message()`.
- Elle est appelée pour chaque réponse construite à partir d'une erreur : les erreurs de vos handlers, la 404 par défaut, les 405, les corps invalides, les 413, les paniques.
- Définissez vous-même le statut (`res::status(err.status())`) : sinon la page d'erreur est envoyée avec `200 OK`.
- Elle n'est pas appelée pour les requêtes bloquées par le bouclier, qui ont leur propre format (voir [Sécurité](security.md#à-quoi-ressemble-une-requête-bloquée)), ni pour les réponses que vous construisez vous-même, comme `(404, "pas ici")`.

### Des pages d'erreur pour les navigateurs

`on_error` ne voit pas la requête. Pour envoyer des pages d'erreur HTML aux navigateurs et du JSON à tous les autres, utilisez un middleware qui examine la réponse :

```rust
app.middleware(|req: Request, next: Next| async move {
    let wants_html = req.header("accept").is_some_and(|a| a.contains("text/html"));
    let mut res = next.run(req).await;
    if wants_html {
        if let Some(err) = res.take_error() {
            return Page::new("Erreur")
                .status(err.status().as_u16())
                .body(h1().text(err.message().to_string()))
                .body(a().href("/").text("Retour à l'accueil"))
                .into_response();
        }
    }
    res
});
```

Pour le seul cas de la page introuvable, `app.fallback` est plus simple : voir [Routes](routing.md#page-404-personnalisée).

## Paniques

Une panique dans un handler (un `unwrap()` sur `None`, un indice hors limites…) n'arrête pas le serveur : le client reçoit `500 {"error":"Internal Server Error"}`, le message de la panique est affiché sur la sortie d'erreur, et les autres requêtes continuent. Préférez quand même `?` et des erreurs explicites : une panique est un bug, pas une façon de répondre.

## Erreurs au démarrage

Certaines erreurs arrêtent l'application avant qu'elle ne serve quoi que ce soit, avec un message qui explique pourquoi :

| Erreur | Ce qui se passe |
|---|---|
| route invalide ou en double | une panique au moment où la route est déclarée : `duplicate route: GET /api/todos is already defined` |
| `DATABASE_URL` erronée, base de données injoignable | `run()` renvoie une erreur qui commence par `database:` |
| un hook `on_start` échoue | `run()` affiche `📦 Paquets could not start: startup: …` avec la cause, et renvoie l'erreur |
| le port est déjà pris | `run()` renvoie une erreur « address in use » |

Comme `main` renvoie un `std::io::Result<()>`, le programme affiche l'erreur et se termine avec un code différent de zéro. Sur Heroku, le dyno plante alors (`H10` dans les journaux) : lisez la ligne juste au-dessus.

## Étapes suivantes

- [Tests](testing.md) : vérifier vos erreurs dans des tests.
- [Sécurité](security.md) : les réponses du bouclier.
- [Le guide des erreurs de vitesse](https://github.com/maxlestage/Vitesse/blob/master/docs/fr/errors.md) : plus de détails sur les erreurs, `on_error` et les paniques.
