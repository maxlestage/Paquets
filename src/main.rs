//! `paquets`, the command line tool: creates, runs and builds Paquets apps.
//!
//! ```text
//! paquets new my-app      a complete project, ready to deploy
//! paquets dev             builds the islands, then runs the app
//! paquets build           release build: islands (pkg/) and server
//! paquets doctor          checks the tools
//! paquets make:model Post title:string --api   generators (model, api, page…)
//! paquets routes          lists the routes of the app
//! paquets db:migrate      runs the migrations
//! ```

mod cli;

use std::process::ExitCode;

const HELP: &str = "\
📦 Paquets — the simplest full-stack framework for Rust

Usage:
  paquets new <name> [--minimal]   create a project (server, pages, API, islands,
                                   Dockerfile, Heroku, CI with auto-merge into master)
  paquets dev [--watch] [-- <args>]
                                   build the islands, then run the app (cargo run);
                                   --watch: rebuild, restart and reload the pages
                                   at every change
  paquets build                    release build: islands in pkg/, then the server
  paquets doctor                   check cargo, the wasm32 target and wasm-bindgen
  paquets routes                   list the routes of the app
  paquets db:migrate               run the pending migrations (app.migrate)
  paquets db:seed                  fill the database (app.seed)

Generators (they write the code and wire it into src/main.rs):
{MAKE}
  paquets help                     this help
  paquets --version                the version

Options of `new`:
  --minimal       one src/main.rs, no islands (no WebAssembly tools needed)
  --path <dir>    use a local checkout of Paquets instead of the published one

Documentation: https://paquets-90dfc06b6fea.herokuapp.com/";

fn help() -> String {
    HELP.replace("{MAKE}", cli::make::USAGE)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            println!("{}", help());
            Ok(())
        }
        Some("-V" | "--version" | "version") => {
            println!("paquets {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("new") => cli::new::run(&args[1..]),
        Some("dev") => cli::build::dev(&args[1..]),
        Some("build") => cli::build::release(),
        Some("doctor") => cli::doctor::run(),
        Some("routes") => cli::app_command("routes", &args[1..]),
        Some("db:migrate") => cli::app_command("migrate", &args[1..]),
        Some("db:seed") => cli::app_command("seed", &args[1..]),
        Some(make) if make.starts_with("make:") => cli::make::run(make, &args[1..]),
        Some(other) => Err(format!(
            "unknown command `{other}`\n\nRun `paquets help` to see the commands."
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
