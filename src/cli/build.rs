//! `paquets dev` and `paquets build`: the islands (WebAssembly), then the server.

use std::fs;
use std::path::Path;
use std::process::Command;

use super::tools;

/// `paquets dev`: islands in debug mode, then `cargo run`.
pub fn dev(args: &[String]) -> Result<(), String> {
    let manifest = manifest()?;
    if let Err(message) = islands(&manifest, false) {
        // The pages still work: the islands stay static until they are built.
        eprintln!("warning: islands not built: {message}");
        eprintln!("         run `paquets doctor` to check the tools");
    }
    let extra = match args.first().map(String::as_str) {
        Some("--") => &args[1..],
        _ => args,
    };
    let mut run = Command::new("cargo");
    run.arg("run");
    if !extra.is_empty() {
        run.arg("--").args(extra);
    }
    tools::run(&mut run)
}

/// `paquets build`: islands in release mode (`pkg/`), then `cargo build --release`.
pub fn release() -> Result<(), String> {
    let manifest = manifest()?;
    islands(&manifest, true)?;
    tools::run(Command::new("cargo").args(["build", "--release"]))?;
    let name = tools::package_name(&manifest).unwrap_or_default();
    println!("📦 Built: {}/release/{name}", tools::target_dir().display());
    if Path::new("pkg").is_dir() && has_islands(&manifest) {
        println!("   Islands: pkg/ (deploy it next to the program, with public/)");
    }
    Ok(())
}

fn manifest() -> Result<String, String> {
    fs::read_to_string("Cargo.toml").map_err(|_| {
        "no Cargo.toml here: run this command in a project folder (see `paquets new`)".to_owned()
    })
}

/// A project has islands when its library is compiled for the browser too.
fn has_islands(manifest: &str) -> bool {
    manifest.contains("cdylib") && Path::new("src/lib.rs").is_file()
}

/// Compiles `src/lib.rs` to WebAssembly and writes the browser bundle to `pkg/`.
/// Installs the missing tools on the way (the wasm32 target, wasm-bindgen-cli).
fn islands(manifest: &str, release: bool) -> Result<(), String> {
    if !has_islands(manifest) {
        return Ok(());
    }
    let name = tools::package_name(manifest).ok_or("no package name in Cargo.toml")?;
    println!("🏝️  Islands");
    if tools::has_wasm_target() == Some(false) {
        println!("   Installing the wasm32-unknown-unknown target (once)…");
        tools::run(Command::new("rustup").args(["target", "add", "wasm32-unknown-unknown"]))?;
    }

    let profile = if !release {
        "dev"
    } else if manifest.contains("[profile.wasm-release]") {
        "wasm-release"
    } else {
        "release"
    };
    tools::run(Command::new("cargo").args([
        "build",
        "--lib",
        "--target",
        "wasm32-unknown-unknown",
        "--profile",
        profile,
    ]))?;

    let lock = fs::read_to_string("Cargo.lock").unwrap_or_default();
    let wanted = tools::lock_version(&lock, "wasm-bindgen")
        .ok_or("wasm-bindgen is not in Cargo.lock: add `wasm-bindgen = \"0.2\"` for wasm32")?;
    if tools::wasm_bindgen_version().as_deref() != Some(wanted.as_str()) {
        println!("   Installing wasm-bindgen {wanted} (once, it matches Cargo.lock)…");
        if !tools::download_wasm_bindgen(&wanted) {
            // No prebuilt binary for this system (or no network to GitHub): compile it.
            tools::run(Command::new("cargo").args([
                "install",
                "wasm-bindgen-cli",
                "--version",
                &wanted,
                "--locked",
            ]))?;
        }
    }

    let dir = if profile == "dev" { "debug" } else { profile };
    let wasm = tools::target_dir()
        .join("wasm32-unknown-unknown")
        .join(dir)
        .join(format!("{}.wasm", name.replace('-', "_")));
    tools::run(
        Command::new("wasm-bindgen")
            .args(["--target", "web", "--no-typescript", "--out-dir", "pkg"])
            .arg(&wasm),
    )?;
    Ok(())
}
