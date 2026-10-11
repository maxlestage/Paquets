# Command line tool

The `paquets` command creates projects, runs them while you develop, builds them for production and checks your tools. It is a thin layer over Cargo: everything it does, you can also do by hand, and this page shows how.

## Installation

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

Install it with cargo:

```sh
cargo install --git https://github.com/maxlestage/Paquets paquets --locked
```

See [Installation](installation.md) for Rust itself and the optional tools.

## The commands

| Command | Does |
|---|---|
| `paquets new <name>` | creates a complete project: server, pages, a model and its API, an island, tests, `Dockerfile`, Heroku files, GitHub workflows |
| `paquets new <name> --minimal` | the same, with a single `src/main.rs` and no islands |
| `paquets dev` | builds the islands (debug), then runs the app with `cargo run` |
| `paquets build` | release build: the islands in `pkg/`, then the server with `cargo build --release` |
| `paquets doctor` | checks Cargo, the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` and its version, Git and Docker |
| `paquets help` | the list of commands |
| `paquets --version` | the version of the tool |

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

Documentation: https://maxlestage.github.io/Paquets/
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

It creates a folder named after the project with every file described in [Project structure](project-structure.md): `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `public/favicon.svg`, `Dockerfile`, `heroku.yml`, `app.json`, `.github/workflows/ci.yml`, `.github/workflows/heroku.yml`, `README.md`, `.gitignore` and `.dockerignore`. Then it runs `git init` with a `master` branch (the branch the CI merges into), unless Git is missing or the folder is already inside a Git repository.

| Option | Effect |
|---|---|
| `--minimal` | one `src/main.rs`, no `src/lib.rs`, no islands: no WebAssembly tool needed (11 files) |
| `--path <dir>` | the project uses a local checkout of Paquets (`paquets = { path = "<dir>" }`) instead of the published one: for working on Paquets itself |

The name must be a valid package name: lowercase letters, digits, `-` and `_`, starting with a letter (`my-app`, `blog2`, `todo_list`). Some names are reserved (`paquets`, `test`, `std`, `core`…). The folder must not exist yet, or be empty:

```text
error: `My-App` is not a valid name: use lowercase letters, digits, `-` and `_`, starting with a letter (e.g. my-app)
error: `my-app` already exists and is not empty
```

The page title of the new app comes from the name: `my-app` becomes "My App".

## paquets dev

```sh
paquets dev
```

Run it in the project folder. It:

1. builds the islands, if the project has some (a `src/lib.rs` and `crate-type = ["cdylib", ...]` in `Cargo.toml`):
   - installs the `wasm32-unknown-unknown` target if it is missing (once);
   - compiles `src/lib.rs` to WebAssembly, in debug mode (fast to compile);
   - installs `wasm-bindgen-cli` at the version of `wasm-bindgen` in `Cargo.lock` if it is missing or different (once, a few minutes);
   - writes the bundle to `pkg/` with `wasm-bindgen --target web`;
2. runs `cargo run`.

If the islands can't be built, it prints a warning and runs the app anyway: pages work, the islands just stay static until the problem is fixed (`paquets doctor` helps).

Arguments after `--` go to your program: `paquets dev -- --some-flag`. Environment variables work as usual: `DATABASE_URL=sqlite://app.db paquets dev`.

`paquets dev` doesn't watch your files: after a change, stop the server (`Ctrl+C`) and run it again. To restart automatically on every change, use a file watcher such as [cargo-watch](https://crates.io/crates/cargo-watch): `cargo watch -s "paquets dev"`.

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

It builds the islands in release mode, with the `wasm-release` profile of `Cargo.toml` when there is one (optimised for size), then the server with `cargo build --release`. The program is in `target/release/`. To run it, launch it from a folder that contains `public/` and `pkg/`:

```sh
./target/release/my-app
```

`CARGO_TARGET_DIR` is respected, if you set it. The `Dockerfile` of a project runs the same steps (see [Docker](deploy-docker.md)).

## paquets doctor

```sh
paquets doctor
```

```text
📦 Paquets 0.1.0

  ✓ cargo 1.97.0 (c980f4866 2026-06-30)
  ✓ wasm32-unknown-unknown target
  ✓ wasm-bindgen 0.2.129 (matches Cargo.lock)
  ✓ git version 2.43.0
  ✗ docker (optional)
      → only to build the production image locally

Ready. Create an app with `paquets new my-app`.
```

| Check | Needed for | When it is missing |
|---|---|---|
| `cargo` | everything | the only required tool: install Rust from https://rustup.rs |
| `wasm32-unknown-unknown` target | islands | `rustup target add wasm32-unknown-unknown`, or let `paquets dev` do it |
| `wasm-bindgen` | islands | `cargo install wasm-bindgen-cli --version <version> --locked`, or let `paquets dev` do it |
| `git` | creating projects, deploying | https://git-scm.com |
| `docker` | building the production image on your computer | optional |

Run inside a project, it also compares the installed `wasm-bindgen` with the version in `Cargo.lock`, and tells you the exact command when they differ:

```text
  ✗ wasm-bindgen 0.2.100, Cargo.lock wants 0.2.129
      → cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

## Without the command line tool

Every command is a shortcut for standard tools:

| Command | By hand |
|---|---|
| `paquets dev` | `cargo build --lib --target wasm32-unknown-unknown` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/debug/my_app.wasm` + `cargo run` |
| `paquets build` | `cargo build --lib --target wasm32-unknown-unknown --profile wasm-release` + `wasm-bindgen --target web --no-typescript --out-dir pkg target/wasm32-unknown-unknown/wasm-release/my_app.wasm` + `cargo build --release` |
| `paquets dev` in a minimal project | `cargo run` |

The `.wasm` file is named after the package, with `-` replaced by `_`. In CI, these commands avoid installing the `paquets` tool.

## Next steps

- [Quick start](quick-start.md): your first project, step by step.
- [Project structure](project-structure.md): every file `paquets new` creates.
- [Islands](islands.md): what `paquets dev` builds.
