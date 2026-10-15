# Admin panel

`app.admin(Admin::new().model::<Todo>())` gives your models an admin panel, like Django's admin: a dashboard, lists with search, sorting, filters and pages, and forms to create, edit and delete rows. It is generated from the models' schemas: there is nothing else to write.

## One line

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

Or, in a project, `paquets make:admin Todo` (see [Generators](generators.md)).

## The pages

| Route | Page |
|---|---|
| `GET /admin` | every model, with its number of rows |
| `GET /admin/<table>` | the rows, 25 per page |
| `GET/POST /admin/<table>/new` | creates a row |
| `GET/POST /admin/<table>/:id` | edits a row |
| `GET/POST /admin/<table>/:id/delete` | deletes a row, after a confirmation |

The list understands `?q=` (searches every text column), `?sort=<column>&dir=desc`, `?<bool column>=true` (the filters shown above the table) and `?page=`. The forms follow the column types: checkboxes for booleans, number fields for numbers, an empty optional field becomes `NULL`. Invalid values and duplicate unique values come back in the form with a message.

Some columns are read-only: generated keys, `created_at`/`updated_at`/`deleted_at` timestamps, and binary columns. Columns whose name contains `password`, `secret` or `token` are never shown.

## Who gets in

By default:

- with [authentication](authentication.md), the users whose `admin` is true (visitors are sent to the login page, other users get a `403`);
- without it, in development builds only, requests from this machine, so that `paquets dev` opens the panel right away. Never in release builds, never through a proxy.

Choose yourself with `allow`, or with a Securite `Gate`:

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

## Settings

| Method | Default |
|---|---|
| `.path("/backoffice")` | `/admin` |
| `.title("Back office")` | `Admin` |
| `.per_page(50)` | 25 |
| `.model::<M>()` | one call per model; its table is created at startup |

The users of authentication are a model too: `Admin::new().model::<paquets::auth::User>()` lists them and lets you make someone an administrator.

## Next steps

- [Authentication](authentication.md): accounts and administrators.
- [Operations](operations.md): API documentation, health checks and metrics.
