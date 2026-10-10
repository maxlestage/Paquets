# Instalación

Esta página instala todo lo que necesitas para crear aplicaciones con Paquets: Rust, la herramienta de línea de comandos `paquets` y (solo si usas islas) dos herramientas de WebAssembly. También muestra cómo añadir Paquets a un proyecto existente sin la herramienta de línea de comandos.

## Requisitos

| Herramienta | Necesaria para | Versión |
|---|---|---|
| Rust y Cargo | todo | 1.88 o posterior (Paquets usa la edición 2024) |
| Un compilador de C | la base de datos SQLite, que se compila dentro de tu programa | cualquiera (`cc`, `gcc`, `clang` o MSVC) |
| El target `wasm32-unknown-unknown` | solo las islas | lo instala `paquets dev` |
| `wasm-bindgen-cli` | solo las islas | exactamente la versión del crate `wasm-bindgen` de tu `Cargo.lock`, la instala `paquets dev` |
| Git | crear un proyecto, desplegar | cualquiera |
| Docker | construir la imagen de producción en tu ordenador (opcional) | cualquiera |

No necesitas Node.js, npm, OpenSSL ni un servidor de base de datos.

## 1. Instala Rust

Si Rust todavía no está instalado, usa [rustup](https://rustup.rs), el instalador oficial:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

En Windows, descarga y ejecuta en su lugar `rustup-init.exe` desde https://rustup.rs. También te ofrece instalar las herramientas de compilación de Microsoft C++, que Rust necesita en Windows.

Abre una terminal nueva y comprueba la instalación:

```sh
rustc --version   # rustc 1.88.0 o posterior
cargo --version
```

Si Rust ya estaba instalado, actualízalo con `rustup update`.

> [!TIP]
> En Linux, si más adelante una compilación falla con `linker 'cc' not found`, instala una toolchain de C: `sudo apt install build-essential` en Debian y Ubuntu, `sudo dnf install gcc` en Fedora. En macOS, ejecuta `xcode-select --install`.

## 2. Instala el comando paquets

El comando `paquets` crea proyectos, los ejecuta y los compila para producción. Instálalo con Cargo:

```sh
cargo install paquets --locked
```

> [!NOTE]
> Hasta que Paquets se publique en crates.io (consulta [Publicar en crates.io](publish-crates-io.md)), instálalo desde el repositorio de GitHub:
>
> ```sh
> cargo install --git https://github.com/maxlestage/Paquets paquets --locked
> ```

`--locked` compila la herramienta con las versiones exactas de las dependencias con las que se probó.

La primera instalación tarda unos minutos, porque Cargo compila la herramienta y sus dependencias. Después, compruébala:

```sh
paquets --version
paquets doctor
```

`paquets doctor` enumera las herramientas que encuentra y te dice cómo instalar las que faltan:

```text
📦 Paquets 0.1.0

  ✓ cargo 1.97.0 (c980f4866 2026-06-30)
  ✗ wasm32-unknown-unknown target (islands)
      → rustup target add wasm32-unknown-unknown (paquets dev does it for you)
  ✗ wasm-bindgen (islands)
      → cargo install wasm-bindgen-cli --locked (paquets dev does it for you)
  ✓ git version 2.43.0
  ✗ docker (optional)
      → only to build the production image locally

Ready. Create an app with `paquets new my-app`.
```

Solo `cargo` es obligatorio. Las demás líneas son informativas. Consulta [Herramienta de línea de comandos](cli.md#paquets-doctor) para más detalles.

## 3. Las herramientas para las islas (opcional)

Las [islas](islands.md) son las partes de una página que se ejecutan en el navegador, compiladas a WebAssembly. Necesitan dos herramientas más. No tienes que instalarlas tú: el primer `paquets dev` en un proyecto con islas instala lo que falte. Para instalarlas por adelantado:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

La versión de `wasm-bindgen-cli` debe ser **exactamente** la versión del crate `wasm-bindgen` del `Cargo.lock` de tu proyecto (`0.2.129` en el momento de escribir esto). `paquets doctor`, ejecutado dentro de un proyecto, te dice qué versión necesita el proyecto.

Si solo creas páginas, APIs y rutas WebSocket, sáltate este paso: un proyecto creado con `paquets new --minimal` no tiene islas y no necesita ninguna herramienta de WebAssembly.

## 4. Crea tu primer proyecto

```sh
paquets new my-app
cd my-app
paquets dev
```

Abre http://localhost:3000. El [Inicio rápido](quick-start.md) continúa a partir de aquí.

## Añadir Paquets a un proyecto existente

También puedes usar Paquets sin la herramienta de línea de comandos, como cualquier otro crate. Añádelo a `Cargo.toml`:

```toml
[dependencies]
paquets = "0.1"

# Para las pruebas (`#[tokio::test]`).
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Mientras Paquets no esté en crates.io, haz que Cargo apunte al repositorio:

```toml
[dependencies]
paquets = { version = "0.1", git = "https://github.com/maxlestage/Paquets" }
```

Después reemplaza `src/main.rs` por una primera aplicación:

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Inicio").body(h1().text("Hola 👋")) });
    app.run()
}
```

y arráncala con `cargo run`. Es todo lo que un proyecto necesita: sin `#[tokio::main]` ni ninguna otra dependencia. Para añadir islas a un proyecto así, consulta [Islas](islands.md#configurar-un-proyecto-a-mano).

## Features de Cargo

Las features activan o desactivan partes opcionales de Paquets. Las que vienen por defecto sirven para la mayoría de las aplicaciones:

| Feature | Por defecto | Añade |
|---|---|---|
| `sqlite` | ✓ | Bases de datos SQLite (`DATABASE_URL=sqlite://app.db`), compiladas dentro: nada que instalar |
| `postgres` | ✓ | PostgreSQL con TLS (Heroku Postgres, Neon, Supabase…) |
| `mysql` | | MySQL y MariaDB, con TLS |
| `redis` | | Redis (filas guardadas como documentos JSON), con TLS `rediss://` |
| `mongodb` | | MongoDB |
| `chrono` | | columnas de fecha y hora (`chrono::NaiveDateTime`, `DateTime<Utc>`…) |
| `uuid` | | columnas `uuid::Uuid` |
| `json` | | columnas JSON (`serde_json::Value`, `Json<T>`) |
| `http3` | | HTTP/3 sobre QUIC, junto a HTTP/1.1 |
| `full` | | todo lo anterior |
| `client` | | el lado navegador de las islas (solo para el target `wasm32-unknown-unknown`) |

Por ejemplo, para usar MySQL y fechas:

```toml
[dependencies]
paquets = { version = "0.1", features = ["mysql", "chrono"] }
```

Para que tu programa sea más pequeño y compile más rápido cuando no necesitas SQLite o PostgreSQL, desactiva las features por defecto y conserva solo lo que uses. La base de datos en memoria siempre está disponible:

```toml
[dependencies]
paquets = { version = "0.1", default-features = false, features = ["postgres"] }
```

## Editor

Cualquier editor sirve. Para tener autocompletado, errores en línea y «ir a la definición», instala [rust-analyzer](https://rust-analyzer.github.io), disponible para VS Code, Zed, Neovim, Helix, Emacs y los IDE de JetBrains. Paquets no tiene lenguaje de plantillas ni macros en las vistas, así que rust-analyzer entiende todo tu código, páginas incluidas.

## Solución de problemas

| Problema | Solución |
|---|---|
| `paquets: command not found` | Cargo instala los comandos en `~/.cargo/bin`. Abre una terminal nueva o añade esa carpeta a tu `PATH`. |
| `linker 'cc' not found` | Instala una toolchain de C (consulta el consejo del paso 1). |
| `error: package ... requires rustc 1.88` o un error sobre la edición 2024 | Tu Rust es demasiado antiguo: `rustup update`. |
| La primera compilación tarda varios minutos | Es normal: Cargo compila cada dependencia una vez. Las compilaciones siguientes solo recompilan tu código. |
| `wasm-bindgen` informa de una diferencia de versión o de esquema | La CLI y el crate no coinciden: `cargo install wasm-bindgen-cli --version <versión de Cargo.lock> --locked`. `paquets doctor` muestra las dos versiones. |

## Próximos pasos

- [Inicio rápido](quick-start.md): crea una pequeña aplicación paso a paso.
- [Herramienta de línea de comandos](cli.md): todos los comandos de `paquets`.
- [Estructura del proyecto](project-structure.md): lo que crea `paquets new`.
