//! The smallest Paquets app.
//!
//!     cargo run --example hello      # http://localhost:3000

use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Hello").body(h1().text("Hello 👋"))
    });
    app.run()
}
