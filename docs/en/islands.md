# Islands

An island is an interactive part of a server-rendered page: a counter, a search box, a quantity selector. The server renders it to HTML like the rest of the page. In the browser, a small WebAssembly bundle, compiled from the same Rust function, makes only that part interactive. The rest of the page stays plain HTML and costs nothing. This page shows how to write islands, place them in pages, build them and deploy them.

## How it works

```text
src/lib.rs ── fn counter(props) ──┬── on the server: rendered to HTML inside the page
                                  │
                                  └── paquets build ──▶ pkg/my_app.js + pkg/my_app_bg.wasm
                                                        loaded by the browser, which makes
                                                        the counter on the page interactive
```

1. The server calls the island's function and writes its HTML into the page, inside an `<active-island>` element that records its name and props.
2. Visitors see the content immediately, even before any script runs. Search engines see it too.
3. The browser loads the bundle, calls the same function with the same props, takes over the HTML that is already there (it doesn't redraw it) and attaches the event handlers.

Only pages that contain at least one island load the bundle. A counter island weighs about **36.8 kB** compressed, WebAssembly and loader included, the smallest among Rust front-end frameworks.

## Requirements

Islands are compiled to WebAssembly, which needs two tools:

- the `wasm32-unknown-unknown` target of Rust;
- `wasm-bindgen-cli`, at exactly the version of the `wasm-bindgen` crate in your `Cargo.lock`.

The first `paquets dev` in a project with islands installs whatever is missing (it takes a few minutes, once). `paquets doctor` shows what is installed. See [Installation](installation.md#3-the-tools-for-islands-optional).

## Your first island

A project created with `paquets new` (without `--minimal`) already has an island, `counter`, in `src/lib.rs`. Here is the whole cycle, with a simpler counter.

### 1. Write the island in src/lib.rs

An island is a function that takes its props as a string and returns a `Node`:

```rust
// src/lib.rs
use paquets::prelude::*;

/// A counter: the props are the starting value.
pub fn counter(props: &str) -> Node {
    let count = use_state(props.parse::<i32>().unwrap_or(0));
    button()
        .on_click(move |_| count.update(|n| *n += 1))
        .text_dyn(move || format!("Clicked {} times", count.get()))
        .into()
}
```

- `use_state(initial)` creates a state: a value that can change in the browser.
- `.on_click(...)` runs code when the button is clicked. Here it adds 1 to the state.
- `.text_dyn(...)` shows a text computed from the state. When the state changes, only that text is updated in the page.

The island must live in `src/lib.rs` (or a module of the library), because the browser bundle is built from the library.

### 2. Register it for the browser

At the end of `src/lib.rs`, the browser entry point lists every island, by name:

```rust
/// The browser entry point: makes every island of the page interactive.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[wasm_bindgen(start)]` runs `start` as soon as the bundle is loaded. `#[cfg(target_arch = "wasm32")]` keeps this function out of the server build.

### 3. Place it in a page

In `src/main.rs`, import the island and place it with `island(name, props, function)`:

```rust
// src/main.rs
use paquets::prelude::*;

use my_app::counter; // `my_app` is your package name, with `_` instead of `-`

async fn home(_req: Request) -> Page {
    Page::new("Home")
        .body(h1().text("A counter"))
        .body(island("counter", "0", counter))
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", home);
    app.run()
}
```

The name (`"counter"`) must be the same here and in `start()`. The props (`"0"`) are the starting value.

### 4. Run it

```sh
paquets dev
```

`paquets dev` compiles `src/lib.rs` to WebAssembly (in debug mode), writes the bundle to `pkg/`, then runs the server. The startup line shows the bundle it found:

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: /pkg/my_app.js
```

Open the page and click the button. After changing an island, run `paquets dev` again: `cargo run` alone rebuilds the server, not the bundle.

## State, events and dynamic views

### State

| Method | Use it to |
|---|---|
| `use_state(initial)` | create a state; the handle is `Copy`, move it into as many closures as you need |
| `state.get()` | read a copy of the value |
| `state.with(\|v\| ...)` | read through a reference, without copying (`text.with(String::len)`) |
| `state.set(value)` | replace the value |
| `state.update(\|v\| ...)` | change the value in place (`*n += 1`, `list.push(...)`) |

### Events

| Method | Event |
|---|---|
| `.on_click(\|e\| ...)` | a click |
| `.on_input(\|e\| ...)` | the value of a field changes, at each keystroke |
| `.on("keydown", \|e\| ...)` | any DOM event, by name: `"change"`, `"submit"`, `"keydown"`, `"focus"`… |

The event `e` gives `e.value()` (the value of the field), `e.key()` (the key of a keyboard event) and `e.prevent_default()`.

### Dynamic views

Methods that take a closure instead of a value follow the state: the closure runs again when a state it reads changes, and only that part of the page is updated.

| Method | Updates |
|---|---|
| `.text_dyn(f)` | a text |
| `.attr_dyn(name, f)` | an attribute (`value` also updates a field) |
| `.class_if(class, f)` | adds a class while `f()` is `true` |
| `.bool_attr(name, f)` | a boolean attribute: `disabled`, `hidden`, `checked`… |
| `.children_dyn(f)` | a list of children, rebuilt when it changes |

A text field that counts its characters:

```rust
/// A text field that counts its characters. The props are the maximum.
pub fn limited_text(props: &str) -> Node {
    let max: usize = props.parse().unwrap_or(280);
    let text = use_state(String::new());
    let count = move || text.with(|t| t.chars().count());
    div()
        .child(textarea().attr("name", "message").on_input(move |e| text.set(e.value())))
        .child(
            p().class("muted")
                .class_if("too-long", move || count() > max)
                .text_dyn(move || format!("{} / {max}", count())),
        )
        .into()
}
```

Place it with `island("limited-text", "140", limited_text)` and add `("limited-text", limited_text)` to the list in `start()`.

## Props

Props travel inside the page's HTML, as a string. Use a number or a word for simple cases, and JSON for structured data:

```rust
// src/lib.rs
/// The props of `add_to_cart`, sent as JSON.
#[data]
#[derive(Default)]
pub struct CartProps {
    pub product: String,
    pub price_cents: u32,
}

/// A quantity field and the total price.
pub fn add_to_cart(props: &str) -> Node {
    let props: CartProps = paquets::serde_json::from_str(props).unwrap_or_default();
    let price = props.price_cents;
    let quantity = use_state(1u32);
    div()
        .class("row")
        .child(strong().text(props.product))
        .child(
            input()
                .attr("type", "number")
                .attr("min", "1")
                .attr_dyn("value", move || quantity.get().to_string())
                .on_input(move |e| quantity.set(e.value().parse().unwrap_or(1))),
        )
        .child(span().text_dyn(move || {
            let total = quantity.get() * price;
            format!("{}.{:02} €", total / 100, total % 100)
        }))
        .into()
}
```

```rust
// src/main.rs
use my_app::{CartProps, add_to_cart};

async fn product(_req: Request) -> Page {
    let props = CartProps { product: "Oak chair".into(), price_cents: 24900 };
    Page::new("Oak chair")
        .body(h1().text("Oak chair"))
        .body(island("add-to-cart", paquets::serde_json::to_string(&props).unwrap(), add_to_cart))
}
```

Props are part of the HTML: keep them small, and never put secrets in them.

## The rules of islands

- **Everything comes from the props.** An island is a plain function (`fn(&str) -> Node`) that captures nothing. In the browser it has no access to your database or your server state: give it what it needs through its props. A view that depends on something only the server has (the time, a random number, a database value not passed in the props) produces a different HTML in the browser, which the browser then redraws, with a warning in its console: `active: re-rendering island (tag mismatch)`.
- **Islands are independent.** Each island has its own states. If two parts of a page must react to each other (a filter and a list), make them one island.
- **The same markup on both sides.** Use `#[cfg(target_arch = "wasm32")]` only for behaviour, never to change what is rendered.
- **Names must match.** An island whose name is not in `start()` simply stays static, without any error.
- **Data from the server.** To read or send data after the page has loaded, call your API with the browser's `fetch` through `web-sys` and `wasm-bindgen-futures` (see [Active's events guide](https://maxlestage.github.io/Active/en/docs/events/)), or use a plain HTML form that posts to a route.

## Building for production

```sh
paquets build
```

`paquets build` compiles the islands in release mode, with the size-optimised `wasm-release` profile, into `pkg/`, then builds the server with `cargo build --release`. Deploy `pkg/` (and `public/`) next to the program: the `Dockerfile` of a generated project already does it (see [Docker](deploy-docker.md)).

At startup, the app looks for `pkg/<name>.js` and `pkg/<name>_bg.wasm` in the current folder and serves them at `/pkg/`. Pages that contain islands then get three tags in their `<head>`: a preload of the module, a preload of the WebAssembly file (downloaded in parallel with the page), and the line that starts it. Their Content-Security-Policy allows WebAssembly. Pages without islands get none of this.

## Setting up a project by hand

A project created with `paquets new --minimal`, or an existing project, needs four changes to get islands.

**1. `Cargo.toml`**: one crate, two targets. The library is also compiled to WebAssembly (`cdylib`), and Paquets is used without its server and databases in the browser:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

# The browser bundle: as small as possible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

Remove `paquets` from the plain `[dependencies]` section: keep it only in the two target sections.

**2. `src/lib.rs`**: the islands and the `start()` function shown above.

**3. `src/main.rs`**: `use my_app::counter;` and `island(...)` in a page.

**4. `.gitignore`**: add `/pkg`.

Then `paquets dev` builds everything. Without the `paquets` tool, these are the commands it runs:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # the version of Cargo.lock
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/my_app.wasm
cargo run --release
```

The `.wasm` file is named after the package, with `-` replaced by `_`.

## Troubleshooting

| Problem | Solution |
|---|---|
| The page shows the island, but clicking does nothing | Check the startup line: `islands: none` means `pkg/` has no bundle; run `paquets dev`. Check that the name in `island(...)` is in the list of `start()`. Open the browser's console for errors. |
| `active: re-rendering island (… mismatch)` in the console | The server HTML and the browser view differ: pass every input through the props (see the rules above). |
| `wasm-bindgen` reports a version or schema mismatch | `cargo install wasm-bindgen-cli --version <version in Cargo.lock> --locked`, or let `paquets dev` do it. |
| Errors about tokio, sockets or `mio` when building for wasm32 | A server-only dependency is compiled for the browser: move it under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. |
| `hydrate` not found | The `client` feature is missing on the wasm32 `paquets` line, or the code is not behind `#[cfg(target_arch = "wasm32")]`. |
| Changes to an island don't show up | Rebuild the bundle with `paquets dev` (not just `cargo run`), then reload the page. |

## Learn more

Islands, states and events come from [Active](https://maxlestage.github.io/Active/). Its documentation goes further: [islands](https://maxlestage.github.io/Active/en/docs/islands/), [state and effects](https://maxlestage.github.io/Active/en/docs/state/), [events](https://maxlestage.github.io/Active/en/docs/events/), and [client-side apps](https://maxlestage.github.io/Active/en/docs/client-app/) (`paquets::mount_to`, `navigate` and `location` with the `client` feature).

## Next steps

- [Pages](pages.md): the rest of the page, rendered on the server.
- [Command line tool](cli.md): `paquets dev` and `paquets build` in detail.
- [Deploy with Docker](deploy-docker.md): how islands are built in the image.
