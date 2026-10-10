# API REST

`app.api::<Model>(path)` le da a un modelo una API JSON completa en una sola línea: listar, leer, crear, reemplazar, modificar y eliminar, con paginación, los códigos de estado correctos y errores claros. Esta página describe cada ruta, muestra cómo llamarlas y cómo proteger o ampliar la API.

## Una sola línea

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
    app.api::<Todo>("/api/todos");
    app.run()
}
```

Esta línea crea la tabla `todo` al arrancar (si todavía no existe) y añade seis rutas:

| Petición | Efecto | Éxito |
|---|---|---|
| `GET /api/todos` | lista las filas, por orden de clave, de 100 en 100 | `200`, array JSON, total en `X-Total-Count` |
| `GET /api/todos/:id` | lee una fila | `200`, objeto JSON |
| `POST /api/todos` | crea una fila a partir del cuerpo JSON | `201 Created`, la fila con su `id` generado |
| `PUT /api/todos/:id` | reemplaza la fila entera | `200`, la fila |
| `PATCH /api/todos/:id` | cambia solo los campos presentes en el cuerpo | `200`, la fila |
| `DELETE /api/todos/:id` | elimina la fila | `204 No Content` |

## Llamar a la API

Los ejemplos usan `curl` contra un servidor que escucha en el puerto 3000.

### Crear

```sh
curl -X POST http://localhost:3000/api/todos \
  -H 'content-type: application/json' \
  -d '{"title": "Documentar la API", "done": false}'
```

```http
HTTP/1.1 201 Created
content-type: application/json

{"id":1,"title":"Documentar la API","done":false}
```

El `id` lo genera la base de datos. Si el cuerpo contiene un `id`, se ignora. Todos los demás campos son obligatorios, salvo que sean un `Option` o tengan `#[serde(default)]` (consulta [Base de datos](database.md#campos-opcionales-en-json)).

### Listar

```sh
curl -i 'http://localhost:3000/api/todos?limit=20&offset=40'
```

```http
HTTP/1.1 200 OK
content-type: application/json
x-total-count: 57

[{"id":41,"title":"...","done":false}, ...]
```

| Parámetro de la query | Por defecto | Significado |
|---|---|---|
| `limit` | `100` | cuántas filas devolver, entre 1 y 1000 (los valores mayores se limitan a 1000) |
| `offset` | `0` | cuántas filas saltar |

Las filas siempre se ordenan por clave primaria. La cabecera `X-Total-Count` contiene el número total de filas, para que un cliente pueda calcular el número de páginas: `ceil(total / limit)`.

### Leer una fila

```sh
curl http://localhost:3000/api/todos/1
```

```json
{"id":1,"title":"Documentar la API","done":false}
```

### Reemplazar

`PUT` envía la fila entera. El `id` viene de la URL:

```sh
curl -X PUT http://localhost:3000/api/todos/1 \
  -H 'content-type: application/json' \
  -d '{"title": "Documentar mejor la API", "done": true}'
```

### Modificar algunos campos

`PATCH` envía solo los campos que hay que cambiar. Los demás conservan su valor:

```sh
curl -X PATCH http://localhost:3000/api/todos/1 \
  -H 'content-type: application/json' \
  -d '{"done": true}'
```

```json
{"id":1,"title":"Documentar mejor la API","done":true}
```

### Eliminar

```sh
curl -X DELETE http://localhost:3000/api/todos/1   # 204 No Content
```

## Errores

Cada error es un objeto JSON con un mensaje `error`:

| Situación | Estado | Cuerpo |
|---|---|---|
| El `id` de la URL no es válido para el tipo de la clave (`/api/todos/abc`) | `400` | ``{"error":"invalid id `abc`"}`` |
| `limit` u `offset` no es un entero positivo | `400` | ``{"error":"`limit` must be a positive integer"}`` |
| JSON no válido, campo que falta, tipo incorrecto | `400` | ``{"error":"invalid JSON: missing field `done` at line 1 column 29"}`` (para `{"title":"Documentar la API"}`) |
| `PATCH` con un cuerpo que no es un objeto JSON | `400` | `{"error":"expected a JSON object"}` |
| Ninguna fila con este `id` (`GET`, `PUT`, `PATCH`, `DELETE`) | `404` | `{"error":"record not found"}` |
| Una columna `unique` ya tiene este valor | `409` | ``{"error":"unique constraint violated on `member.email`"}`` |
| Un problema del servidor (base de datos inaccesible…) | `500` | `{"error":"Internal Server Error"}`, con los detalles en los logs |

## Claves

La clave primaria puede ser de cualquier tipo que se pueda leer de una URL:

- un `id` entero (el caso habitual), generado por la base de datos;
- un `String` elegido por el cliente, marcado con `#[memoire(primary_key)]`: `/api/tags/rust`;
- un `uuid::Uuid` (feature `uuid`).

```rust
#[model(table = "tags")]
struct Tag {
    #[memoire(primary_key)]
    slug: String,
    label: String,
}

app.api::<Tag>("/api/tags"); // POST {"slug": "rust", "label": "Rust"} y luego GET /api/tags/rust
```

Una clave de tipo cadena o UUID no se genera: el cliente debe enviarla en los cuerpos de los `POST`.

## Varias APIs

Llama a `app.api` una vez por modelo. Cada uno obtiene su propia tabla y sus propias rutas:

```rust
app.api::<Todo>("/api/todos");
app.api::<Tag>("/api/tags");
app.api::<Member>("/api/members");
```

No puedes declarar una ruta propia con el mismo método y la misma ruta que una ruta de `app.api` (por ejemplo tu propio `GET /api/todos`): la aplicación provoca un pánico al arrancar con `duplicate route`. Las demás rutas están libres, como `GET /api/todos/stats`.

## Proteger la API

Las rutas de `app.api` son públicas: cualquiera puede crear o eliminar filas. Está bien para un prototipo, rara vez para producción. El escudo ya bloquea los ataques, las avalanchas de peticiones y los formularios de otros sitios, pero no sabe quiénes son tus usuarios. Añade un middleware que compruebe algo que solo tienen tus clientes, en cada petición que modifica datos:

```rust
async fn require_token(req: Request, next: Next) -> Response {
    let writes = req.method() != Method::GET && req.method() != Method::HEAD;
    if req.path().starts_with("/api/") && writes {
        let expected = std::env::var("API_TOKEN").unwrap_or_default();
        let sent = req.header("authorization").and_then(|h| h.strip_prefix("Bearer "));
        if expected.is_empty() || sent != Some(expected.as_str()) {
            return Error::unauthorized("token ausente o incorrecto").into_response();
        }
    }
    next.run(req).await
}

app.middleware(require_token);
app.api::<Todo>("/api/todos");
```

La lectura sigue siendo pública, y la escritura necesita `Authorization: Bearer <token>`, donde el token es la variable de entorno `API_TOKEN` (una config var en Heroku). Para cuentas y sesiones, consulta [Seguridad](security.md#autenticación).

## Cuándo escribir tus propias rutas

`app.api` es un CRUD genérico. No valida los valores más allá de sus tipos, ni filtra las listas, ni oculta campos, ni comprueba a quién pertenece una fila. En cuanto necesites reglas, escribe tú las rutas: son unas pocas líneas con la API de la [base de datos](database.md).

```rust
#[data]
struct NewTodo {
    title: String,
}

async fn create_todo(req: Request) -> Result<(StatusCode, Json<Todo>)> {
    let new: NewTodo = req.json().await?;
    let title = new.title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err(Error::unprocessable("el título debe tener entre 1 y 200 caracteres"));
    }
    let mut todo = Todo { id: 0, title: title.to_string(), done: false };
    todo.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

async fn search_todos(req: Request) -> Result<Json<Vec<Todo>>> {
    let q = req.query("q").unwrap_or_default().to_string();
    let todos = Todo::query()
        .filter(Todo::TITLE.icontains(q))
        .order_by(Todo::ID.asc())
        .limit(50)
        .all(req.db())
        .await?;
    Ok(Json(todos))
}

app.model::<Todo>(); // la tabla, sin las rutas genéricas
app.post("/api/todos", create_todo);
app.get("/api/todos/search", search_todos);
```

## Llamar a la API desde un navegador

Una página de tu propio sitio puede llamar a la API con `fetch`: la petición viene del mismo origen, así que el escudo la deja pasar. Un front end alojado en **otro** origen (otro dominio, o un servidor de desarrollo en otro puerto) necesita dos cambios en el escudo: CORS, para que el navegador permita a la página leer las respuestas, y un origen de confianza para el guard CSRF, para que se acepten sus peticiones `POST`, `PUT`, `PATCH` y `DELETE`:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .guard(Cors::new().allow_origin("https://app.example.com"))
        .guard(Csrf::new().trusted_origin("https://app.example.com"))
        .build(),
);
```

Consulta [Seguridad](security.md#modificar-el-escudo).

## Probar la API

```rust
#[tokio::test]
async fn creates_and_lists_todos() {
    let mut app = App::new();
    app.api::<Todo>("/api/todos");
    let client = app.test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Documentar la API", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
    let created: Todo = res.json();
    assert_eq!(created.id, 1);

    let res = client.get("/api/todos").await;
    assert_eq!(res.header("x-total-count"), Some("1"));
    assert_eq!(res.json::<Vec<Todo>>().len(), 1);

    assert_eq!(client.get("/api/todos/99").await.status(), 404);
}
```

Consulta [Pruebas](testing.md).

## Próximos pasos

- [Base de datos](database.md): modelos, consultas y transacciones.
- [Seguridad](security.md): el escudo, la autenticación, CORS.
- [Errores](errors.md): todos los estados, y cómo cambiar el formato de los errores.
