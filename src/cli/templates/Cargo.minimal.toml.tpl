[package]
name = "{{name}}"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
paquets = {{paquets}}

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
