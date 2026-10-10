//! Shared by the server and the browser: the components and the islands.
//!
//! The server renders them to HTML (src/main.rs); `paquets build` compiles this
//! file to WebAssembly so that the browser makes the islands interactive.

use paquets::prelude::*;

/// An island: rendered by the server, then interactive in the browser.
pub fn counter(props: &str) -> Node {
    let count = use_state(props.parse::<i32>().unwrap_or(0));
    div()
        .class("row")
        .child(
            button()
                .class("ghost")
                .attr("aria-label", "minus")
                .on_click(move |_| count.update(|n| *n -= 1))
                .text("−"),
        )
        .child(strong().text_dyn(move || count.get().to_string()))
        .child(
            button()
                .attr("aria-label", "plus")
                .on_click(move |_| count.update(|n| *n += 1))
                .text("+"),
        )
        .into()
}

/// The browser entry point: makes every island of the page interactive.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
