//! `paquets doctor`: are the tools there?

use std::fs;

use super::tools;

pub fn run() -> Result<(), String> {
    println!("📦 Paquets {}\n", env!("CARGO_PKG_VERSION"));
    let mut ok = true;

    match tools::output("cargo", &["--version"]) {
        Some(version) => line(true, &version, ""),
        None => {
            ok = false;
            line(false, "cargo", "install Rust: https://rustup.rs");
        }
    }
    match tools::has_wasm_target() {
        Some(true) => line(true, "wasm32-unknown-unknown target", ""),
        Some(false) => line(
            false,
            "wasm32-unknown-unknown target (islands)",
            "rustup target add wasm32-unknown-unknown (paquets dev does it for you)",
        ),
        None => line(
            false,
            "rustup",
            "needed to add the wasm32 target: https://rustup.rs",
        ),
    }

    let wanted = fs::read_to_string("Cargo.lock")
        .ok()
        .and_then(|lock| tools::lock_version(&lock, "wasm-bindgen"));
    match (tools::wasm_bindgen_version(), wanted) {
        (Some(have), Some(want)) if have == want => {
            line(
                true,
                &format!("wasm-bindgen {have} (matches Cargo.lock)"),
                "",
            );
        }
        (Some(have), Some(want)) => line(
            false,
            &format!("wasm-bindgen {have}, Cargo.lock wants {want}"),
            &format!("cargo install wasm-bindgen-cli --version {want} --locked"),
        ),
        (Some(have), None) => line(true, &format!("wasm-bindgen {have}"), ""),
        (None, want) => line(
            false,
            "wasm-bindgen (islands)",
            &format!(
                "cargo install wasm-bindgen-cli{} --locked (paquets dev does it for you)",
                want.map(|v| format!(" --version {v}")).unwrap_or_default()
            ),
        ),
    }

    match tools::output("git", &["--version"]) {
        Some(version) => line(true, &version, ""),
        None => line(false, "git", "https://git-scm.com"),
    }
    match tools::output("docker", &["--version"]) {
        Some(version) => line(true, &version, ""),
        None => line(
            false,
            "docker (optional)",
            "only to build the production image locally",
        ),
    }

    println!();
    if ok {
        println!("Ready. Create an app with `paquets new my-app`.");
        Ok(())
    } else {
        Err("cargo is required".into())
    }
}

fn line(good: bool, what: &str, hint: &str) {
    let mark = if good { "✓" } else { "✗" };
    if hint.is_empty() {
        println!("  {mark} {what}");
    } else {
        println!("  {mark} {what}\n      → {hint}");
    }
}
