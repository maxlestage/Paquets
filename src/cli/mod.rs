//! The commands of the `paquets` tool (standard library only).

pub mod build;
pub mod doctor;
pub mod make;
pub mod new;
mod tools;

/// `paquets routes`, `paquets db:migrate`, `paquets db:seed`: runs the app with
/// `PAQUETS_COMMAND`, which answers instead of serving.
pub fn app_command(command: &str, args: &[String]) -> Result<(), String> {
    if !std::path::Path::new("Cargo.toml").is_file() {
        return Err(
            "no Cargo.toml here: run this command in a project folder (see `paquets new`)".into(),
        );
    }
    let mut cargo = std::process::Command::new("cargo");
    cargo.args(["run", "--quiet"]);
    if !args.is_empty() {
        cargo.arg("--").args(args);
    }
    cargo.env("PAQUETS_COMMAND", command);
    let status = cargo
        .status()
        .map_err(|e| format!("cannot start cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo run` failed ({status})"))
    }
}
