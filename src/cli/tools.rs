//! Running cargo, rustup and wasm-bindgen, and reading Cargo files.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Runs a command with the terminal attached; an error if it fails.
pub fn run(command: &mut Command) -> Result<(), String> {
    let shown = format!("{command:?}").replace('"', "");
    println!("  → {shown}");
    let status = command
        .status()
        .map_err(|e| format!("cannot start `{shown}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{shown}` failed ({status})"))
    }
}

/// The trimmed standard output of a successful command.
pub fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// The version of `package` in a `Cargo.lock`.
pub fn lock_version(lock: &str, package: &str) -> Option<String> {
    let wanted = format!("name = \"{package}\"");
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == wanted {
            let version = lines.next()?.trim();
            return version
                .strip_prefix("version = \"")?
                .strip_suffix('"')
                .map(str::to_owned);
        }
    }
    None
}

/// The `name` of the `[package]` section of a `Cargo.toml`.
pub fn package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && let Some(value) = line.strip_prefix("name") {
            let value = value.trim_start().strip_prefix('=')?.trim();
            return Some(value.trim_matches('"').to_owned());
        }
    }
    None
}

/// Where cargo writes what it builds.
pub fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| PathBuf::from("target"), PathBuf::from)
}

/// Whether the `wasm32-unknown-unknown` target is installed (`None` without rustup).
pub fn has_wasm_target() -> Option<bool> {
    let installed = output("rustup", &["target", "list", "--installed"])?;
    Some(
        installed
            .lines()
            .any(|l| l.trim() == "wasm32-unknown-unknown"),
    )
}

/// The version of the installed `wasm-bindgen` command, if any.
pub fn wasm_bindgen_version() -> Option<String> {
    let out = output("wasm-bindgen", &["--version"])?;
    out.split_whitespace().nth(1).map(str::to_owned)
}

/// Where `cargo install` puts programs (`$CARGO_HOME/bin`, `~/.cargo/bin` by default).
fn cargo_bin() -> Option<PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|home| PathBuf::from(home).join(".cargo"))
        })?;
    Some(home.join("bin"))
}

/// The name of the prebuilt wasm-bindgen archive for this system, if there is one.
fn wasm_bindgen_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("windows", "x86_64") => Some("x86_64-pc-windows-msvc"),
        _ => None,
    }
}

/// Installs the prebuilt `wasm-bindgen` of GitHub's releases in `~/.cargo/bin`, with
/// `curl` and `tar` (present on Linux, macOS and Windows 10+). `false` if it did not
/// work, so that the caller compiles it instead.
pub fn download_wasm_bindgen(version: &str) -> bool {
    let (Some(target), Some(bin)) = (wasm_bindgen_target(), cargo_bin()) else {
        return false;
    };
    let name = format!("wasm-bindgen-{version}-{target}");
    let url = format!(
        "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/{version}/{name}.tar.gz"
    );
    let dir = std::env::temp_dir().join(format!("paquets-{name}"));
    let archive = dir.join("wasm-bindgen.tar.gz");
    let program = if cfg!(windows) {
        "wasm-bindgen.exe"
    } else {
        "wasm-bindgen"
    };
    let quiet = |command: &mut Command| {
        command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    let done = std::fs::create_dir_all(&dir).is_ok()
        && std::fs::create_dir_all(&bin).is_ok()
        && quiet(
            Command::new("curl")
                .args(["-sSfL", "-o"])
                .arg(&archive)
                .arg(&url),
        )
        && quiet(
            Command::new("tar")
                .arg("-xzf")
                .arg(&archive)
                .arg("-C")
                .arg(&dir),
        )
        && std::fs::copy(dir.join(&name).join(program), bin.join(program)).is_ok();
    let _ = std::fs::remove_dir_all(&dir);
    done && wasm_bindgen_version().as_deref() == Some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_versions_from_cargo_lock() {
        let lock = "[[package]]\nname = \"wasm-bindgen\"\nversion = \"0.2.129\"\nsource = \"x\"\n\n[[package]]\nname = \"wasm-bindgen-macro\"\nversion = \"0.2.129\"\n";
        assert_eq!(
            lock_version(lock, "wasm-bindgen").as_deref(),
            Some("0.2.129")
        );
        assert_eq!(lock_version(lock, "serde"), None);
    }

    #[test]
    fn reads_the_package_name() {
        let manifest =
            "[workspace]\nname = \"no\"\n\n[package]\nname = \"my-app\"\nversion = \"0.1.0\"\n";
        assert_eq!(package_name(manifest).as_deref(), Some("my-app"));
        assert_eq!(package_name("[dependencies]\nname = \"x\""), None);
    }
}
