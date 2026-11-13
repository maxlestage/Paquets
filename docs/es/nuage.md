# Nuage DB

Nuage DB es la base de datos de los proyectos Paquets: `paquets new` la usa desde la versión 0.3.3. Guarda tus datos en una carpeta del proyecto mientras desarrollas, y en un servidor Nuage DB en producción, con el mismo código. Esta página explica cómo funciona en Paquets, cómo ejecutar y desplegar un servidor, conectar una aplicación, usar ramas para previsualizaciones y pruebas, la consola y el panel.

## Por qué Nuage DB

[Nuage DB](https://github.com/maxlestage/Nuage) es la base de datos de Memoire, el ORM de Paquets, del mismo autor:

- **Escrita enteramente en Rust**, como el resto de tu aplicación: un solo lenguaje desde el navegador hasta el disco.
- **Se cuida sola**: un registro de escritura anticipada (write-ahead log) sincronizado en disco antes de cada confirmación, recuperación tras un fallo al arrancar, puntos de control y compactación automáticos, sumas de verificación en el registro y en las páginas, y copias de seguridad automáticas en una carpeta o en un almacenamiento compatible con S3, restauradas por sí solas cuando el servidor arranca con un disco vacío.
- **En disco, sin límite de memoria**: tablas e índices viven en un archivo B-tree con una caché de páginas acotada; una búsqueda por clave primaria o por índice solo lee las filas que necesita.
- **Ramas instantáneas**: una copia copy-on-write de toda la base, hecha en un instante sea cual sea su tamaño, para una previsualización o una prueba.
- **Un solo servidor, siempre encendido**, que sirve la base, la consola `/admin` y el panel en directo `/app` en un solo puerto.
- **También desde WebAssembly**: el cliente habla HTTP(S), en nativo y con el `fetch` del navegador.

Tus modelos y consultas no cambian: Nuage DB ejecuta las instrucciones de Memoire con el propio motor de Memoire, así que `req.db()`, el constructor de consultas, las relaciones, las transacciones y las migraciones funcionan como con cualquier otra base.

## Desarrollo y producción

Un proyecto creado con `paquets new` elige Nuage DB en `src/main.rs`:

```rust
fn app() -> App {
    let mut app = App::new();
    // Nuage DB: kept in data/ in development, on a Nuage DB server in production
    // (NUAGE_URL and NUAGE_TOKEN), without changing this code.
    app.nuage("data");
    // ...
    app
}
```

El entorno decide adónde van los datos:

| Entorno | Base de datos | Línea de arranque |
|---|---|---|
| nada (desarrollo) | el modo local de Nuage DB, en la carpeta `data/` | `database: nuage, local (data)` |
| `NUAGE_URL=https://mi-nuage.herokuapp.com`, `NUAGE_TOKEN=…` | un servidor Nuage DB | `database: nuage (https://mi-nuage.herokuapp.com)` |
| `NUAGE_URL=otra-carpeta` | el modo local, en esa carpeta | `database: nuage, local (otra-carpeta)` |
| `DATABASE_URL=…` sin `NUAGE_URL` | esa base: PostgreSQL, SQLite… (ver [Base de datos](database.md#elegir-la-base-de-datos)) | `database: postgres`… |

`NUAGE_URL` tiene prioridad sobre `DATABASE_URL`, que tiene prioridad sobre la carpeta de `app.nuage(...)`. Una base elegida en el código con `app.database(...)` tiene prioridad sobre todas, y las pruebas creadas con `app.test()` siempre tienen una base nueva en memoria.

**El modo local** guarda la base en `data/app.memoire`, con el motor con el que Nuage DB ejecuta las instrucciones (el de Memoire): una consulta da las mismas filas en tu ordenador y en el servidor. La carpeta se crea si no existe; `paquets new` la añade a `.gitignore` y `.dockerignore`. Toda la base se mantiene en memoria mientras la aplicación se ejecuta, lo que va bien para desarrollar; las ramas, la consola y el panel vienen con un servidor, que también puedes ejecutar en tu máquina (siguiente sección).

## Ejecutar un servidor en tu máquina

El servidor es el programa `nuage` del [repositorio Nuage](https://github.com/maxlestage/Nuage):

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
NUAGE_TOKEN=dev-token NUAGE_ADMIN_PASSWORD=dev-password cargo run -p nuage --release
# nuage: data is stored in /home/yo/Nuage/nuage-data on this server's disk
# nuage: listening on 0.0.0.0:8080
```

Luego, en tu proyecto:

```bash
NUAGE_URL=http://localhost:8080 NUAGE_TOKEN=dev-token paquets dev
#   database: nuage (http://localhost:8080) · security: on (10 guards) · islands: /pkg/my_app.js
```

Abre http://localhost:8080/admin para la consola y http://localhost:8080/app para el panel (inicia sesión con `NUAGE_ADMIN_PASSWORD`).

## Desplegar un servidor Nuage DB

El servidor es un solo programa configurado con variables de entorno. Despliégalo una vez, junto a tus aplicaciones:

| Variable | Uso |
|---|---|
| `PORT` | el puerto (lo define Heroku y la mayoría de los proveedores; `8080` si no) |
| `NUAGE_TOKEN` | el token de la API de la base: sin él, la API rechaza todas las peticiones |
| `NUAGE_ADMIN_PASSWORD` | la contraseña de `/admin` y `/app`: sin ella, la consola está apagada |
| `NUAGE_DATA_DIR` | la carpeta de los datos (`nuage-data` por defecto) |
| `NUAGE_BACKUP_DIR` | copias de seguridad en una carpeta |
| `NUAGE_BACKUP_S3_BUCKET`, `_REGION`, `_ENDPOINT`, `_ACCESS_KEY_ID`, `_SECRET_ACCESS_KEY` | copias de seguridad en un almacenamiento compatible con S3 |
| `NUAGE_CACHE_MB` | la caché de páginas, en MiB (64 por defecto) |

**En Heroku**, la raíz del repositorio Nuage se despliega tal cual (`Procfile`, `RustConfig`, `heroku-build.sh`, y `app.json` para un botón **Deploy to Heroku**):

1. Crea la aplicación con el buildpack de Rust: `heroku create mi-nuage --buildpack emk/rust`, o desde el panel, con el botón del repositorio Nuage.
2. Crea un bucket (AWS S3, Cloudflare R2, Scaleway…) y una clave de acceso limitada a él.
3. Define las config vars:

   ```bash
   heroku config:set NUAGE_TOKEN=$(openssl rand -hex 32) \
     NUAGE_ADMIN_PASSWORD='una frase secreta larga' \
     NUAGE_BACKUP_S3_BUCKET=mi-bucket NUAGE_BACKUP_S3_REGION=eu-west-3 \
     NUAGE_BACKUP_S3_ACCESS_KEY_ID=... NUAGE_BACKUP_S3_SECRET_ACCESS_KEY=...
   ```

4. Despliega y abre `https://mi-nuage.herokuapp.com/admin`.

> [!WARNING]
> El disco de un dyno se borra en cada reinicio (al menos una vez al día). La carpeta de datos es solo una copia de trabajo: **los datos viven en las copias de seguridad**, y cada arranque los restaura desde ellas. Configura las copias en S3 en Heroku, y ejecuta **exactamente un** dyno Basic (nunca se duerme): dos dynos serían dos bases escribiendo en las mismas copias. Las confirmaciones hechas desde el último segmento de registro enviado (30 segundos por defecto) solo se pierden si el dyno se detiene sin el `SIGTERM` que Heroku envía antes.

**Con Docker**, en cualquier proveedor con un volumen persistente:

```bash
git clone https://github.com/maxlestage/Nuage && cd Nuage
docker build -t nuage .
docker run -p 8080:8080 -v nuage-data:/data -e NUAGE_TOKEN=... -e NUAGE_ADMIN_PASSWORD=... nuage
```

Sirve el servidor por HTTPS (Heroku y la mayoría de los proveedores lo hacen por ti): el token viaja en una cabecera, y la cookie de la consola es `Secure`.

## Conectar tu aplicación

Dale a tu aplicación la dirección y el token del servidor. En Heroku: tu aplicación → **Settings** → **Reveal Config Vars**:

| Config var | Valor |
|---|---|
| `NUAGE_URL` | `https://mi-nuage.herokuapp.com` |
| `NUAGE_TOKEN` | el `NUAGE_TOKEN` del servidor |

La aplicación se reinicia, se conecta, comprueba el token y crea las tablas de tus modelos. Si no se puede llegar al servidor o rechaza el token, la aplicación no arranca y dice por qué. El token nunca forma parte de la URL, y la línea de arranque nunca lo muestra.

Paquets mantiene abiertas las conexiones HTTP con el servidor entre peticiones, y usa TLS para las direcciones `https://`. Con un servidor, tu aplicación se ejecuta en un runtime tokio multihilo, como con PostgreSQL.

Para abrir la base tú mismo, `Db::nuage` abre cualquiera de los dos (la dirección de un servidor, o una carpeta), y `Db::connect` también acepta la dirección de un servidor, con el token de `NUAGE_TOKEN`. Hazlo desde un `main` asíncrono que arranque la aplicación con `app.serve().await` (ver [Configuración](configuration.md#inicialización-asíncrona)), para que las conexiones vivan en el runtime de la aplicación:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let token = std::env::var("NUAGE_TOKEN").ok();
    let db = Db::nuage("https://mi-nuage.herokuapp.com", token.as_deref())
        .await
        .map_err(std::io::Error::other)?;
    let mut app = app();
    app.database(db);
    app.serve().await
}
```

## Ramas para previsualizaciones y pruebas

Una rama es una copia de la base hecha en un instante, sea cual sea su tamaño: comparte todas las páginas de su madre y solo guarda sus propios cambios. Los cambios posteriores de la base principal no le llegan.

**Previsualizaciones.** Define `NUAGE_BRANCH` junto a `NUAGE_URL` en una aplicación de previsualización (una review app de Heroku, una aplicación de staging): al arrancar, Paquets crea la rama a partir de la base principal si todavía no existe, y luego ejecuta todas las consultas en ella. La previsualización trabaja con datos reales y no puede tocar los de producción.

```text
NUAGE_URL=https://mi-nuage.herokuapp.com
NUAGE_TOKEN=…
NUAGE_BRANCH=preview-42
```

Reinicia o borra una rama desde la consola (**Branches**), o en el código, con el cliente de `Db::Nuage`:

```rust
if let Db::Nuage(nuage) = req.db() {
    nuage.reset_branch("preview-42").await?; // olvida sus cambios
    nuage.delete_branch("preview-42").await?;
}
```

**Pruebas.** `app.test()` usa una base en memoria: rápida, y la opción correcta para la mayoría de las pruebas. Para probar con una copia de datos reales, dale a una prueba su propia rama:

```rust
use paquets::orm::NuageDb;

#[tokio::test]
async fn con_una_copia_de_produccion() -> paquets::orm::Result<()> {
    let (Ok(url), Ok(token)) = (std::env::var("NUAGE_URL"), std::env::var("NUAGE_TOKEN")) else {
        return Ok(()); // sin servidor: se omite
    };
    let server = NuageDb::connect(&url, &token).await?;
    let name = format!("test-{}", std::process::id());
    server.create_branch(&name, None).await?;

    let mut app = app();
    app.database(Db::from(server.on_branch(&name)));
    let client = app.test().await;
    assert_eq!(client.get("/").await.status(), 200);

    server.delete_branch(&name).await?;
    Ok(())
}
```

## La consola y el panel

El servidor que desplegaste tiene dos páginas, protegidas por `NUAGE_ADMIN_PASSWORD`:

- **`/admin`**, la consola: una vista general (tamaño, tablas, filas, ramas, tiempo en marcha, conexiones, caché, último punto de control, copias de seguridad), cada tabla y sus filas página a página, una consola de consultas (un pequeño subconjunto de SQL, con una confirmación antes de los cambios), las ramas, y el almacenamiento, con botones para un punto de control, una copia de seguridad, una comprobación de integridad y una compactación. La genera el servidor y funciona sin JavaScript, también en un teléfono.
- **`/app`**, el panel en directo: peticiones por segundo, latencia, conexiones, almacenamiento, caché, registro de escritura, copias de seguridad, bases y ramas, y un registro de eventos en directo, actualizado cada segundo. Se instala como una aplicación en un teléfono.

`GET /api/health` responde `{"status":"ok"}` a los monitores de disponibilidad. Con `NUAGE_PUBLIC_STATUS=true`, `/app` muestra una parte pública sin iniciar sesión.

## Desde el navegador

El cliente también funciona en WebAssembly, en una isla, con el `fetch` del navegador. Añade la feature `nuage` al lado del navegador de `Cargo.toml`:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client", "nuage"] }
```

`paquets::orm::NuageDb` queda disponible en `src/lib.rs`, con los mismos modelos y consultas.

> [!WARNING]
> El token abre toda la base. Envíalo al navegador solo en una aplicación para ti o tu equipo (un panel interno detrás de un inicio de sesión), nunca en una página pública: las páginas públicas leen y escriben a través de la API de tu servidor, que guarda el token en el servidor.

## Pasar un proyecto existente a Nuage DB

Los proyectos creados antes de la 0.3.3 no cambian: sin `NUAGE_URL` y sin `app.nuage(...)`, la base sigue viniendo de `DATABASE_URL`, y está en memoria sin ella. Para pasar a Nuage DB:

1. En `src/main.rs`, después de `App::new()`, añade `app.nuage("data");`.
2. Añade `/data` a `.gitignore` y `data` a `.dockerignore`.
3. En el `Dockerfile`, antes de `USER app`, añade `RUN mkdir data && chown app data`, para que la aplicación pueda escribir en la carpeta cuando `NUAGE_URL` no está definida.
4. Despliega un servidor Nuage DB, y define `NUAGE_URL` y `NUAGE_TOKEN` en la aplicación.

Para copiar los datos existentes, ejecuta un programa puntual, o un [seed](database.md#datos-al-arrancar), que lea de la base antigua y escriba en la nueva:

```rust
let old = Db::connect(&std::env::var("OLD_DATABASE_URL").unwrap()).await?;
let token = std::env::var("NUAGE_TOKEN").ok();
let new = Db::nuage(&std::env::var("NUAGE_URL").unwrap(), token.as_deref()).await?;
new.push::<Todo>().await?;
let todos = Todo::query().all(&old).await?;
Todo::insert_many(&new, &todos).await?;
```

La feature `nuage` está activada por defecto. Un proyecto que no la usa puede quitarla con `default-features = false` (ver [Instalación](installation.md#features-de-cargo)).

## Límites

- **Sin SQL.** Nuage DB ejecuta las instrucciones de Memoire: `db.raw(...)` y los pasos `.sql(...)` de las migraciones responden «no soportado», como con la base en memoria. Para SQL escrito a mano, usa PostgreSQL con `DATABASE_URL`.
- **Un escritor a la vez.** Las escrituras se aplican una tras otra, las lecturas en paralelo; un servidor por base.
- **El modo local es para desarrollar**: mantiene toda la base en memoria mientras la aplicación se ejecuta, y no tiene ramas, ni consola, ni panel (ejecuta un servidor para ellos).
- **Los joins, `GROUP BY` y los agregados** distintos de `COUNT(*)` leen en memoria las filas que combinan, después de los filtros de cada tabla.

La guía completa de Nuage DB, con sus copias de seguridad, sus transacciones y todos sus ajustes, está en [la documentación de Memoire](https://memoire-7f4525f1fdaf.herokuapp.com/es/docs/nuage/) y en el [repositorio Nuage](https://github.com/maxlestage/Nuage).

## Próximos pasos

- [Base de datos](database.md): modelos, consultas, transacciones y las otras bases.
- [Configuración](configuration.md): todas las variables de entorno.
- [Desplegar en Heroku](deploy-heroku-mobile.md): tu aplicación, desde tu teléfono.
