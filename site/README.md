# The Paquets website

The website and documentation hub of [Paquets](https://maxlestage.github.io/Paquets/),
written **with Paquets**: the server is a `paquets::App`, the pages are `paquets::Page`s
built with [active](https://maxlestage.github.io/Active/) views, and the interactive
parts are active islands compiled to WebAssembly. English, French and Spanish.

| Part | Where |
|---|---|
| Server, pages, docs, static export | `src/server/` (native) |
| Islands: docs search, code tabs, typing terminal | `src/islands.rs` (server and browser) |
| Browser bundle: hydration and page motion | `src/client/` (wasm32) |
| Texts in three languages | `src/i18n.rs` (a missing translation does not compile) |
| Styles, logo, Open Graph image | `public/` (served at `/`) |
| Documentation pages | `../docs/{en,fr,es}/{slug}.md` (index in `src/catalog.rs`) |

## Routes

| Path | Page |
|---|---|
| `/` | redirects to `/en/`, `/fr/` or `/es/` (`Accept-Language`) |
| `/{lang}/` | home page |
| `/{lang}/docs/` | documentation hub (5 categories) |
| `/{lang}/docs/{slug}/` | a documentation page (English text while a translation is missing) |
| `/search/{lang}.txt` | search index, downloaded once by the search island |
| `/sitemap.xml`, `/robots.txt` | for search engines |
| `/demo`, `/api/todos` | the live demo of `examples/todo.rs` (server only) |
| `/site.css`, `/assets/…`, `/pkg/…` | static files and the browser bundle |

## Build and run

Everything runs from this folder (`site/`): Paquets serves `public/` and `pkg/` from the
current directory.

```bash
# 1. The browser bundle (islands + motion) → pkg/
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/paquets_site.wasm

# 2. The server, on http://localhost:3000 (PORT on Heroku)
cargo run --release
PORT=4000 cargo run --release
```

`wasm-bindgen-cli` must match the `wasm-bindgen` version of `Cargo.toml` (`0.2.129`):
`cargo install wasm-bindgen-cli --version 0.2.129`. Without `pkg/`, the site still works,
without the islands and the animations driven by WebAssembly.

Environment: `PORT`, `DATABASE_URL` (the demo's database, in memory by default),
`SITE_URL` (public address for canonical and Open Graph URLs; taken from each request
otherwise), `BASE_PATH`, `DOCS_DIR` (the Markdown pages; `../docs` by default, then
`docs/` in the current directory, e.g. in a Docker image).

## Static export (GitHub Pages)

```bash
BASE_PATH=/Paquets SITE_URL=https://maxlestage.github.io/Paquets \
  cargo run --release -- export dist
```

Every page is rendered by the app itself through Paquets' in-memory test client
(`app.test()`), then written to `dist/<path>/index.html`, with `404.html`, `sitemap.xml`,
`robots.txt`, the search indexes, `public/` and `pkg/`. Every link is behind `BASE_PATH`.
`/` is the English home page (no `Accept-Language` on a static host), and the live demo
is left out. Build `pkg/` first.

## Checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings
cargo test
```

The tests cover the texts (every string filled in and really translated, placeholders,
code comments), the routes, the Markdown rendering and link rewriting, every page of the
site through `app.test()` (status, `lang`, canonical and `hreflang` URLs, CSP, islands),
the redirections, the search index, the sitemap, the demo, the static export behind
`/Paquets`, and the documentation itself (`tests/docs.rs`: every page of the index
exists in `en`, `fr` and `es`, has a title and sections, and every link between pages
leads to a page and an anchor that exist).
