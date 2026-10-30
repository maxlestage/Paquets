# Islas

Una isla es una parte interactiva de una página renderizada en el servidor: un contador, un cuadro de búsqueda, un selector de cantidad. El servidor la renderiza a HTML como el resto de la página. En el navegador, un pequeño bundle de WebAssembly, compilado a partir de la misma función de Rust, hace interactiva solo esa parte. El resto de la página sigue siendo HTML simple y no cuesta nada. Esta página muestra cómo escribir islas, colocarlas en las páginas, compilarlas y desplegarlas.

## Cómo funciona

```text
src/lib.rs ── fn counter(props) ──┬── en el servidor: renderizada a HTML dentro de la página
                                  │
                                  └── paquets build ──▶ pkg/my_app.js + pkg/my_app_bg.wasm
                                                        cargados por el navegador, que hace
                                                        interactivo el contador de la página
```

1. El servidor llama a la función de la isla y escribe su HTML en la página, dentro de un elemento `<active-island>` que guarda su nombre y sus props.
2. Los visitantes ven el contenido de inmediato, incluso antes de que se ejecute ningún script. Los buscadores también lo ven.
3. El navegador carga el bundle, llama a la misma función con las mismas props, se apropia del HTML que ya está ahí (no lo vuelve a dibujar) y conecta los manejadores de eventos.

Solo las páginas que contienen al menos una isla cargan el bundle. Una isla de contador pesa unos **36.8 kB** comprimida, WebAssembly y cargador incluidos: la más pequeña entre los frameworks front-end de Rust.

## Requisitos

Las islas se compilan a WebAssembly, lo que necesita dos herramientas:

- el target `wasm32-unknown-unknown` de Rust;
- `wasm-bindgen-cli`, exactamente en la versión del crate `wasm-bindgen` de tu `Cargo.lock`.

El primer `paquets dev` en un proyecto con islas instala lo que falte (tarda unos minutos, una sola vez). `paquets doctor` muestra lo que está instalado. Consulta [Instalación](installation.md#3-las-herramientas-para-las-islas-opcional).

## Tu primera isla

Un proyecto creado con `paquets new` (sin `--minimal`) ya tiene una isla, `counter`, en `src/lib.rs`. Este es el ciclo completo, con un contador más sencillo.

### 1. Escribe la isla en src/lib.rs

Una isla es una función que recibe sus props como una cadena y devuelve un `Node`:

```rust
// src/lib.rs
use paquets::prelude::*;

/// Un contador: las props son el valor inicial.
pub fn counter(props: &str) -> Node {
    let count = use_state(props.parse::<i32>().unwrap_or(0));
    button()
        .on_click(move |_| count.update(|n| *n += 1))
        .text_dyn(move || format!("Has hecho clic {} veces", count.get()))
        .into()
}
```

- `use_state(initial)` crea un estado: un valor que puede cambiar en el navegador.
- `.on_click(...)` ejecuta código cuando se hace clic en el botón. Aquí suma 1 al estado.
- `.text_dyn(...)` muestra un texto calculado a partir del estado. Cuando el estado cambia, solo se actualiza ese texto en la página.

La isla debe estar en `src/lib.rs` (o en un módulo de la biblioteca), porque el bundle del navegador se compila a partir de la biblioteca.

### 2. Regístrala para el navegador

Al final de `src/lib.rs`, el punto de entrada del navegador enumera todas las islas, por nombre:

```rust
/// El punto de entrada del navegador: hace interactivas todas las islas de la página.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[wasm_bindgen(start)]` ejecuta `start` en cuanto se carga el bundle. `#[cfg(target_arch = "wasm32")]` deja esta función fuera de la compilación del servidor.

### 3. Colócala en una página

En `src/main.rs`, importa la isla y colócala con `island(name, props, function)`:

```rust
// src/main.rs
use paquets::prelude::*;

use my_app::counter; // `my_app` es el nombre de tu paquete, con `_` en lugar de `-`

async fn home(_req: Request) -> Page {
    Page::new("Inicio")
        .body(h1().text("Un contador"))
        .body(island("counter", "0", counter))
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", home);
    app.run()
}
```

El nombre (`"counter"`) debe ser el mismo aquí y en `start()`. Las props (`"0"`) son el valor inicial.

### 4. Ejecútala

```sh
paquets dev
```

`paquets dev` compila `src/lib.rs` a WebAssembly (en modo debug), escribe el bundle en `pkg/` y luego ejecuta el servidor. La línea de arranque muestra el bundle que ha encontrado:

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: /pkg/my_app.js
```

Abre la página y haz clic en el botón. Después de modificar una isla, vuelve a ejecutar `paquets dev`: `cargo run` por sí solo recompila el servidor, no el bundle.

## Estado, eventos y vistas dinámicas

### Estado

| Método | Úsalo para |
|---|---|
| `use_state(initial)` | crear un estado; el handle es `Copy`, muévelo a tantas closures como necesites |
| `state.get()` | leer una copia del valor |
| `state.with(\|v\| ...)` | leer a través de una referencia, sin copiar (`text.with(String::len)`) |
| `state.set(value)` | reemplazar el valor |
| `state.update(\|v\| ...)` | cambiar el valor in situ (`*n += 1`, `list.push(...)`) |
| `memo(\|\| ...)` | un valor derivado de otros estados, que solo se vuelve a calcular cuando ellos cambian y que solo notifica cuando cambia su resultado (`memo(move \|\| items.with(Vec::len))`) |
| `untrack(\|\| ...)` | leer estados sin seguirlos |
| `on_cleanup(\|\| ...)` | ejecutar código cuando desaparece la parte de la página que lo creó (detener un temporizador, quitar un listener) |

### Eventos

| Método | Evento |
|---|---|
| `.on_click(\|e\| ...)` | un clic |
| `.on_input(\|e\| ...)` | el valor de un campo cambia, con cada pulsación de tecla |
| `.on("keydown", \|e\| ...)` | cualquier evento del DOM, por su nombre: `"change"`, `"submit"`, `"keydown"`, `"focus"`… |

El evento `e` ofrece `e.value()` (el valor del campo), `e.key()` (la tecla de un evento de teclado) y `e.prevent_default()`.

### Vistas dinámicas

Los métodos que reciben una closure en lugar de un valor siguen al estado: la closure se vuelve a ejecutar cuando cambia un estado que lee, y solo se actualiza esa parte de la página.

| Método | Actualiza |
|---|---|
| `.text_dyn(f)` | un texto |
| `.attr_dyn(name, f)` | un atributo (`value` también actualiza un campo) |
| `.class_if(class, f)` | añade una clase mientras `f()` sea `true` |
| `.bool_attr(name, f)` | un atributo booleano: `disabled`, `hidden`, `checked`… |
| `.children_dyn(f)` | una lista de hijos, que se reconstruye cuando cambia |

Un campo de texto que cuenta sus caracteres:

```rust
/// Un campo de texto que cuenta sus caracteres. Las props son el máximo.
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

Colócalo con `island("limited-text", "140", limited_text)` y añade `("limited-text", limited_text)` a la lista de `start()`.

## Props

Las props viajan dentro del HTML de la página, como una cadena. Usa un número o una palabra para los casos sencillos, y JSON para los datos estructurados:

```rust
// src/lib.rs
/// Las props de `add_to_cart`, enviadas como JSON.
#[data]
#[derive(Default)]
pub struct CartProps {
    pub product: String,
    pub price_cents: u32,
}

/// Un campo de cantidad y el precio total.
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
    let props = CartProps { product: "Silla de roble".into(), price_cents: 24900 };
    Page::new("Silla de roble")
        .body(h1().text("Silla de roble"))
        .body(island("add-to-cart", paquets::serde_json::to_string(&props).unwrap(), add_to_cart))
}
```

Las props forman parte del HTML: mantenlas pequeñas y nunca pongas secretos en ellas.

## Las reglas de las islas

- **Todo viene de las props.** Una isla es una función normal (`fn(&str) -> Node`) que no captura nada. En el navegador no tiene acceso a tu base de datos ni al estado de tu servidor: dale lo que necesite a través de sus props. Una vista que depende de algo que solo tiene el servidor (la hora, un número aleatorio, un valor de la base de datos que no se pasa en las props) produce un HTML distinto en el navegador, que el navegador vuelve a dibujar, con un aviso en su consola: `active: re-rendering island (tag mismatch)`.
- **Las islas son independientes.** Cada isla tiene sus propios estados. Si dos partes de una página deben reaccionar la una a la otra (un filtro y una lista), conviértelas en una sola isla, o comparte un store (consulta [Guardar datos en el navegador](#guardar-datos-en-el-navegador)).
- **El mismo marcado en los dos lados.** Usa `#[cfg(target_arch = "wasm32")]` solo para el comportamiento, nunca para cambiar lo que se renderiza.
- **Los nombres deben coincidir.** Una isla cuyo nombre no está en `start()` se queda estática, sin ningún error.
- **Datos del servidor.** Para leer o enviar datos después de que la página se haya cargado, llama a tu API con el `fetch` del navegador a través de `web-sys` y `wasm-bindgen-futures` (consulta la [guía de eventos de Active](https://active-c1883a966650.herokuapp.com/es/docs/events/)), o usa un formulario HTML normal que envíe los datos a una ruta.

## Guardar datos en el navegador

Con la feature `browser-storage`, las islas pueden guardar datos en el navegador del visitante, con el módulo `web` de Memoire (`paquets::orm::web`):

- **`Store<S>`**, estado de la aplicación al estilo de Pinia: `Store::new("cart")` devuelve el mismo store en todas las islas de la página, guardado en `localStorage` en cada cambio, restaurado en la siguiente visita y sincronizado entre pestañas;
- **`LocalStorageDb`**, una base de datos Memoire en `localStorage`: modelos, consultas y transacciones, como en el servidor.
- **`IndexedDbStorage`** guarda cualquiera de los dos en IndexedDB en lugar de `localStorage`: mucho más espacio y escrituras en segundo plano: `LocalStorageDb::open_with("notas", IndexedDbStorage::open("mi-app").await?)`. Se lee una vez, al abrirlo; un cambio hecho en otra pestaña se ve en la siguiente apertura.

Las islas de `src/lib.rs` se compilan para los dos lados (el servidor las renderiza y luego se hidratan en el navegador), así que activa la feature en las dos dependencias:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", features = ["browser-storage"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client", "browser-storage"] }
```

Dos islas que comparten un carrito, el badge del menú y el botón de la página de un producto:

```rust
use paquets::orm::web::Store;
use paquets::prelude::*;

#[data]
#[derive(Default)]
pub struct Cart {
    pub items: Vec<String>,
}

/// El número de artículos del carrito, al día en todas las islas y todas las pestañas.
pub fn cart_badge(_props: &str) -> Node {
    let count = use_state(0);
    // Los efectos solo se ejecutan en el navegador: el servidor renderiza un carrito vacío.
    effect(move || {
        let cart = Store::<Cart>::new("cart");
        count.set(cart.get(|c| c.items.len()));
        let follow = cart.subscribe_local(move |c| count.set(c.items.len()));
        on_cleanup(move || drop(follow));
    });
    span().class("badge").text_dyn(move || count.get().to_string()).into()
}

/// El botón de la página de un producto. Las props son el nombre del producto.
pub fn add_to_cart(product: &str) -> Node {
    let product = product.to_owned();
    button()
        .text("Añadir al carrito")
        .on_click(move |_| {
            Store::<Cart>::new("cart").update(|c| c.items.push(product.clone()));
        })
        .into()
}
```

Los datos se quedan en el navegador: el servidor nunca los ve, salvo que una isla los envíe (a tu API, con `fetch`). Consulta la [guía de almacenamiento del navegador](https://memoire-7f4525f1fdaf.herokuapp.com/es/docs/browser-storage/) de Memoire para las versiones y las migraciones, `SessionStorage` y `LocalStorageDb`.

## Compilar para producción

```sh
paquets build
```

`paquets build` compila las islas en modo release, con el perfil `wasm-release` optimizado para el tamaño, en `pkg/`, y luego compila el servidor con `cargo build --release`. Despliega `pkg/` (y `public/`) junto al programa: el `Dockerfile` de un proyecto generado ya lo hace (consulta [Docker](deploy-docker.md)).

Al arrancar, la aplicación busca `pkg/<name>.js` y `pkg/<name>_bg.wasm` en la carpeta actual y los sirve en `/pkg/`. Las páginas que contienen islas reciben entonces tres etiquetas en su `<head>`: una precarga del módulo, una precarga del archivo WebAssembly (que se descarga en paralelo con la página) y la línea que lo arranca. Su Content-Security-Policy permite WebAssembly. Las páginas sin islas no reciben nada de esto.

## Configurar un proyecto a mano

Un proyecto creado con `paquets new --minimal`, o un proyecto existente, necesita cuatro cambios para tener islas.

**1. `Cargo.toml`**: un crate, dos targets. La biblioteca también se compila a WebAssembly (`cdylib`), y en el navegador Paquets se usa sin su servidor ni sus bases de datos:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

# El bundle del navegador: lo más pequeño posible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

Quita `paquets` de la sección `[dependencies]` normal: déjalo solo en las dos secciones de target.

**2. `src/lib.rs`**: las islas y la función `start()` que se mostró antes.

**3. `src/main.rs`**: `use my_app::counter;` e `island(...)` en una página.

**4. `.gitignore`**: añade `/pkg`.

Después, `paquets dev` lo compila todo. Sin la herramienta `paquets`, estos son los comandos que ejecuta:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # la versión de Cargo.lock
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/my_app.wasm
cargo run --release
```

El archivo `.wasm` lleva el nombre del paquete, con `-` reemplazado por `_`.

## Solución de problemas

| Problema | Solución |
|---|---|
| La página muestra la isla, pero hacer clic no hace nada | Revisa la línea de arranque: `islands: none` significa que `pkg/` no tiene bundle; ejecuta `paquets dev`. Comprueba que el nombre usado en `island(...)` está en la lista de `start()`. Abre la consola del navegador para ver los errores. |
| `active: re-rendering island (… mismatch)` en la consola | El HTML del servidor y la vista del navegador no coinciden: pasa todos los datos de entrada por las props (consulta las reglas de más arriba). |
| `wasm-bindgen` informa de una diferencia de versión o de esquema | `cargo install wasm-bindgen-cli --version <versión de Cargo.lock> --locked`, o deja que lo haga `paquets dev`. |
| Errores sobre tokio, sockets o `mio` al compilar para wasm32 | Una dependencia exclusiva del servidor se está compilando para el navegador: muévela a `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. |
| No se encuentra `hydrate` | Falta la feature `client` en la línea de `paquets` para wasm32, o el código no está detrás de `#[cfg(target_arch = "wasm32")]`. |
| Los cambios en una isla no aparecen | Recompila el bundle con `paquets dev` (no solo `cargo run`) y luego recarga la página. |

## Más información

Las islas, los estados y los eventos vienen de [Active](https://active-c1883a966650.herokuapp.com/). Su documentación va más allá: [islas](https://active-c1883a966650.herokuapp.com/es/docs/islands/), [estado y efectos](https://active-c1883a966650.herokuapp.com/es/docs/state/), [eventos](https://active-c1883a966650.herokuapp.com/es/docs/events/) y [aplicaciones del lado del cliente](https://active-c1883a966650.herokuapp.com/es/docs/client-app/) (`paquets::mount_to`, `navigate` y `location` con la feature `client`).

## Próximos pasos

- [Páginas](pages.md): el resto de la página, renderizado en el servidor.
- [Herramienta de línea de comandos](cli.md): `paquets dev` y `paquets build` en detalle.
- [Desplegar con Docker](deploy-docker.md): cómo se compilan las islas en la imagen.
