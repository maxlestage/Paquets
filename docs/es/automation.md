# Automatización

El repositorio de Paquets, y todo proyecto creado con `paquets new`, viene con workflows de GitHub que hacen el trabajo repetitivo: prueban cada push, **fusionan automáticamente en `master` cada rama en verde** y luego despliegan `master` en Heroku (y, en el repositorio de Paquets, publican el sitio web en GitHub Pages). Esta página explica cada workflow, las reglas de la fusión automática y cómo desactivar cualquier parte.

## La visión de conjunto

```text
push de una rama ──▶ CI: formato, Clippy, pruebas…
                       │
                       ├── rojo  ──▶ no pasa nada más: corrige la rama y vuelve a hacer push
                       │
                       └── verde ──▶ la rama se fusiona en master (salvo que se llame wip/…)
                                       │
                                       ├──▶ Deploy to Heroku   (si HEROKU_API_KEY y HEROKU_APP_NAME están definidos)
                                       └──▶ Pages              (repositorio de Paquets: el sitio web)
```

Nada llega a `master`, ni a producción, sin pasar las pruebas. Y nunca tienes que abrir, revisar ni fusionar tú una pull request, lo que hace posible todo el ciclo desde un teléfono.

## Los workflows

| Archivo | Nombre en la pestaña Actions | Se ejecuta | Hace |
|---|---|---|---|
| `.github/workflows/ci.yml` | **CI** | en cada push y cada pull request, y a mano | lo comprueba todo y luego fusiona la rama en `master` |
| `.github/workflows/heroku.yml` | **Deploy to Heroku** | después de cada fusión automática, en los push a `master`, a mano | despliega `master` en Heroku (consulta [Heroku](deploy-heroku-mobile.md#4-despliegues-automáticos-con-github-actions)) |
| `.github/workflows/pages.yml` | **Pages** | después de cada fusión automática, en los push a `master` que modifican el sitio o la documentación, a mano | publica el sitio web en GitHub Pages (solo en el repositorio de Paquets) |
| `.github/workflows/release.yml` | **Release** | cuando se publica una release de GitHub, a mano | publica los crates en crates.io (solo en el repositorio de Paquets; consulta [Publicar en crates.io](publish-crates-io.md)) |

Un proyecto creado con `paquets new` tiene los dos primeros: `ci.yml` y `heroku.yml`.

## CI: qué se comprueba

### En un proyecto creado con paquets new

Un job `test` y luego la fusión:

| Paso | Comando |
|---|---|
| formato | `cargo fmt --check` |
| lints, servidor | `cargo clippy --all-targets -- -D warnings` |
| lints, islas (si existe `src/lib.rs`) | `cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings` |
| pruebas | `cargo test` |

### En el repositorio de Paquets

Cuatro jobs se ejecutan en paralelo, y la fusión espera a todos:

| Job | Comprueba |
|---|---|
| **Crate** | el formato; Clippy en el servidor con las features por defecto y con todas las features; Clippy en el lado navegador (`wasm32`, feature `client`); las pruebas con las features por defecto y con todas las features; la documentación de la API; los paquetes de crates.io (`paquets-macros` por completo, y la lista de archivos de `paquets`) |
| **MSRV** | el crate compila con Rust 1.88, la versión más antigua compatible |
| **paquets new** | genera un proyecto completo y uno mínimo con la herramienta de línea de comandos y ejecuta sus propias comprobaciones (formato, Clippy, pruebas y el Clippy de las islas para wasm32) |
| **Website** | el sitio web (una aplicación Paquets en `site/`): formato, Clippy (servidor y WebAssembly), sus pruebas (que incluyen todas las páginas de la documentación, en los tres idiomas, con sus enlaces), la compilación de sus islas y la exportación estática para GitHub Pages |

Cualquier comprobación en rojo lo detiene todo: la rama no se fusiona.

Para ejecutar las comprobaciones principales en tu ordenador antes de hacer push:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings   # proyectos con islas
cargo test
```

## La fusión automática

Cuando todas las comprobaciones de un push están en verde, el job **Merge into master** fusiona la rama en `master`:

- solo para los **push** (no para las pull requests que vienen de forks), en cualquier rama salvo la propia `master`;
- **salvo las ramas cuyo nombre empieza por `wip/`** (`wip/new-design`, `wip/experiment`): se prueban, pero nunca se fusionan automáticamente. Úsalas para el trabajo en curso;
- con un commit de fusión, `Merge branch '<branch>' into master (auto-merge)`, hecho por `github-actions[bot]`;
- si la rama ya está en `master`, no pasa nada;
- las fusiones nunca se ejecutan a la vez: se esperan unas a otras.

Después de la fusión, el propio job lanza los workflows de despliegue (`gh workflow run heroku.yml`, y `pages.yml` en el repositorio de Paquets). Es necesario porque GitHub no lanza workflows para un push hecho por un workflow.

### Cuando la fusión falla

Si `master` ha cambiado de una forma que entra en conflicto con tu rama, el job de fusión falla y `master` queda intacta. Trae `master` a tu rama, resuelve los conflictos y haz push: la CI se vuelve a ejecutar y fusiona. Desde un teléfono, el sitio web de GitHub puede hacerlo en una pull request (**Update branch** y luego **Resolve conflicts**).

Si se rechaza el push a `master` (`403` o «protected branch»), consulta [Protección de ramas](#protección-de-ramas) más abajo.

## GitHub Pages: el sitio web

El sitio web de Paquets (en `site/`) es una aplicación Paquets: páginas renderizadas en el servidor con Active, islas en WebAssembly y una demo en vivo. Para GitHub Pages, que solo sirve archivos, el workflow **Pages** compila las islas y luego exporta todas las páginas a archivos estáticos con el propio código de servidor del sitio:

```sh
cd site
BASE_PATH=/Paquets cargo run --release -- export dist
```

`BASE_PATH` añade como prefijo a cada enlace la ruta del repositorio, porque GitHub Pages sirve el sitio en `https://<owner>.github.io/<repository>/`. El workflow sube `site/dist` y lo publica.

Configuración inicial, una sola vez, que se puede hacer desde un teléfono: **Settings** → **Pages** → **Build and deployment** → **Source**: **GitHub Actions**. Mientras tanto, el workflow se salta con un aviso en lugar de fallar. Para publicar enseguida: **Actions** → **Pages** → **Run workflow**.

El mismo sitio, con su demo en vivo y su API REST, es lo que despliega el botón **Deploy to Heroku** (consulta [Heroku](deploy-heroku-mobile.md)).

## Sin Dependabot

Los repositorios no tienen configuración de Dependabot: ningún bot abre pull requests para actualizar las dependencias. Como cada rama en verde se fusiona y se despliega automáticamente, la actualización de un bot llegaría a producción sin que nadie lo decidiera. Las dependencias solo cambian cuando alguien ejecuta `cargo update` y sube el nuevo `Cargo.lock` en una rama, que la CI prueba como cualquier otro cambio.

Para comprobar de vez en cuando si hay dependencias desactualizadas, ejecuta `cargo update --dry-run` (o [`cargo outdated`](https://crates.io/crates/cargo-outdated)) en tu ordenador.

## Permisos

El job de fusión pide dos permisos en el archivo del workflow: `contents: write` (para hacer push a `master`) y `actions: write` (para lanzar los despliegues). Funcionan con los ajustes por defecto de GitHub. Si tu organización restringe los permisos de los workflows, revisa **Settings** → **Actions** → **General** → **Workflow permissions**.

### Protección de ramas

Si `master` está protegida por una regla que exige pull requests o revisiones, el push del bot se rechaza. Permite que GitHub Actions se salte la regla, o desactiva la fusión automática (ver más abajo) y fusiona tú las pull requests.

## Desactivarlo

| Para detener… | Haz esto |
|---|---|
| la fusión de una rama | llámala `wip/...` |
| la fusión automática por completo | elimina el job `merge:` al final de `.github/workflows/ci.yml`; las pruebas se siguen ejecutando |
| un workflow entero | **Actions** → el workflow → **⋯** → **Disable workflow** (sitio web de GitHub) |
| los despliegues en Heroku | elimina el secreto `HEROKU_API_KEY` o la variable `HEROKU_APP_NAME`: el workflow se salta entonces con un aviso |
| el sitio de GitHub Pages | **Settings** → **Pages**, despublica el sitio, o desactiva el workflow **Pages** |
| la publicación en crates.io | no publiques releases, o elimina el secreto `CARGO_REGISTRY_TOKEN` |

Los workflows desactivados se pueden volver a activar desde el mismo menú.

## Próximos pasos

- [Desplegar en Heroku desde el móvil](deploy-heroku-mobile.md): el secreto y la variable de los despliegues.
- [Publicar en crates.io](publish-crates-io.md): el workflow de publicación.
- [Pruebas](testing.md): escribe las pruebas que ejecuta la CI.
