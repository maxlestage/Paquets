# Rendimiento

Paquets es rápido sin ajustes: su servidor, vitesse, es más rápido que actix-web y axum en la misma máquina, su escudo de seguridad cuesta unos pocos microsegundos por petición, y las páginas no envían ningún código al navegador salvo que contengan islas. Esta página da las cifras, explica cómo usa el servidor tu CPU y enumera lo que mantiene rápida tu propia aplicación.

## Las cifras

### El servidor (vitesse)

Peticiones por segundo en una VM de 4 vCPU, con el servidor fijado en 2 núcleos y el generador de carga (wrk) en los otros 2, con 128 conexiones keep-alive. Entre paréntesis: el tiempo de CPU del servidor por petición (cuanto menos, mejor).

| Escenario | axum 0.8 | actix-web 4 | **vitesse** |
|---|---:|---:|---:|
| `GET /` (texto) | 168 165 (11.7 µs) | 274 195 (7.1 µs) | **291 415 (6.1 µs)** |
| `GET /json` | 165 988 (11.9 µs) | 248 987 (7.9 µs) | **313 438 (5.9 µs)** |
| `GET /json` enviado por un navegador (12 cabeceras) | 131 710 (15.0 µs) | 179 505 (10.9 µs) | **290 478 (6.8 µs)** |
| `GET /users/:id` (parámetro + JSON) | 151 303 (13.1 µs) | 209 173 (9.4 µs) | **304 323 (6.2 µs)** |
| `POST /echo` (lee y devuelve JSON) | 105 570 (18.8 µs) | 170 129 (11.7 µs) | **253 941 (7.6 µs)** |
| `GET /` en pipeline ×16 | 213 678 (9.3 µs) | 1 272 510 (1.6 µs) | **2 781 541 (0.64 µs)** |

La metodología, los otros frameworks medidos (Express, Drogon) y la forma de reproducir estas cifras están en la [guía de rendimiento de vitesse](https://maxlestage.github.io/Vitesse/es/docs/performance/).

### El escudo (securite)

Todo el pipeline recomendado (identificador de petición, higiene, filtro de métodos, límite del cuerpo, límite de tasa, bloqueador de bots, cortafuegos, CSRF, cabeceras de seguridad, refuerzo de cookies) cuesta unos **6 µs** por petición.

### Páginas e islas (active)

| | Renderizado en el servidor de una página de producto | WebAssembly + cargador para un contador (comprimido) | Interactivo con 100 ms de latencia |
|---|---:|---:|---:|
| **active** | **5.6 µs** | **36.8 kB** | **224 ms** |
| Leptos 0.8 + leptos_meta | 13.4 µs | 60.9 kB | 227 ms |
| Sycamore 0.9 | 33.0 µs | 45.8 kB | 227 ms |
| Dioxus 0.7 | 24.2 µs | 170.4 kB | 339 ms |

El bundle más pequeño de todos, y un renderizado en el servidor 2.4 veces más rápido que el del otro framework que fusiona las etiquetas SEO de los componentes (Leptos con leptos_meta). Un motor de plantillas sin fusión de SEO, como maud, renderiza más rápido (1.4 µs), pero no envía nada al navegador. Los detalles están en los [benchmarks de active](https://maxlestage.github.io/Active/es/docs/benchmarks/).

### Lo que esto significa para tu aplicación

Estos benchmarks miden cada paquete por separado, con handlers triviales. En una aplicación Paquets real, una petición también pasa por el escudo y el log de peticiones, y la mayoría de los handlers consultan una base de datos. En la práctica, **el tiempo se va en la base de datos y en tu propio código**: la parte del framework son unos pocos microsegundos. Por eso los consejos siguientes se centran en ellos.

## Cómo usa el servidor tu CPU

- Con la base de datos en memoria o SQLite, `app.run()` arranca **un hilo por núcleo de CPU**, cada uno con su propio bucle de eventos y su propio socket de escucha. El kernel reparte las conexiones entre los hilos, y una petición nunca cambia de hilo. Es el modo más rápido de vitesse.
- Con un servidor de base de datos (PostgreSQL…), la aplicación funciona sobre el runtime multihilo de tokio, compartido con las conexiones a la base de datos, y los hilos libres toman trabajo de los ocupados.

En los dos modos, un handler que bloquea su hilo (un cálculo largo, una lectura de archivo bloqueante, `std::thread::sleep`) retrasa todas las demás peticiones de ese hilo. Mantén los handlers asíncronos y saca los cálculos pesados del bucle de eventos:

```rust
use paquets::vitesse::tokio::task;

async fn report(_req: Request) -> Result<String> {
    // Un cálculo largo: ejecútalo en un hilo pensado para el trabajo bloqueante.
    let total = task::spawn_blocking(|| (1..=50_000_000u64).sum::<u64>()).await?;
    Ok(format!("total: {total}"))
}
```

El hash de contraseñas, el procesamiento de imágenes y las compresiones grandes van en `spawn_blocking`.

## Consejos

### Compila en modo release

Una compilación debug (`cargo run`, `paquets dev`) puede ser diez veces más lenta, o más. Mide y despliega solo compilaciones release: `paquets build`, o `cargo build --release`. El `Dockerfile` de un proyecto generado compila en modo release, con optimización en tiempo de enlazado.

### Base de datos

- **Añade índices** a las columnas por las que filtras u ordenas: `#[memoire(index)]` (las columnas únicas ya están indexadas).
- **Lee solo lo que muestras**: `.limit(...)` o `.paginate(...)` en las listas, `.columns(...)` cuando necesitas una o dos columnas, `.count(...)` o `.exists(...)` en lugar de cargar filas para contarlas.
- **Evita una consulta por fila** (el problema «N+1»). Carga las filas relacionadas en una sola consulta con `is_in`:

```rust
async fn posts_with_authors(req: Request) -> Result<Json<Vec<(String, String)>>> {
    let db = req.db();
    let posts = Post::query().order_by(Post::ID.desc()).limit(20).all(db).await?;
    // Una sola consulta para todos los autores, en lugar de una por artículo.
    let ids: Vec<i64> = posts.iter().map(|p| p.author_id).collect();
    let authors = Author::query().filter(Author::ID.is_in(ids)).all(db).await?;
    let rows = posts
        .into_iter()
        .map(|post| {
            let name = authors.iter().find(|a| a.id == post.author_id).map_or("?", |a| a.name.as_str());
            (post.title, name.to_string())
        })
        .collect();
    Ok(Json(rows))
}
```

- **Agrupa las escrituras** en una transacción, o usa `insert_many`, `update_many` y `delete_many`.
- memoire construye en tiempo de compilación el SQL de las operaciones por clave primaria (`get`, `insert`, `update`, `delete`): no cuestan ninguna generación de SQL en tiempo de ejecución.

### Páginas

- Una página sin islas no envía nada de JavaScript. Añade islas solo donde la página deba reaccionar en el navegador, y mantén sus props pequeñas: forman parte del HTML.
- El tema integrado ocupa unos 3.5 kB, inline en cada página. Para una hoja de estilos grande que comparten todas las páginas, ponla en `public/` y enlázala con `.stylesheet(...)`: los navegadores la guardan en caché.

### Archivos estáticos

Los archivos de `public/` se envían con `ETag` y `Last-Modified`, y los navegadores los revalidan (`304 Not Modified`). Para los archivos cuyo nombre cambia con su contenido (`app.3f9c2b.css`), permite una caché larga con una carpeta dedicada:

```rust
use std::time::Duration;
use paquets::vitesse::ServeDir;

app.serve_dir("/assets", ServeDir::new("assets").max_age(Duration::from_secs(31_536_000)));
```

### Compresión

vitesse no comprime las respuestas. El texto se comprime muy bien (HTML, JSON, CSS, WebAssembly), así que pon una CDN (Cloudflare, Fastly…) o un proxy inverso delante de la aplicación si el ancho de banda importa. El router de Heroku tampoco comprime.

### Logs

Una línea de log por petición cuesta poco, pero escribir en la terminal no es gratis con mucha carga. `app.quiet()` desactiva el log de peticiones, para los benchmarks o cuando tu plataforma ya registra las peticiones.

## Medir

Compila en modo release y luego somete la aplicación a carga con una herramienta como [oha](https://crates.io/crates/oha) o wrk:

```sh
paquets build
./target/release/my-app &
oha -z 10s -c 64 http://127.0.0.1:3000/
```

> [!IMPORTANT]
> Todas las peticiones de una prueba de carga vienen de una sola dirección IP, así que el límite de tasa del escudo (20 peticiones por segundo por IP, ráfagas de 100) responde `429` a la mayoría de ellas, y lo que mides es el limitador. Para un benchmark en tu propia máquina, quita el límite de tasa y el log de peticiones:
>
> ```rust
> app.security(paquets::shield().without("rate_limit").build());
> app.quiet();
> ```
>
> No despliegues nunca esa configuración: el límite de tasa es lo que protege tu aplicación de las avalanchas de peticiones.

Mide lo que hacen tus usuarios: las páginas y las rutas de la API que importan, con datos realistas en la base de datos.

## Próximos pasos

- [Configuración](configuration.md): workers, hilos y el runtime.
- [Base de datos](database.md): consultas, índices y transacciones.
- [Desplegar con Docker](deploy-docker.md): una imagen de producción optimizada.
