//! `paquets dev` and `paquets build`: the islands (WebAssembly), then the server.

use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, SystemTime};

use super::tools;

/// `paquets dev`: islands in debug mode, then `cargo run`; with `--watch`, again at
/// every change of the sources, and the pages reload by themselves.
pub fn dev(args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) == Some("--watch") {
        return watch(&args[1..]);
    }
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

/// The folders and files whose changes restart the server.
const WATCHED: [&str; 5] = ["src", "public", "Cargo.toml", "templates", "views"];

/// The latest modification time under the watched paths, and the number of files.
fn snapshot(root: &Path) -> (SystemTime, usize) {
    fn visit(path: &Path, latest: &mut SystemTime, count: &mut usize, depth: usize) {
        let Ok(meta) = fs::metadata(path) else {
            return;
        };
        if meta.is_dir() {
            if depth > 12 {
                return;
            }
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                    visit(&entry.path(), latest, count, depth + 1);
                }
            }
        } else {
            *count += 1;
            if let Ok(modified) = meta.modified()
                && modified > *latest
            {
                *latest = modified;
            }
        }
    }
    let mut latest = SystemTime::UNIX_EPOCH;
    let mut count = 0;
    for name in WATCHED {
        visit(&root.join(name), &mut latest, &mut count, 0);
    }
    (latest, count)
}

/// `paquets dev --watch`: builds and starts the server, then rebuilds and restarts it
/// when a file of `src/`, `public/`… changes. Pages opened in the browser reload when
/// the new server is up (Server-Sent Events from `/__paquets/live-reload`).
fn watch(args: &[String]) -> Result<(), String> {
    let manifest = manifest()?;
    let name = tools::package_name(&manifest).ok_or("no package name in Cargo.toml")?;
    let extra = match args.first().map(String::as_str) {
        Some("--") => &args[1..],
        _ => args,
    };
    println!("👀 Watching src/, public/ and Cargo.toml (Ctrl+C to stop)");
    let mut server: Option<Child> = None;
    loop {
        if let Some(mut old) = server.take() {
            let _ = old.kill();
            let _ = old.wait();
        }
        if let Err(message) = islands(&manifest, false) {
            eprintln!("warning: islands not built: {message}");
        }
        match tools::run(Command::new("cargo").args(["build", "--bin", &name])) {
            Ok(()) => {
                let program = tools::target_dir()
                    .join("debug")
                    .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
                match Command::new(&program)
                    .args(extra)
                    .env("PAQUETS_LIVE_RELOAD", "1")
                    .spawn()
                {
                    Ok(child) => server = Some(child),
                    Err(e) => eprintln!("error: cannot start {}: {e}", program.display()),
                }
            }
            Err(message) => eprintln!(
                "error: {message}
   waiting for a change…"
            ),
        }
        // Waits for a change of the sources (builds only write target/ and pkg/).
        let baseline = snapshot(Path::new("."));
        loop {
            std::thread::sleep(Duration::from_millis(400));
            if let Some(child) = server.as_mut()
                && let Ok(Some(status)) = child.try_wait()
            {
                eprintln!("   the server stopped ({status}): waiting for a change…");
                server = None;
            }
            if snapshot(Path::new(".")) != baseline {
                // Lets the editor finish writing.
                std::thread::sleep(Duration::from_millis(150));
                break;
            }
        }
        println!("\n🔄 Change detected: rebuilding…");
    }
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
