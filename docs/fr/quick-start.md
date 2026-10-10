# Démarrage rapide

Dans ce tutoriel, vous construisez **Carnet**, une petite application avec une page d'accueil, un formulaire, une base de données et une API JSON, puis vous la testez. Comptez une dizaine de minutes. Il suffit que Rust et la commande `paquets` soient installés (voir [Installation](installation.md)).

## 1. Créer le projet

```sh
paquets new carnet --minimal
cd carnet
```

`--minimal` crée un projet avec un seul fichier source, `src/main.rs`, et sans îlots : aucun outil WebAssembly n'est donc nécessaire pour ce tutoriel. Sans `--minimal`, vous obtenez aussi un `src/lib.rs` avec un compteur interactif (voir [Îlots](islands.md)). Les deux sortes de projets contiennent les mêmes fichiers de déploiement ; la page [Structure d'un projet](project-structure.md) les décrit.

## 2. Le lancer

```sh
paquets dev
```

Le premier lancement compile toutes les dépendances, ce qui prend quelques minutes. Les suivants ne recompilent que votre code. Quand le serveur est prêt, il affiche :

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: none
```

Ouvrez http://localhost:3000 dans votre navigateur : vous voyez la page de départ du projet. La deuxième ligne vous indique que la base de données est en mémoire, que le bouclier de sécurité est actif avec ses 10 guards, et que le projet n'a pas d'îlots.

Chaque requête affiche ensuite une ligne, comme `GET / 200 0.339 ms`. Arrêtez le serveur avec `Ctrl+C`.

> [!TIP]
> `paquets dev`, c'est `cargo run` plus la compilation des îlots quand le projet en a. Dans un projet minimal, `cargo run` fait exactement la même chose.

## 3. Écrire la page d'accueil

Remplacez le contenu de `src/main.rs` par ce programme :

```rust
use paquets::prelude::*;

/// La page d'accueil.
async fn home(_req: Request) -> Page {
    Page::new("Carnet")
        .description("Mes notes, en Rust.")
        .body(html::main().child(h1().text("📒 Carnet")))
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", home);
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Redémarrez le serveur (`Ctrl+C`, puis `paquets dev`) et rechargez la page. Lisons le programme en partant du bas :

- `main` construit l'application et la lance. `run()` bloque jusqu'à ce que vous appuyiez sur `Ctrl+C`.
- `app()` crée l'application et déclare ses routes. Construire l'application dans sa propre fonction permet aux tests d'utiliser exactement la même, comme vous le verrez à l'étape 8.
- `app.get("/", home)` signifie : pour une requête `GET` sur `/`, appeler `home`.
- `home` est un **handler** : une fonction `async` qui reçoit la requête et renvoie une réponse. Ici, la réponse est une `Page`, un document HTML complet avec un titre, une description pour les moteurs de recherche et les réseaux sociaux, et un thème intégré.
- Le corps est une **vue**, construite avec une fonction par balise HTML : `h1().text("📒 Carnet")` donne `<h1>📒 Carnet</h1>`.

> [!NOTE]
> Pourquoi `html::main()` et pas `main()` ? Dans ce fichier, le nom `main` est déjà pris par la fonction `main` du programme. La balise `<main>` s'écrit donc ici `html::main()`. Voir [Pages](pages.md#la-balise-main-dans-mainrs).

## 4. Ajouter une route avec un paramètre

Ajoutez cette route dans `app()`, après `app.get("/", home);` :

```rust
app.get("/hello/:name", |req: Request| async move {
    format!("Bonjour {} !", req.param("name").unwrap_or("toi"))
});
```

Redémarrez et ouvrez http://localhost:3000/hello/Ada : la page affiche `Bonjour Ada !`. Le segment `:name` du chemin est un paramètre, lu avec `req.param("name")`. Ce handler est une closure plutôt qu'une fonction nommée, et il renvoie une `String`, qui devient une réponse en texte brut. Voir [Routes](routing.md) et [Requêtes et réponses](requests-responses.md).

## 5. Enregistrer les notes dans une base de données

Ajoutez un **modèle** en haut du fichier, après la ligne `use` :

```rust
/// Une note : une table de la base de données, également envoyée et reçue en JSON.
#[model]
struct Note {
    id: i64,
    text: String,
    done: bool,
}
```

et une ligne dans `app()` :

```rust
app.api::<Note>("/api/notes");
```

Et voilà une API REST complète. Redémarrez le serveur, puis essayez-la depuis un autre terminal :

```sh
# Créer une note : 201 Created, avec l'id choisi par la base de données
curl -X POST http://localhost:3000/api/notes \
  -H 'content-type: application/json' \
  -d '{"text": "Apprendre Rust", "done": false}'

# Lister les notes
curl http://localhost:3000/api/notes

# Marquer la note 1 comme faite (PATCH ne modifie que les champs envoyés)
curl -X PATCH http://localhost:3000/api/notes/1 \
  -H 'content-type: application/json' \
  -d '{"done": true}'

# La supprimer : 204 No Content
curl -X DELETE http://localhost:3000/api/notes/1
```

Ce qui s'est passé :

- `#[model]` a transformé la struct en une table de base de données nommée `note`, avec des colonnes typées. Le champ `id` est la clé primaire, générée par la base. La struct peut aussi être convertie depuis et vers du JSON.
- Au démarrage, `app.api::<Note>(...)` a créé la table (elle n'existait pas encore) et ajouté six routes : lister, lire, créer, remplacer, modifier et supprimer. Voir [API REST](rest-api.md).
- Pour l'instant, la base de données est en mémoire : arrêtez le serveur et les notes disparaissent. L'étape 7 règle ce problème.

## 6. Afficher les notes sur la page, avec un formulaire

Remplacez `home` par une version qui lit les notes, et ajoutez un handler pour le formulaire :

```rust
/// Les données envoyées par le formulaire « nouvelle note ».
#[data]
struct NewNote {
    text: String,
}

/// La page d'accueil : le formulaire et la liste des notes.
async fn home(req: Request) -> Result<Page> {
    let notes = Note::query().order_by(Note::ID.desc()).all(req.db()).await?;
    Ok(Page::new("Carnet")
        .description("Mes notes, en Rust.")
        .body(
            html::main()
                .child(h1().text("📒 Carnet"))
                .child(
                    form()
                        .attr("method", "post")
                        .attr("action", "/notes")
                        .class("row")
                        .child(input().attr("name", "text").attr("placeholder", "Une nouvelle note"))
                        .child(button().text("Ajouter")),
                )
                .child(ul().children(notes.into_iter().map(|note| li().text(note.text)))),
        ))
}

/// Enregistre la note envoyée par le formulaire, puis revient à la page d'accueil.
async fn add_note(req: Request) -> Result<Redirect> {
    let form: NewNote = req.form().await?;
    let mut note = Note { id: 0, text: form.text, done: false };
    note.insert(req.db()).await?;
    Ok(Redirect::see_other("/"))
}
```

Enregistrez la route du formulaire dans `app()` :

```rust
app.post("/notes", add_note);
```

Redémarrez, ajoutez quelques notes avec le formulaire, et regardez-les apparaître. Quelques nouveautés :

- `req.db()` donne la base de données au handler. `Note::query()...all(req.db()).await?` lit toutes les notes, de la plus récente à la plus ancienne. Les noms de colonnes (`Note::ID`) sont vérifiés par le compilateur.
- `#[data]` marque une struct qui ne voyage que sous forme de données (un formulaire, un corps JSON), sans table.
- Les handlers renvoient maintenant un `Result<...>`. Le `?` après un appel à la base de données ou à `req.form()` arrête le handler en cas d'erreur et envoie le bon statut HTTP : `400` pour un formulaire invalide, `404` pour une ligne absente, `500` pour un problème côté serveur. Voir [Erreurs](errors.md).
- `Redirect::see_other("/")` renvoie le navigateur vers la page d'accueil une fois le formulaire envoyé : c'est le schéma classique « post, redirect, get ».
- Le formulaire n'a pas besoin de jeton CSRF : le bouclier de sécurité refuse déjà les formulaires envoyés depuis d'autres sites. Voir [Sécurité](security.md).

## 7. Conserver les données

La base de données est choisie par la variable d'environnement `DATABASE_URL`. Faites-la pointer vers un fichier SQLite :

```sh
DATABASE_URL=sqlite://carnet.db paquets dev
```

La ligne de démarrage indique maintenant `database: sqlite`. Le fichier `carnet.db` est créé à côté de `Cargo.toml`, la table `note` y est créée, et vos notes survivent aux redémarrages. Dans PowerShell sous Windows, écrivez `$env:DATABASE_URL="sqlite://carnet.db"; paquets dev`.

Envie d'une première note dans une base vide ? Ajoutez ceci dans `app()`, avant `app` :

```rust
app.on_start(|db| async move {
    if Note::count(db).await? == 0 {
        Note { id: 0, text: "Essayer Paquets".into(), done: false }.insert(db).await?;
    }
    Ok(())
});
```

`on_start` s'exécute une seule fois, quand la base de données est prête et que les tables existent, avant la première requête. En production sur Heroku, `DATABASE_URL` pointe vers PostgreSQL et le même code fonctionne sans changement. Voir [Base de données](database.md).

## 8. Le tester

Ajoutez ceci à la fin de `src/main.rs` :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_form_adds_a_note() {
        let client = app().test().await;

        let res = client.post("/notes").form(&[("text", "Écrire un test")]).await;
        assert_eq!(res.status(), 303);

        let html = client.get("/").await.text();
        assert!(html.contains("Essayer Paquets"));
        assert!(html.contains("Écrire un test"));
    }
}
```

Lancez les tests :

```sh
cargo test
```

`app().test().await` démarre votre vraie application, avec ses routes, sa sécurité et son hook `on_start`, sans ouvrir de port. La base de données est en mémoire et vide pour chaque test, quoi que dise `DATABASE_URL` : les tests ne touchent donc jamais à vos données. Voir [Tests](testing.md).

## Le programme complet

```rust
use paquets::prelude::*;

/// Une note : une table de la base de données, également envoyée et reçue en JSON.
#[model]
struct Note {
    id: i64,
    text: String,
    done: bool,
}

/// Les données envoyées par le formulaire « nouvelle note ».
#[data]
struct NewNote {
    text: String,
}

/// La page d'accueil : le formulaire et la liste des notes.
async fn home(req: Request) -> Result<Page> {
    let notes = Note::query().order_by(Note::ID.desc()).all(req.db()).await?;
    Ok(Page::new("Carnet")
        .description("Mes notes, en Rust.")
        .body(
            html::main()
                .child(h1().text("📒 Carnet"))
                .child(
                    form()
                        .attr("method", "post")
                        .attr("action", "/notes")
                        .class("row")
                        .child(input().attr("name", "text").attr("placeholder", "Une nouvelle note"))
                        .child(button().text("Ajouter")),
                )
                .child(ul().children(notes.into_iter().map(|note| li().text(note.text)))),
        ))
}

/// Enregistre la note envoyée par le formulaire, puis revient à la page d'accueil.
async fn add_note(req: Request) -> Result<Redirect> {
    let form: NewNote = req.form().await?;
    let mut note = Note { id: 0, text: form.text, done: false };
    note.insert(req.db()).await?;
    Ok(Redirect::see_other("/"))
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", home);
    app.post("/notes", add_note);
    app.get("/hello/:name", |req: Request| async move {
        format!("Bonjour {} !", req.param("name").unwrap_or("toi"))
    });
    app.api::<Note>("/api/notes");
    app.on_start(|db| async move {
        if Note::count(db).await? == 0 {
            Note { id: 0, text: "Essayer Paquets".into(), done: false }.insert(db).await?;
        }
        Ok(())
    });
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_form_adds_a_note() {
        let client = app().test().await;

        let res = client.post("/notes").form(&[("text", "Écrire un test")]).await;
        assert_eq!(res.status(), 303);

        let html = client.get("/").await.text();
        assert!(html.contains("Essayer Paquets"));
        assert!(html.contains("Écrire un test"));
    }
}
```

## 9. Le mettre en ligne

Votre projet contient déjà tout ce qu'il faut pour le déployer : un `Dockerfile`, les fichiers pour Heroku et les workflows GitHub. Poussez-le dans un dépôt GitHub, puis suivez [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md) : quelques gestes, sans ordinateur. N'oubliez pas que Heroku n'a pas d'offre gratuite (un dyno Eco coûte environ 5 $ par mois).

## Étapes suivantes

- [Structure d'un projet](project-structure.md) : chaque fichier du projet.
- [Pages](pages.md) : mises en page, thème, balises SEO.
- [Îlots](islands.md) : ajouter des parties interactives qui s'exécutent dans le navigateur.
- [Base de données](database.md) : requêtes, transactions, PostgreSQL.
