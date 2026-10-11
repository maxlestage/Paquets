# Installation

This page installs everything you need to build Paquets apps: Rust, the `paquets` command line tool, and (only if you use islands) two WebAssembly tools. It also shows how to add Paquets to an existing project without the command line tool.

## Requirements

| Tool | Needed for | Version |
|---|---|---|
| Rust and Cargo | everything | 1.88 or newer (Paquets uses the 2024 edition) |
| A C compiler | the SQLite database, which is compiled into your program | any (`cc`, `gcc`, `clang` or MSVC) |
| `wasm32-unknown-unknown` target | islands only | installed by `paquets dev` |
| `wasm-bindgen-cli` | islands only | the exact version of the `wasm-bindgen` crate in your `Cargo.lock`, installed by `paquets dev` |
| Git | creating a project, deploying | any |
| Docker | building the production image on your computer (optional) | any |

You don't need Node.js, npm, OpenSSL or a database server.

## 1. Install Rust

If Rust is not installed yet, use [rustup](https://rustup.rs), the official installer:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run `rustup-init.exe` from https://rustup.rs instead. It also offers to install the Microsoft C++ build tools, which Rust needs on Windows.

Open a new terminal, then check the installation:

```sh
rustc --version   # rustc 1.88.0 or newer
cargo --version
```

If Rust was already installed, update it with `rustup update`.

> [!TIP]
> On Linux, if a build later fails with `linker 'cc' not found`, install a C toolchain: `sudo apt install build-essential` on Debian and Ubuntu, `sudo dnf install gcc` on Fedora. On macOS, run `xcode-select --install`.

## 2. Install the paquets command

The `paquets` command creates projects, runs them and builds them for production. Install it with Cargo:

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

`--locked` builds the tool with the exact dependency versions it was tested with.

The first installation takes a few minutes, because Cargo compiles the tool and its dependencies. Then check it:

```sh
paquets --version
paquets doctor
```

`paquets doctor` lists the tools it finds and tells you how to install the missing ones:

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

Only `cargo` is required. The other lines are information. See [Command line tool](cli.md#paquets-doctor) for details.

## 3. The tools for islands (optional)

[Islands](islands.md) are the parts of a page that run in the browser, compiled to WebAssembly. They need two more tools. You don't have to install them yourself: the first `paquets dev` in a project with islands installs whatever is missing. To install them in advance:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

The version of `wasm-bindgen-cli` must be **exactly** the version of the `wasm-bindgen` crate in your project's `Cargo.lock` (`0.2.129` at the time of writing). `paquets doctor`, run inside a project, tells you which version the project needs.

If you only build pages, APIs and WebSocket routes, skip this step: a project created with `paquets new --minimal` has no islands and needs no WebAssembly tool.

## 4. Create your first project

```sh
paquets new my-app
cd my-app
paquets dev
```

Open http://localhost:3000. The [Quick start](quick-start.md) continues from here.

## Adding Paquets to an existing project

You can also use Paquets without the command line tool, like any other crate. Add it to `Cargo.toml`:

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }

# For the tests (`#[tokio::test]`).
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Add Paquets to `Cargo.toml`:

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }
```

Then replace `src/main.rs` with a first app:

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Home").body(h1().text("Hello 👋")) });
    app.run()
}
```

and start it with `cargo run`. That is all a project needs: no `#[tokio::main]`, no other dependency. To add islands to such a project, see [Islands](islands.md#setting-up-a-project-by-hand).

## Cargo features

Features turn optional parts of Paquets on or off. The defaults suit most apps:

| Feature | Default | Adds |
|---|---|---|
| `sqlite` | ✓ | SQLite databases (`DATABASE_URL=sqlite://app.db`), compiled in: nothing to install |
| `postgres` | ✓ | PostgreSQL with TLS (Heroku Postgres, Neon, Supabase…) |
| `mysql` | | MySQL and MariaDB, with TLS |
| `redis` | | Redis (rows stored as JSON documents), with `rediss://` TLS |
| `mongodb` | | MongoDB |
| `chrono` | | date and time columns (`chrono::NaiveDateTime`, `DateTime<Utc>`…) |
| `uuid` | | `uuid::Uuid` columns |
| `json` | | JSON columns (`serde_json::Value`, `Json<T>`) |
| `encryption` | | in-memory databases kept in an encrypted file (see [Database](database.md#an-in-memory-database-kept-in-a-file)) |
| `tls` | | HTTPS in the server itself: `app.tls(...)` (see [Configuration](configuration.md#https-in-the-server)) |
| `http2` | | HTTP/2, negotiated with `tls`, and in clear text (h2c) |
| `http3` | | HTTP/3 over QUIC, next to HTTP/1.1 |
| `session` | | signed cookies and server-side sessions (see [Security](security.md#accounts-and-sessions)) |
| `templates` | | HTML templates with minijinja: `app.views("views")`, `req.render(...)` |
| `multipart` | | file uploads: `req.multipart()` (see [Requests and responses](requests-responses.md#file-uploads)) |
| `compress` | | Brotli or gzip responses: `middleware::compress()` (see [Performance](performance.md#compression)) |
| `tower` | | tower layers as middlewares: `middleware::layer(...)` |
| `full` | | everything above |
| `client` | | the browser side of islands (for the `wasm32-unknown-unknown` target only) |
| `browser-storage` | | in islands, a state store and a database kept in `localStorage` (see [Islands](islands.md#keeping-data-in-the-browser)) |

For example, to use MySQL and dates:

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", features = ["mysql", "chrono"] }
```

To make your program smaller and faster to compile when you don't need SQLite or PostgreSQL, turn the defaults off and keep only what you use. The in-memory database is always available:

```toml
[dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["postgres"] }
```

## Editor

Any editor works. For completion, inline errors and "go to definition", install [rust-analyzer](https://rust-analyzer.github.io), available for VS Code, Zed, Neovim, Helix, Emacs and JetBrains IDEs. Paquets has no template language and no macros in views, so rust-analyzer understands all of your code, pages included.

## Troubleshooting

| Problem | Solution |
|---|---|
| `paquets: command not found` | Cargo installs commands in `~/.cargo/bin`. Open a new terminal, or add that folder to your `PATH`. |
| `linker 'cc' not found` | Install a C toolchain (see the tip in step 1). |
| `error: package ... requires rustc 1.88` or an edition 2024 error | Your Rust is too old: `rustup update`. |
| The first build takes several minutes | Normal: Cargo compiles every dependency once. Later builds only recompile your code. |
| `wasm-bindgen` reports a version or schema mismatch | The CLI and the crate differ: `cargo install wasm-bindgen-cli --version <version in Cargo.lock> --locked`. `paquets doctor` shows both versions. |

## Next steps

- [Quick start](quick-start.md): build a small app step by step.
- [Command line tool](cli.md): every `paquets` command.
- [Project structure](project-structure.md): what `paquets new` creates.
