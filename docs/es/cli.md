# Herramienta de línea de comandos

El comando `paquets` crea proyectos, los ejecuta mientras desarrollas, los compila para producción y comprueba tus herramientas. Es una capa fina sobre Cargo: todo lo que hace también lo puedes hacer a mano, y esta página muestra cómo.

## Instalación

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

Instálalo con cargo:

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

Consulta [Instalación](installation.md) para instalar Rust y las herramientas opcionales.

## Los comandos

| Comando | Hace |
|---|---|
| `paquets new <name>` | crea un proyecto completo: servidor, páginas, un modelo y su API, una isla, pruebas, `Dockerfile`, archivos de Heroku, workflows de GitHub |
| `paquets new <name> --minimal` | lo mismo, con un único `src/main.rs` y sin islas |
| `paquets dev` | compila las islas (debug) y luego ejecuta la aplicación con `cargo run` |
| `paquets build` | compilación release: las islas en `pkg/` y luego el servidor con `cargo build --release` |
| `paquets doctor` | comprueba Cargo, el target `wasm32-unknown-unknown`, `wasm-bindgen-cli` y su versión, Git y Docker |
| `paquets help` | la lista de comandos |
| `paquets --version` | la versión de la herramienta |

```text
$ paquets help
📦 Paquets — the simplest full-stack framework for Rust

Usage:
  paquets new <name> [--minimal]   create a project (server, pages, API, islands,
                                   Dockerfile, Heroku, CI with auto-merge into master)
  paquets dev [-- <args>]          build the islands, then run the app (cargo run)
  paquets build                    release build: islands in pkg/, then the server
  paquets doctor                   check cargo, the wasm32 target and wasm-bindgen
  paquets help                     this help
  paquets --version                the version

Options of `new`:
  --minimal       one src/main.rs, no islands (no WebAssembly tools needed)
  --path <dir>    use a local checkout of Paquets instead of the published one

Documentation: https://paquets-90dfc06b6fea.herokuapp.com/
```

## paquets new

```sh
paquets new my-app
```

```text
📦 my-app is ready (12 files, git repository on master).

  cd my-app
  paquets dev        # http://localhost:3000

The first `paquets dev` also prepares the WebAssembly tools for the islands.
Deploy from your phone: push the project to GitHub, then see README.md.
```

Crea una carpeta con el nombre del proyecto que contiene todos los archivos descritos en [Estructura del proyecto](project-structure.md): `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `public/favicon.svg`, `Dockerfile`, `heroku.yml`, `app.json`, `.github/workflows/ci.yml`, `.github/workflows/heroku.yml`, `README.md`, `.gitignore` y `.dockerignore`. Después ejecuta `git init` con una rama `master` (la rama en la que fusiona la CI), salvo que falte Git o que la carpeta ya esté dentro de un repositorio Git.

| Opción | Efecto |
|---|---|
| `--minimal` | un solo `src/main.rs`, sin `src/lib.rs` ni islas: no hace falta ninguna herramienta de WebAssembly (11 archivos) |
| `--path <dir>` | el proyecto usa una copia local de Paquets (`paquets = { path = "<dir>" }`) en lugar de la publicada: para trabajar en el propio Paquets |

El nombre debe ser un nombre de paquete válido: letras minúsculas, números, `-` y `_`, empezando por una letra (`my-app`, `blog2`, `todo_list`). Algunos nombres están reservados (`paquets`, `test`, `std`, `core`…). La carpeta no debe existir todavía, o debe estar vacía:

```text
error: `My-App` is not a valid name: use lowercase letters, digits, `-` and `_`, starting with a letter (e.g. my-app)
error: `my-app` already exists and is not empty
```

El título de las páginas de la nueva aplicación se deriva del nombre: `my-app` se convierte en «My App».

## paquets dev

```sh
paquets dev
```

Ejecútalo en la carpeta del proyecto. Este comando:

1. compila las islas, si el proyecto las tiene (un `src/lib.rs` y `crate-type = ["cdylib", ...]` en `Cargo.toml`):
   - instala el target `wasm32-unknown-unknown` si falta (una sola vez);
   - compila `src/lib.rs` a WebAssembly, en modo debug (rápido de compilar);
   - instala `wasm-bindgen-cli` en la versión de `wasm-bindgen` de `Cargo.lock` si falta o es distinta (una sola vez, unos minutos);
   - escribe el bundle en `pkg/` con `wasm-bindgen --target web`;
2. ejecuta `cargo run`.

Si las islas no se pueden compilar, muestra un aviso y ejecuta la aplicación de todos modos: las páginas funcionan, y las islas simplemente se quedan estáticas hasta que se resuelva el problema (`paquets doctor` ayuda).

Los argumentos que van después de `--` se pasan a tu programa: `paquets dev -- --some-flag`. Las variables de entorno funcionan como siempre: `DATABASE_URL=sqlite://app.db paquets dev`.

`paquets dev` no vigila tus archivos: después de un cambio, detén el servidor (`Ctrl+C`) y vuelve a ejecutarlo. Para reiniciar automáticamente con cada cambio, usa un vigilante de archivos como [cargo-watch](https://crates.io/crates/cargo-watch): `cargo watch -s "paquets dev"`.

## paquets build

```sh
paquets build
```

```text
🏝️  Islands
  → cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
  → wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/wasm-release/my_app.wasm
  → cargo build --release
📦 Built: target/release/my-app
   Islands: pkg/ (deploy it next to the program, with public/)
```

Compila las islas en modo release, con el perfil `wasm-release` de `Cargo.toml` cuando existe (optimizado para el tamaño), y luego el servidor con `cargo build --release`. El programa queda en `target/release/`. Para ejecutarlo, lánzalo desde una carpeta que contenga `public/` y `pkg/`:

```sh
./target/release/my-app
```

`CARGO_TARGET_DIR` se respeta, si lo defines. El `Dockerfile` de un proyecto ejecuta los mismos pasos (consulta [Docker](deploy-docker.md)).

## paquets doctor

```sh
paquets doctor
```

```text
📦 Paquets 0.1.1

  ✓ cargo 1.97.0 (c980f4866 2026-06-30)
  ✓ wasm32-unknown-unknown target
  ✓ wasm-bindgen 0.2.129 (matches Cargo.lock)
  ✓ git version 2.43.0
  ✗ docker (optional)
      → only to build the production image locally

Ready. Create an app with `paquets new my-app`.
```

| Comprobación | Necesaria para | Si falta |
|---|---|---|
| `cargo` | todo | la única herramienta obligatoria: instala Rust desde https://rustup.rs |
| target `wasm32-unknown-unknown` | las islas | `rustup target add wasm32-unknown-unknown`, o deja que lo haga `paquets dev` |
| `wasm-bindgen` | las islas | `cargo install wasm-bindgen-cli --version <version> --locked`, o deja que lo haga `paquets dev` |
| `git` | crear proyectos, desplegar | https://git-scm.com |
| `docker` | construir la imagen de producción en tu ordenador | opcional |

Ejecutado dentro de un proyecto, también compara el `wasm-bindgen` instalado con la versión de `Cargo.lock` y, cuando no coinciden, te da el comando exacto:

```text
  ✗ wasm-bindgen 0.2.100, Cargo.lock wants 0.2.129
      → cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

## Sin la herramienta de línea de comandos

Cada comando es un atajo para herramientas estándar:

| Comando | A mano |
|---|---|
| `paquets dev` | `cargo build --lib --target wasm32-unknown-unknown` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/debug/my_app.wasm` + `cargo run` |
| `paquets build` | `cargo build --lib --target wasm32-unknown-unknown --profile wasm-release` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/wasm-release/my_app.wasm` + `cargo build --release` |
| `paquets dev` en un proyecto mínimo | `cargo run` |

El archivo `.wasm` lleva el nombre del paquete, con `-` reemplazado por `_`. En la CI, estos comandos evitan tener que instalar la herramienta `paquets`.

## Próximos pasos

- [Inicio rápido](quick-start.md): tu primer proyecto, paso a paso.
- [Estructura del proyecto](project-structure.md): todos los archivos que crea `paquets new`.
- [Islas](islands.md): lo que compila `paquets dev`.
