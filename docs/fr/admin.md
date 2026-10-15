# Panneau d'administration

`app.admin(Admin::new().model::<Todo>())` donne à vos modèles un panneau d'administration, comme l'admin de Django : un tableau de bord, des listes avec recherche, tri, filtres et pages, et des formulaires pour créer, modifier et supprimer des lignes. Il est généré à partir des schémas des modèles : il n'y a rien d'autre à écrire.

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
    app.admin(Admin::new().model::<Todo>());
    app.run() // http://localhost:3000/admin
}
```

Ou, dans un projet, `paquets make:admin Todo` (voir [Générateurs](generators.md)).

## Les pages

| Route | Page |
|---|---|
| `GET /admin` | tous les modèles, avec leur nombre de lignes |
| `GET /admin/<table>` | les lignes, 25 par page |
| `GET/POST /admin/<table>/new` | crée une ligne |
| `GET/POST /admin/<table>/:id` | modifie une ligne |
| `GET/POST /admin/<table>/:id/delete` | supprime une ligne, après confirmation |

La liste comprend `?q=` (cherche dans toutes les colonnes de texte), `?sort=<colonne>&dir=desc`, `?<colonne booléenne>=true` (les filtres affichés au-dessus du tableau) et `?page=`. Les formulaires suivent les types des colonnes : cases à cocher pour les booléens, champs numériques pour les nombres, un champ facultatif vide devient `NULL`. Les valeurs invalides et les doublons de valeurs uniques reviennent dans le formulaire avec un message.

Certaines colonnes sont en lecture seule : les clés générées, les horodatages `created_at`/`updated_at`/`deleted_at` et les colonnes binaires. Les colonnes dont le nom contient `password`, `secret` ou `token` ne sont jamais affichées.

## Qui entre

Par défaut :

- avec l'[authentification](authentication.md), les utilisateurs dont `admin` est vrai (les visiteurs vont à la page de connexion, les autres utilisateurs reçoivent un `403`) ;
- sans elle, dans les builds de développement seulement, les requêtes venant de cette machine, pour que `paquets dev` ouvre le panneau tout de suite. Jamais dans un build de release, jamais à travers un proxy.

Choisissez vous-même avec `allow`, ou avec une `Gate` de Securite :

```rust
use paquets::prelude::*;
use paquets::auth::User;
use paquets::securite::authz::Gate;

let by_rule = Admin::new().allow(|req: &Request| {
    req.user().is_some_and(|user| user.email.ends_with("@example.com"))
});

let gate = Gate::<User>::new().define("admin", |user| user.admin);
let by_gate = Admin::new().gate(gate, "admin");
```

## Réglages

| Méthode | Par défaut |
|---|---|
| `.path("/backoffice")` | `/admin` |
| `.title("Back-office")` | `Admin` |
| `.per_page(50)` | 25 |
| `.model::<M>()` | un appel par modèle ; sa table est créée au démarrage |

Les utilisateurs de l'authentification sont aussi un modèle : `Admin::new().model::<paquets::auth::User>()` les liste et permet de nommer un administrateur.

## Étapes suivantes

- [Authentification](authentication.md) : comptes et administrateurs.
- [Exploitation](operations.md) : documentation de l'API, santé et métriques.
