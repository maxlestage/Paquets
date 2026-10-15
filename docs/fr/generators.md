# Générateurs

`paquets make:*` écrit le code que vous écririez à la main — un modèle, son API REST, une page, une migration, un test, l'authentification, un panneau d'administration — et le branche dans `src/main.rs`, comme Artisan de Laravel ou les générateurs de Rails. Le code généré est du Rust ordinaire et idiomatique : lisez-le, modifiez-le, il est à vous.

## Les générateurs

| Commande | Écrit | Branche dans `fn app()` |
|---|---|---|
| `paquets make:model Post title:string body:text` | `src/models/post.rs`, `src/models.rs` | `app.model::<models::Post>();` |
| `paquets make:api Post title:string` | le modèle, avec un test de son API | `app.api::<models::Post>("/api/posts");` |
| `paquets make:page about` | `src/pages/about.rs` (avec un test), `src/pages.rs` | `app.get("/about", pages::about::about);` |
| `paquets make:migration add_slug` | `src/migrations/m<horodatage>_add_slug.rs`, `src/migrations.rs` | `app.migrate(migrations::all());` |
| `paquets make:test checkout` | `src/feature_tests/checkout.rs` | le module, compilé par `cargo test` seulement |
| `paquets make:auth` | `Cargo.toml` (Argon2 optimisé même en debug) | `app.auth(paquets::auth::Auth::new());` |
| `paquets make:admin Post Comment` | — | `app.admin(Admin::new().model::<models::Post>()…);` |

`make:model` accepte aussi `--api` (comme `make:api`) et `--admin` (ajoute le modèle au [panneau d'administration](admin.md)). Chaque générateur déclare les modules nécessaires (`mod models;`…) et passe `rustfmt` sur le résultat : un projet créé par `paquets new` passe toujours `cargo fmt --check` et `cargo clippy -- -D warnings`, ce que la CI vérifie.

Lancez-les dans le dossier du projet. Ils n'écrasent jamais un fichier : générer deux fois le même modèle est une erreur.

## Modèles et champs

```sh
paquets make:api Post title:string body:text published:bool "views:i64?" slug:string:unique --admin
```

écrit `src/models/post.rs` :

```rust
//! The `Post` model: a table of the database (`posts`), also sent and
//! received as JSON.

use paquets::prelude::*;

/// Post.
#[model(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub views: Option<i64>,
    #[memoire(unique)]
    pub slug: String,
}
```

et un test qui crée une ligne par `POST /api/posts` puis la liste. Les champs s'écrivent `nom:type`, une clé `id: i64` est ajoutée en premier :

| Type | Rust |
|---|---|
| `string`, `text` | `String` |
| `i64`, `int`, `integer` | `i64` |
| `i32` | `i32` |
| `f64`, `float`, `number` | `f64` |
| `bool`, `boolean` | `bool` |

Un `?` après le type rend le champ facultatif (`Option<T>`, à mettre entre guillemets dans le shell : `"views:i64?"`), et `:unique` ajoute une contrainte d'unicité. Les noms passent en `snake_case` (`BlogPost` et `blog-post` donnent `blog_post`), le type en `PascalCase`, la table et le chemin de l'API au pluriel (`posts`, `categories`).

## Pages, migrations et tests

`paquets make:page pricing` écrit une page rendue sur le serveur, routée sur `/pricing`, avec un test qui la demande :

```rust
/// `GET /pricing`, rendered on the server with its SEO tags.
pub async fn pricing(_req: Request) -> Page {
    Page::new("Pricing")
        .description("Pricing.")
        .body(
            html::main()
                .class("stack")
                .child(h1().text("Pricing"))
                .child(p().text("Edit src/pages/pricing.rs.")),
        )
}
```

`paquets make:migration add_slug_to_posts` ajoute une migration versionnée (sa version est l'heure UTC) à la liste de `src/migrations.rs` ; décrivez ses changements avec le `Migration` de memoire (`.create_table::<M>()`, `.sql(…)`, `.sql_for(dialecte, …)`). Voir [Exploitation](operations.md#migrations-et-données-dexemple).

`paquets make:test checkout` écrit un test fonctionnel qui traverse toute l'application, en mémoire :

```rust
#[tokio::test]
async fn checkout() {
    let client = crate::app().test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
}
```

Tests et pages utilisent la fonction `fn app() -> App` que `paquets new` écrit dans `src/main.rs` ; sans elle, les générateurs indiquent la ligne à ajouter vous-même.

## Authentification et administration

`paquets make:auth` active l'[authentification](authentication.md) : inscription, connexion, réinitialisation du mot de passe et codes à deux facteurs. `paquets make:admin Post Comment` ajoute des modèles au [panneau d'administration](admin.md) (et le crée si besoin).

## Étapes suivantes

- [Authentification](authentication.md) : les pages et la protection des routes.
- [Panneau d'administration](admin.md) : gérer vos modèles depuis le navigateur.
- [Exploitation](operations.md) : documentation de l'API, liste des routes, migrations, santé et métriques, rechargement automatique.
