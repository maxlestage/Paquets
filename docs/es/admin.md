# Panel de administración

`app.admin(Admin::new().model::<Todo>())` da a tus modelos un panel de administración, como el admin de Django: un tablero, listas con búsqueda, orden, filtros y páginas, y formularios para crear, editar y borrar filas. Se genera a partir de los esquemas de los modelos: no hay nada más que escribir.

## Una línea

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

O, en un proyecto, `paquets make:admin Todo` (ver [Generadores](generators.md)).

## Las páginas

| Ruta | Página |
|---|---|
| `GET /admin` | todos los modelos, con su número de filas |
| `GET /admin/<tabla>` | las filas, 25 por página |
| `GET/POST /admin/<tabla>/new` | crea una fila |
| `GET/POST /admin/<tabla>/:id` | edita una fila |
| `GET/POST /admin/<tabla>/:id/delete` | borra una fila, tras una confirmación |

La lista entiende `?q=` (busca en todas las columnas de texto), `?sort=<columna>&dir=desc`, `?<columna booleana>=true` (los filtros mostrados sobre la tabla) y `?page=`. Los formularios siguen los tipos de las columnas: casillas para los booleanos, campos numéricos para los números, un campo opcional vacío se convierte en `NULL`. Los valores inválidos y los valores únicos repetidos vuelven al formulario con un mensaje.

Algunas columnas son de solo lectura: las claves generadas, las marcas de tiempo `created_at`/`updated_at`/`deleted_at` y las columnas binarias. Las columnas cuyo nombre contiene `password`, `secret` o `token` nunca se muestran.

## Quién entra

Por defecto:

- con la [autenticación](authentication.md), los usuarios cuyo `admin` es verdadero (los visitantes van a la página de inicio de sesión, los demás usuarios reciben un `403`);
- sin ella, solo en builds de desarrollo, las peticiones de esta máquina, para que `paquets dev` abra el panel enseguida. Nunca en un build de release, nunca a través de un proxy.

Elige tú con `allow`, o con una `Gate` de Securite:

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

## Ajustes

| Método | Por defecto |
|---|---|
| `.path("/backoffice")` | `/admin` |
| `.title("Backoffice")` | `Admin` |
| `.per_page(50)` | 25 |
| `.model::<M>()` | una llamada por modelo; su tabla se crea al arrancar |

Los usuarios de la autenticación también son un modelo: `Admin::new().model::<paquets::auth::User>()` los lista y permite nombrar a un administrador.

## Próximos pasos

- [Autenticación](authentication.md): cuentas y administradores.
- [Operaciones](operations.md): documentación de la API, salud y métricas.
