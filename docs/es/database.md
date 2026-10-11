# Base de datos

Paquets guarda los datos con [memoire-orm](https://maxlestage.github.io/Memoire/), un ORM con tipado seguro: un struct de Rust describe una tabla, y las consultas se escriben en Rust y las comprueba el compilador. El mismo código funciona en memoria, en SQLite, PostgreSQL, MySQL/MariaDB, Redis o MongoDB: la base de datos se elige con una URL. Esta página cubre los modelos, la elección de la base de datos, la lectura y la escritura, las transacciones, los cambios de esquema y el SQL sin procesar.

## Modelos

Un modelo es un struct marcado con `#[model]`:

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,      // la clave primaria, generada por la base de datos
    title: String,
    done: bool,
}
```

`#[model]` convierte el struct en una tabla de la base de datos **y** en un valor JSON. Añade `#[derive(Clone, Debug, Serialize, Deserialize, Model)]` (los derives que ya hayas escrito se conservan) y hace que el código generado apunte a los crates que reexporta Paquets, así que no necesitas ninguna otra dependencia.

| Regla | Detalles |
|---|---|
| Nombre de la tabla | el nombre del struct en `snake_case`, sin pluralizar: `Todo` → `todo`, `BlogPost` → `blog_post`. Elige otro nombre con `#[model(table = "todos")]`. |
| Clave primaria | el campo llamado `id`. Un `id` entero (`i64`, `i32` o `i16`) lo genera la base de datos al insertar una fila. |
| `id` en JSON | un `id` entero se puede omitir en los cuerpos JSON: los clientes que crean una fila todavía no lo conocen. |
| Columnas | una por campo, con el mismo nombre. `Option<T>` hace que una columna admita valores nulos; todas las demás columnas son `NOT NULL`. |
| Columnas tipadas | cada campo obtiene una constante en `UPPER_SNAKE_CASE`, que se usa en las consultas: `Todo::TITLE`, `Todo::DONE`. |

### Tipos de columna

| Tipo de Rust | Notas |
|---|---|
| `bool` | |
| `i16`, `i32`, `i64` (y `i8`, `u8`, `u16`, `u32`) | `u64`, `usize` e `i128` no son compatibles: usa `i64` |
| `f32`, `f64` | |
| `String` | |
| `Vec<u8>` | datos binarios |
| `Option<T>` | cualquiera de los anteriores, admitiendo nulos |
| `chrono::NaiveDateTime`, `DateTime<Utc>`, `NaiveDate`, `NaiveTime` | feature `chrono` |
| `uuid::Uuid` | feature `uuid` |
| `serde_json::Value`, `Json<T>` | feature `json` |

### Atributos de columna

Los atributos de Memoire funcionan en los campos, dentro de `#[memoire(...)]`:

```rust
#[model(table = "users")]
struct User {
    id: i64,
    #[memoire(unique)]
    email: String,
    #[memoire(index)]
    name: String,
    #[memoire(default = "0")]
    visits: i32,
    bio: Option<String>,
}
```

| Atributo | Efecto |
|---|---|
| `#[memoire(unique)]` | una segunda fila con el mismo valor se rechaza: `409 Conflict` en un handler |
| `#[memoire(index)]` | un índice, para las columnas por las que filtras u ordenas a menudo |
| `#[memoire(column = "e_mail")]` | otro nombre para la columna |
| `#[memoire(default = "0")]` | un valor por defecto SQL, que se usa cuando añades la columna a una tabla existente |
| `#[memoire(references = "users(id)", on_delete = "cascade")]` | una clave foránea |
| `#[memoire(primary_key)]` | otro campo como clave primaria, por ejemplo un `slug: String` |
| `#[memoire(fulltext)]` | prepara una columna de texto para la [búsqueda de texto completo](#consultas) con `matches`: un índice GIN en PostgreSQL, un índice de palabras en memoria |
| `#[memoire(skip)]` | un campo que no se guarda (su tipo debe implementar `Default`) |

En el struct, `#[memoire(unique(a, b))]` y `#[memoire(index(a, b))]` abarcan varias columnas a la vez, y se pueden repetir. `unique(user_id, team_id)` rechaza una segunda fila con el mismo par (`409 Conflict`), mientras que cada columna por separado puede repetirse:

```rust
#[model(table = "memberships")]
#[memoire(unique(user_id, team_id), index(team_id, role))]
struct Membership {
    id: i64,
    user_id: i64,
    team_id: i64,
    role: String,
}
```

### Campos opcionales en JSON

Un campo `bool` o `String` es obligatorio en los cuerpos JSON. Para que los clientes puedan omitirlo, añade `#[serde(default)]` (el campo recibe entonces `false`, `0` o una cadena vacía), o conviértelo en un `Option`:

```rust
#[model]
struct Task {
    id: i64,
    title: String,
    #[serde(default)]
    done: bool,          // se puede omitir: false
    due: Option<String>, // se puede omitir o ser null: NULL en la base de datos
}
```

## Elegir la base de datos

La base de datos se elige con la variable de entorno `DATABASE_URL` al arrancar la aplicación:

| `DATABASE_URL` | Base de datos | Feature de Cargo |
|---|---|---|
| *(sin definir)* o `memory` | en memoria, se vacía cuando el servidor se detiene | siempre disponible |
| `memory://data/app.memoire` | en memoria, guardada en un archivo y leída de nuevo al arrancar | siempre disponible |
| `sqlite://app.db`, `app.db`, `sqlite::memory:` | un archivo SQLite, que se crea si no existe | `sqlite` (por defecto) |
| `postgres://user:password@host:5432/name` | PostgreSQL, con TLS cuando el servidor lo ofrece | `postgres` (por defecto) |
| `mysql://…`, `mariadb://…` | MySQL o MariaDB | `mysql` |
| `redis://…`, `rediss://…` | Redis | `redis` |
| `mongodb://…`, `mongodb+srv://…` | MongoDB | `mongodb` |

```sh
DATABASE_URL=sqlite://app.db paquets dev
```

La línea de arranque te dice cuál se usa: `database: sqlite`. En Heroku, añadir el add-on Heroku Postgres define `DATABASE_URL` por ti: no hay nada que cambiar en el código.

También puedes elegir la base de datos en el código. En ese caso tiene prioridad sobre `DATABASE_URL`:

```rust
let mut app = App::new();
app.database("sqlite://app.db"); // un archivo junto al lugar donde se ejecuta el programa
// o bien: app.database(Db::memory());
```

Si la URL es incorrecta o no se puede llegar al servidor, la aplicación no arranca y dice por qué. Una URL que necesita una feature que no has activado incluso te dice qué escribir en `Cargo.toml`:

```text
📦 Paquets could not start: database: operation not supported by this backend: `mysql` URLs need the `mysql` feature: paquets = { git = "https://github.com/maxlestage/Paquets", features = ["mysql"] }
```

> [!NOTE]
> Las pruebas creadas con `app.test()` siempre usan una base de datos en memoria nueva, salvo que la propia prueba llame a `app.database(...)`. Ahí `DATABASE_URL` se ignora, así que las pruebas nunca tocan tus datos.

### Una base de datos en memoria guardada en un archivo

`DATABASE_URL=memory://data/app.memoire` conserva la velocidad del motor en memoria (datos, índices y transacciones en memoria) y además escribe cada cambio confirmado en el archivo, que se vuelve a leer cuando la aplicación arranca de nuevo. El archivo (y su carpeta) se crea si no existe y se compacta de vez en cuando. Las tablas vuelven cuando se usa su modelo, y sus columnas se reconocen por el nombre.

Con la feature `encryption`, el archivo se puede cifrar con una contraseña (AES-256-GCM, una clave derivada con Argon2id). Ábrelo tú mismo y pásalo a la aplicación:

```rust
use paquets::orm::{MemoryDb, StorageOptions};

let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD");
let db = MemoryDb::open_with("data/app.memoire", StorageOptions::new().password(password))
    .map_err(std::io::Error::other)?;
app.database(Db::from(db));
```

Como SQLite, necesita un disco persistente: no en Heroku. Consulta [el motor en memoria de Memoire](https://maxlestage.github.io/Memoire/es/docs/memory/) para la durabilidad (`StorageOptions::sync`) y la compactación.

## Las tablas se crean al arrancar

Cuando la aplicación arranca, crea la tabla de cada modelo registrado con `app.api::<M>(...)` (consulta [API REST](rest-api.md)) o `app.model::<M>()`, **si la tabla todavía no existe**. Usa `app.model` para los modelos que solo usas en tus propios handlers:

```rust
let mut app = App::new();
app.api::<Todo>("/api/todos"); // tabla + rutas REST
app.model::<User>();            // solo la tabla
```

Una tabla existente nunca se modifica: cuando más adelante añadas un campo a un modelo, consulta [Modificar una tabla más adelante](#modificar-una-tabla-más-adelante).

## Leer datos

En un handler, `req.db()` devuelve la base de datos. Todos los métodos de los modelos la reciben como primer argumento:

```rust
async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?;          // 400 si no es un número
    let todo = Todo::get(req.db(), id).await?; // 404 si no existe esa fila
    Ok(Json(todo))
}
```

| Código | Devuelve |
|---|---|
| `Todo::all(db).await?` | `Vec<Todo>`: todas las filas |
| `Todo::get(db, id).await?` | `Todo`, o un error «no encontrado» (`404` en un handler) |
| `Todo::find(db, id).await?` | `Option<Todo>` |
| `Todo::find_many(db, [1, 2, 3]).await?` | `Vec<Todo>`: las filas de esas claves, en una sola consulta |
| `Todo::count(db).await?` | `u64` |
| `todo.reload(db).await?` | actualiza un struct desde la base de datos |

### Consultas

`Todo::query()` empieza una consulta. Encadena filtros, ordenación y límites, y luego ejecútala:

```rust
async fn open_todos(req: Request) -> Result<Json<Vec<Todo>>> {
    let todos = Todo::query()
        .filter(Todo::DONE.eq(false))
        .filter(Todo::TITLE.contains("docs"))
        .order_by(Todo::ID.desc())
        .limit(20)
        .all(req.db())
        .await?;
    Ok(Json(todos))
}
```

Las constantes de columna están tipadas: `Todo::DONE.eq("yes")` no compila, porque `done` es un `bool`.

| Filtro | Código |
|---|---|
| igual, distinto | `Todo::DONE.eq(false)`, `.ne(false)` |
| comparaciones | `User::VISITS.gt(10)`, `.ge(10)`, `.lt(10)`, `.le(10)` |
| y, o, no | `a & b`, `a \| b`, `!a`, o varias llamadas a `.filter(...)` (y) |
| en una lista | `Todo::ID.is_in([1, 2, 3])` |
| entre | `User::VISITS.between(1, 10)` |
| nulo | `User::BIO.is_null()`, `.is_not_null()` |
| texto | `.contains("x")`, `.starts_with("x")`, `.ends_with("x")`, `.icontains("x")` (sin distinguir mayúsculas), `.like("%x%")` |
| búsqueda de texto completo | `Book::TITLE.matches("rust orm")`: las filas cuyo texto contiene todas las palabras, como palabras completas, sin distinguir mayúsculas, en todas las bases |

| Se ejecuta con | Devuelve |
|---|---|
| `.all(db)` | `Vec<Todo>` |
| `.first(db)` | `Option<Todo>` |
| `.one(db)` | `Todo`, o «no encontrado» |
| `.count(db)` | `u64` |
| `.exists(db)` | `bool` |
| `.paginate(db, page, per_page)` | una página de resultados: `items`, `page`, `per_page` y `total` |
| `.columns(Todo::TITLE).all(db)` | `Vec<String>`: solo esa columna |

Ordenación y paginación: `.order_by(Todo::TITLE)` (ascendente), `.order_by(Todo::ID.desc())`, `.limit(20)`, `.offset(40)`, `.page(3, 20)`.

Una lista paginada en una página:

```rust
async fn todo_list(req: Request) -> Result<Page> {
    let page: u64 = req.query("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let result = Todo::query().order_by(Todo::ID.asc()).paginate(req.db(), page, 20).await?;
    Ok(Page::new(format!("Tareas, página {page}")).body(
        section()
            .child(p().class("muted").text(format!("{} tareas en total", result.total)))
            .child(ul().children(result.items.into_iter().map(|t| li().text(t.title)))),
    ))
}
```

El valor que devuelve `paginate` es el tipo `Page` propio de Memoire (`paquets::orm::Page`), no la `Page` HTML: solo usas sus campos.

## Filas relacionadas

Una columna declarada con `references` enlaza dos modelos, y Memoire lee el enlace en ella: no hay nada más que declarar.

```rust
#[model(table = "users")]
struct User {
    id: i64,
    name: String,
}

#[model(table = "posts")]
struct Post {
    id: i64,
    #[memoire(references = "users(id)", index)]
    author_id: i64,
    title: String,
}
```

| Código | Devuelve |
|---|---|
| `Post::belonging_to(&user)` | una consulta sobre los artículos de un usuario, para filtrar, ordenar o contar como cualquier otra |
| `post.find_parent::<User, _>(db).await?` | `Option<User>`: el autor de un artículo |
| `users.load_children::<Post>(db).await?` | `Vec<Vec<Post>>`: los artículos de cada usuario, alineados con `users` |
| `posts.load_parent::<User>(db).await?` | `Vec<Option<User>>`: el autor de cada artículo |
| `posts.load_via::<PostTag, Tag>(db).await?` | muchos a muchos, a través de un modelo que referencia a los dos |

`load_children`, `load_parent` y `load_via` (el trait `LoadRelated`, en el prelude) hacen una consulta por nivel, sea cual sea el número de filas de partida, en lugar de una consulta por fila:

```rust
async fn authors(req: Request) -> Result<Page> {
    let users = User::query().order_by(User::NAME.asc()).all(req.db()).await?;
    let posts = users.load_children::<Post>(req.db()).await?; // una sola consulta para todos los artículos
    Ok(Page::new("Autores").body(ul().children(users.into_iter().zip(posts).map(
        |(user, posts)| li().text(format!("{}: {} artículos", user.name, posts.len())),
    ))))
}
```

## Escribir datos

```rust
// Insertar: el struct debe ser `mut`, el id generado se escribe de vuelta en él.
let mut todo = Todo { id: 0, title: "Escribir la documentación".into(), done: false };
todo.insert(db).await?;
println!("nuevo id: {}", todo.id);

// Actualizar todas las columnas de la fila con este id («no encontrado» si no existe).
todo.done = true;
todo.update(db).await?;

// Eliminar: `true` si se eliminó una fila.
let deleted: bool = Todo::delete_by_key(db, todo.id).await?;
```

| Código | Efecto |
|---|---|
| `row.insert(db)` | inserta la fila y escribe de vuelta el `id` generado |
| `Todo::insert_many(db, &rows)` | inserta varias filas, devuelve cuántas |
| `row.upsert(db)` | inserta, o reemplaza la fila con la misma clave |
| `row.update(db)` | guarda todas las columnas de la fila |
| `row.update_only(db, (Todo::TITLE, Todo::DONE))` | guarda solo esas columnas |
| `row.delete(db)` | elimina la fila, devuelve `bool` |
| `Todo::delete_by_key(db, id)` | elimina por clave, devuelve `bool` |
| `Todo::update_many().set(Todo::DONE, true).filter(...).execute(db)` | actualiza muchas filas, devuelve cuántas |
| `Todo::delete_many().filter(...).execute(db)` | elimina muchas filas, devuelve cuántas |

`update_many` también tiene `.increment(column, n)`, `.decrement(column, n)` y `.set_null(column)`.

## Transacciones

Una transacción agrupa varios cambios: o se guardan todos, o ninguno. Empieza una con `db.begin()`, pasa `&tx` en lugar de la base de datos y luego haz `commit`:

```rust
#[model]
struct Account {
    id: i64,
    balance: i64,
}

#[data]
struct Transfer {
    from: i64,
    to: i64,
    amount: i64,
}

async fn transfer(req: Request) -> Result<StatusCode> {
    let t: Transfer = req.json().await?;
    let tx = req.db().begin().await?;
    Account::update_many()
        .decrement(Account::BALANCE, t.amount)
        .filter(Account::ID.eq(t.from))
        .execute(&tx)
        .await?;
    Account::update_many()
        .increment(Account::BALANCE, t.amount)
        .filter(Account::ID.eq(t.to))
        .execute(&tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
```

Si algo falla antes del `commit` (un `?` sale antes de tiempo), la transacción se descarta y todos los cambios se deshacen.

## Datos al arrancar

`app.on_start` se ejecuta una vez, después de abrir la base de datos y de que existan las tablas, antes de la primera petición. Úsalo para añadir los datos iniciales:

```rust
app.on_start(|db| async move {
    if Todo::count(db).await? == 0 {
        Todo { id: 0, title: "Prueba Paquets".into(), done: false }.insert(db).await?;
    }
    Ok(())
});
```

Si el hook devuelve un error, la aplicación no arranca y el mensaje empieza por `startup:`. Puedes llamar a `on_start` varias veces: los hooks se ejecutan en orden.

## Modificar una tabla más adelante

Las tablas se crean cuando no existen, pero **nunca se modifican**. Si añades un campo a un modelo cuya tabla ya existe (en un archivo SQLite o en Heroku Postgres), falta la columna, y las consultas sobre ese modelo fallan con un `500`. Con la base de datos en memoria (desarrollo, pruebas) no hay nada que hacer: la tabla se vuelve a crear en cada arranque.

Para una base de datos existente, usa una **migración**: un cambio numerado, registrado en la base de datos para que solo se ejecute una vez. El `Migrator` de Memoire se encarga de ello, y `on_start` es el sitio donde ejecutarlo. Este ejemplo añade una columna `priority` a la tabla `todo` en PostgreSQL:

```rust
use paquets::orm::{Dialect, Migration, Migrator};

app.on_start(|db| async move {
    Migrator::new()
        .add(Migration::new(1, "add_todo_priority").sql_for(
            Dialect::Postgres,
            "ALTER TABLE todo ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 0",
        ))
        .run(db)
        .await?;
    Ok(())
});
```

Después añade `priority: i32` al struct. Cómo funciona y qué reglas seguir:

- `sql_for(Dialect::Postgres, ...)` solo se ejecuta en PostgreSQL; se omite en la base de datos en memoria, así que tus pruebas siguen funcionando. También existen `Dialect::Sqlite` y `Dialect::MySql`, y `.sql(...)` para todas las bases de datos SQL.
- `IF NOT EXISTS` hace que el cambio también sea seguro en una base de datos completamente nueva: ahí la tabla se acaba de crear a partir del struct actual y ya tiene la columna.
- Nunca modifiques una migración que ya se haya ejecutado en algún sitio: añade una nueva con el número siguiente.
- No uses `create_table::<M>()` en estas migraciones para un modelo registrado con `app.api` o `app.model`: su tabla ya se creó antes de que se ejecute `on_start`, y la migración fallaría con «table already exists».

> [!NOTE]
> SQLite y MySQL no admiten `ADD COLUMN IF NOT EXISTS`. Para ellas, ejecuta tú las migraciones **antes** de que la aplicación cree sus tablas: abre la base de datos con `Db::connect(&url).await?`, ejecuta el `Migrator` (con pasos `create_table` construidos a partir de copias congeladas de tus structs), pásale luego la base de datos a la aplicación con `app.database(db)` y arráncala con `app.serve().await` dentro de `#[tokio::main]`. La [guía de migraciones](https://maxlestage.github.io/Memoire/es/docs/migrations/) de Memoire explica en detalle los structs congelados.

## SQL sin procesar

Cuando el constructor de consultas no basta, escribe SQL. Los marcadores de posición son `?` en SQLite y MySQL, y `$1`, `$2`… en PostgreSQL. Las filas se decodifican por posición, en tuplas o en structs:

```rust
use paquets::orm::params;

async fn top_visitors(req: Request) -> Result<Json<Vec<(i64, String)>>> {
    let rows: Vec<(i64, String)> = req
        .db()
        .raw("SELECT id, name FROM users WHERE visits > ? ORDER BY visits DESC", &params![10])
        .await?;
    Ok(Json(rows))
}
```

`db.raw_execute(sql, &params![...])` ejecuta una sentencia que no devuelve filas y da el número de filas afectadas. El SQL sin procesar funciona en SQLite, PostgreSQL y MySQL; la base de datos en memoria, Redis y MongoDB devuelven un error «unsupported».

## Errores

Los errores de la base de datos se convierten en estados HTTP cuando llegan a un handler a través de `?`:

| Error | Estado |
|---|---|
| no existe la fila (`get`, `one`, `update`) | `404 Not Found`: `{"error":"record not found"}` |
| valor duplicado en una columna `unique` | `409 Conflict`: ``{"error":"unique constraint violated on `user.email`"}`` |
| otra restricción (clave foránea…), conflicto de transacción | `409 Conflict` |
| `NULL` en un campo que no es `Option`, valor no válido | `400 Bad Request` |
| cualquier otra cosa (conexión perdida, tabla o columna que falta…) | `500 Internal Server Error`; los detalles van a los logs del servidor, nunca al cliente |

Si prefieres reaccionar tú mismo a un error, compruébalo con un `match` antes del `?`: `Todo::find` devuelve `None` en lugar de un error, y los errores de Memoire tienen `is_not_found()`, `is_unique_violation()` e `is_conflict()`. Consulta [Errores](errors.md).

## ¿Qué base de datos usar?

| Situación | Elección |
|---|---|
| Aprendizaje, prototipos, pruebas | en memoria (nada que configurar) |
| Un servidor con un disco persistente (un VPS, una Raspberry Pi, un volumen de Docker) | SQLite: un archivo, nada que instalar, muy rápido; o `memory://app.memoire`, el motor en memoria guardado en un archivo |
| Heroku, o varios servidores | PostgreSQL |
| Una base de datos existente | la que tengas: MySQL/MariaDB, Redis o MongoDB |

> [!WARNING]
> No uses SQLite en Heroku: el sistema de archivos de un dyno es temporal, y el archivo se pierde en cada despliegue y en cada reinicio (al menos una vez al día). Añade en su lugar el add-on Heroku Postgres (consulta [Desplegar en Heroku](deploy-heroku-mobile.md#2-añadir-una-base-de-datos)).

Con PostgreSQL, el pool abre hasta 4 conexiones por núcleo de CPU cuando el tráfico lo necesita. Los planes alojados pequeños limitan las conexiones (a menudo a 20): para elegir tú el tamaño del pool, abre el pool con Memoire (`PgPool::connect_with(&url, 10)`) y pásalo con `app.database(Db::from(pool))` desde un `main` asíncrono (consulta [Configuración](configuration.md#inicialización-asíncrona)).

## Más información

La documentación de Memoire va más allá: [modelos](https://maxlestage.github.io/Memoire/es/docs/models/), [consultas](https://maxlestage.github.io/Memoire/es/docs/queries/) (joins, agregados, group by), [relaciones](https://maxlestage.github.io/Memoire/es/docs/relations/), [transacciones](https://maxlestage.github.io/Memoire/es/docs/transactions/), [migraciones](https://maxlestage.github.io/Memoire/es/docs/migrations/), [notificaciones de cambios](https://maxlestage.github.io/Memoire/es/docs/events/) y una página por base de datos. Todo funciona con `req.db()`: acepta todos los métodos de los modelos y todas las consultas. En Paquets, Memoire se reexporta como `paquets::orm`.

## Próximos pasos

- [API REST](rest-api.md): una API JSON completa para un modelo en una sola línea.
- [Pruebas](testing.md): prueba tus consultas con una base de datos en memoria.
- [Errores](errors.md): lo que hace `?` con los errores de la base de datos.
