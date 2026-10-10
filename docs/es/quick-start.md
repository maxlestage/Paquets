# Inicio rápido

En este tutorial construyes **Notebook**, una pequeña aplicación con una página de inicio, un formulario, una base de datos y una API JSON, y luego la pruebas. Lleva unos diez minutos y solo supone que Rust y el comando `paquets` están instalados (consulta [Instalación](installation.md)).

## 1. Crea el proyecto

```sh
paquets new notebook --minimal
cd notebook
```

`--minimal` crea un proyecto con un único archivo fuente, `src/main.rs`, y sin islas, así que no necesitas ninguna herramienta de WebAssembly para este tutorial. Sin `--minimal`, también obtienes un `src/lib.rs` con un contador interactivo (consulta [Islas](islands.md)). Los dos tipos de proyecto contienen los mismos archivos de despliegue; [Estructura del proyecto](project-structure.md) los describe.

## 2. Ejecútalo

```sh
paquets dev
```

La primera ejecución compila todas las dependencias, lo que lleva unos minutos. Las siguientes solo recompilan tu código. Cuando el servidor está listo, muestra:

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: none
```

Abre http://localhost:3000 en tu navegador: verás la página inicial del proyecto. La segunda línea te dice que la base de datos está en memoria, que el escudo de seguridad está activo con sus 10 guards y que el proyecto no tiene islas.

A partir de ahí, cada petición muestra una línea, como `GET / 200 0.339 ms`. Detén el servidor con `Ctrl+C`.

> [!TIP]
> `paquets dev` es `cargo run` más la compilación de las islas cuando el proyecto las tiene. En un proyecto mínimo, `cargo run` hace exactamente lo mismo.

## 3. Escribe la página de inicio

Reemplaza el contenido de `src/main.rs` por este programa:

```rust
use paquets::prelude::*;

/// La página de inicio.
async fn home(_req: Request) -> Page {
    Page::new("Notebook")
        .description("Mis notas, en Rust.")
        .body(html::main().child(h1().text("📒 Notebook")))
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

Reinicia el servidor (`Ctrl+C` y luego `paquets dev`) y recarga la página. Leamos el programa empezando por abajo:

- `main` construye la aplicación y la ejecuta. `run()` bloquea hasta que pulsas `Ctrl+C`.
- `app()` crea la aplicación y declara sus rutas. Construir la aplicación en su propia función permite que las pruebas usen exactamente la misma aplicación, como verás en el paso 8.
- `app.get("/", home)` dice: para una petición `GET` a `/`, llama a `home`.
- `home` es un **handler**: una función `async` que recibe la petición y devuelve una respuesta. Aquí la respuesta es una `Page`, un documento HTML completo con un título, una descripción para los buscadores y las redes sociales, y un tema integrado.
- El cuerpo es una **vista** construida con una función por etiqueta HTML: `h1().text("📒 Notebook")` es `<h1>📒 Notebook</h1>`.

> [!NOTE]
> ¿Por qué `html::main()` y no `main()`? En este archivo, el nombre `main` ya lo ocupa la función `main` del programa. Por eso aquí la etiqueta `<main>` se escribe `html::main()`. Consulta [Páginas](pages.md#la-etiqueta-main-en-mainrs).

## 4. Añade una ruta con un parámetro

Añade esta ruta en `app()`, después de `app.get("/", home);`:

```rust
app.get("/hello/:name", |req: Request| async move {
    format!("¡Hola, {}!", req.param("name").unwrap_or("tú"))
});
```

Reinicia y abre http://localhost:3000/hello/Ada: la página dice `¡Hola, Ada!`. El segmento `:name` de la ruta es un parámetro, que se lee con `req.param("name")`. Este handler es una closure en lugar de una función con nombre, y devuelve un `String`, que se convierte en una respuesta de texto plano. Consulta [Enrutamiento](routing.md) y [Peticiones y respuestas](requests-responses.md).

## 5. Guarda notas en una base de datos

Añade un **modelo** al principio del archivo, después de la línea `use`:

```rust
/// Una nota: una tabla de la base de datos, que también se envía y se recibe como JSON.
#[model]
struct Note {
    id: i64,
    text: String,
    done: bool,
}
```

y una línea en `app()`:

```rust
app.api::<Note>("/api/notes");
```

Eso es una API REST completa. Reinicia el servidor y pruébala desde otra terminal:

```sh
# Crear una nota: 201 Created, con el id elegido por la base de datos
curl -X POST http://localhost:3000/api/notes \
  -H 'content-type: application/json' \
  -d '{"text": "Aprender Rust", "done": false}'

# Listar las notas
curl http://localhost:3000/api/notes

# Marcar la nota 1 como hecha (PATCH solo cambia los campos que envías)
curl -X PATCH http://localhost:3000/api/notes/1 \
  -H 'content-type: application/json' \
  -d '{"done": true}'

# Eliminarla: 204 No Content
curl -X DELETE http://localhost:3000/api/notes/1
```

Qué ha pasado:

- `#[model]` convirtió el struct en una tabla de la base de datos llamada `note`, con columnas tipadas. El campo `id` es la clave primaria, generada por la base de datos. El struct también se puede convertir a JSON y desde JSON.
- Al arrancar, `app.api::<Note>(...)` creó la tabla (todavía no existía) y añadió seis rutas: listar, leer, crear, reemplazar, modificar y eliminar. Consulta [API REST](rest-api.md).
- De momento la base de datos está en memoria: si detienes el servidor, las notas desaparecen. El paso 7 lo soluciona.

## 6. Muestra las notas en la página, con un formulario

Reemplaza `home` por una versión que lee las notas y añade un handler para el formulario:

```rust
/// Los datos que envía el formulario «nueva nota».
#[data]
struct NewNote {
    text: String,
}

/// La página de inicio: el formulario y la lista de notas.
async fn home(req: Request) -> Result<Page> {
    let notes = Note::query().order_by(Note::ID.desc()).all(req.db()).await?;
    Ok(Page::new("Notebook")
        .description("Mis notas, en Rust.")
        .body(
            html::main()
                .child(h1().text("📒 Notebook"))
                .child(
                    form()
                        .attr("method", "post")
                        .attr("action", "/notes")
                        .class("row")
                        .child(input().attr("name", "text").attr("placeholder", "Una nota nueva"))
                        .child(button().text("Añadir")),
                )
                .child(ul().children(notes.into_iter().map(|note| li().text(note.text)))),
        ))
}

/// Guarda la nota enviada por el formulario y vuelve a la página de inicio.
async fn add_note(req: Request) -> Result<Redirect> {
    let form: NewNote = req.form().await?;
    let mut note = Note { id: 0, text: form.text, done: false };
    note.insert(req.db()).await?;
    Ok(Redirect::see_other("/"))
}
```

Registra la ruta del formulario en `app()`:

```rust
app.post("/notes", add_note);
```

Reinicia, añade algunas notas con el formulario y mira cómo aparecen. Algunas novedades:

- `req.db()` le da la base de datos al handler. `Note::query()...all(req.db()).await?` lee todas las notas, de la más reciente a la más antigua. El compilador comprueba los nombres de las columnas (`Note::ID`).
- `#[data]` marca un struct que solo viaja como datos (un formulario, un cuerpo JSON), sin tabla.
- Ahora los handlers devuelven `Result<...>`. El `?` tras una llamada a la base de datos o a `req.form()` detiene el handler si hay un error y envía el estado HTTP correcto: `400` para un formulario no válido, `404` para una fila que no existe, `500` para un problema del servidor. Consulta [Errores](errors.md).
- `Redirect::see_other("/")` devuelve al navegador a la página de inicio después de enviar el formulario: el patrón clásico «post, redirect, get».
- El formulario no necesita token CSRF: el escudo de seguridad ya rechaza los formularios enviados desde otros sitios web. Consulta [Seguridad](security.md).

## 7. Conserva los datos

La base de datos se toma de la variable de entorno `DATABASE_URL`. Haz que apunte a un archivo SQLite:

```sh
DATABASE_URL=sqlite://notebook.db paquets dev
```

Ahora la línea de arranque dice `database: sqlite`. El archivo `notebook.db` se crea junto a `Cargo.toml`, la tabla `note` se crea dentro y tus notas sobreviven a los reinicios. En Windows PowerShell, escribe `$env:DATABASE_URL="sqlite://notebook.db"; paquets dev`.

¿Quieres una primera nota en una base de datos vacía? Añade esto a `app()`, antes de `app`:

```rust
app.on_start(|db| async move {
    if Note::count(db).await? == 0 {
        Note { id: 0, text: "Prueba Paquets".into(), done: false }.insert(db).await?;
    }
    Ok(())
});
```

`on_start` se ejecuta una vez, cuando la base de datos está lista y las tablas existen, antes de la primera petición. En producción en Heroku, `DATABASE_URL` apunta a PostgreSQL y el mismo código funciona sin cambios. Consulta [Base de datos](database.md).

## 8. Pruébalo

Añade esto al final de `src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_form_adds_a_note() {
        let client = app().test().await;

        let res = client.post("/notes").form(&[("text", "Escribir una prueba")]).await;
        assert_eq!(res.status(), 303);

        let html = client.get("/").await.text();
        assert!(html.contains("Prueba Paquets"));
        assert!(html.contains("Escribir una prueba"));
    }
}
```

Ejecuta las pruebas:

```sh
cargo test
```

`app().test().await` arranca tu aplicación real, con sus rutas, su seguridad y su hook `on_start`, sin abrir ningún puerto. La base de datos está en memoria y vacía en cada prueba, diga lo que diga `DATABASE_URL`, así que las pruebas nunca tocan tus datos. Consulta [Pruebas](testing.md).

## El programa completo

```rust
use paquets::prelude::*;

/// Una nota: una tabla de la base de datos, que también se envía y se recibe como JSON.
#[model]
struct Note {
    id: i64,
    text: String,
    done: bool,
}

/// Los datos que envía el formulario «nueva nota».
#[data]
struct NewNote {
    text: String,
}

/// La página de inicio: el formulario y la lista de notas.
async fn home(req: Request) -> Result<Page> {
    let notes = Note::query().order_by(Note::ID.desc()).all(req.db()).await?;
    Ok(Page::new("Notebook")
        .description("Mis notas, en Rust.")
        .body(
            html::main()
                .child(h1().text("📒 Notebook"))
                .child(
                    form()
                        .attr("method", "post")
                        .attr("action", "/notes")
                        .class("row")
                        .child(input().attr("name", "text").attr("placeholder", "Una nota nueva"))
                        .child(button().text("Añadir")),
                )
                .child(ul().children(notes.into_iter().map(|note| li().text(note.text)))),
        ))
}

/// Guarda la nota enviada por el formulario y vuelve a la página de inicio.
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
        format!("¡Hola, {}!", req.param("name").unwrap_or("tú"))
    });
    app.api::<Note>("/api/notes");
    app.on_start(|db| async move {
        if Note::count(db).await? == 0 {
            Note { id: 0, text: "Prueba Paquets".into(), done: false }.insert(db).await?;
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

        let res = client.post("/notes").form(&[("text", "Escribir una prueba")]).await;
        assert_eq!(res.status(), 303);

        let html = client.get("/").await.text();
        assert!(html.contains("Prueba Paquets"));
        assert!(html.contains("Escribir una prueba"));
    }
}
```

## 9. Ponlo en línea

Tu proyecto ya contiene todo lo necesario para desplegarlo: un `Dockerfile`, los archivos para Heroku y los workflows de GitHub. Súbelo a un repositorio de GitHub y sigue [Desplegar en Heroku desde el móvil](deploy-heroku-mobile.md): unos pocos toques, sin necesidad de ordenador. Recuerda que Heroku no tiene plan gratuito (un dyno Eco cuesta unos 5 $ al mes).

## Próximos pasos

- [Estructura del proyecto](project-structure.md): todos los archivos del proyecto.
- [Páginas](pages.md): layouts, el tema, las etiquetas SEO.
- [Islas](islands.md): añade partes interactivas que se ejecutan en el navegador.
- [Base de datos](database.md): consultas, transacciones, PostgreSQL.
