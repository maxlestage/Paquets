[package]
name = "{{name}}"
version = "0.1.0"
edition = "2024"
publish = false

# One crate, two targets: the server (src/main.rs) and, compiled to WebAssembly by
# `paquets build`, the islands of src/lib.rs.
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = {{paquets}}

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = {{paquets_client}}
wasm-bindgen = "0.2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true

# The browser bundle: as small as possible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
