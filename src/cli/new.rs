//! `paquets new <name>`: a complete project, ready to run and to deploy.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where projects get Paquets from: its GitHub repository, until it is on crates.io.
const SOURCE: &str = r#"git = "https://github.com/maxlestage/Paquets""#;

struct Options {
    name: String,
    minimal: bool,
    path: Option<PathBuf>,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse(args)?;
    let dir = PathBuf::from(&options.name);
    if dir.exists() && fs::read_dir(&dir).map_or(true, |mut d| d.next().is_some()) {
        return Err(format!(
            "`{}` already exists and is not empty",
            dir.display()
        ));
    }
    let files = files(&options);
    for (path, content) in &files {
        let path = dir.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let git = init_git(&dir);

    println!(
        "📦 {} is ready ({} files{}).\n",
        options.name,
        files.len(),
        git
    );
    println!("  cd {}", options.name);
    println!("  paquets dev        # http://localhost:3000\n");
    if !options.minimal {
        println!("The first `paquets dev` also prepares the WebAssembly tools for the islands.");
    }
    println!("Deploy from your phone: push the project to GitHub, then see README.md.");
    Ok(())
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut name = None;
    let mut minimal = false;
    let mut path = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--minimal" => minimal = true,
            "--path" => {
                let dir = args.next().ok_or("--path needs a directory")?;
                let dir = fs::canonicalize(dir).map_err(|e| format!("--path {dir}: {e}"))?;
                path = Some(dir);
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option `{flag}`")),
            value if name.is_none() => name = Some(value.to_owned()),
            value => return Err(format!("unexpected argument `{value}`")),
        }
    }
    let name = name.ok_or("usage: paquets new <name> [--minimal]")?;
    check_name(&name)?;
    Ok(Options {
        name,
        minimal,
        path,
    })
}

/// A valid package name: lowercase letters, digits, `-` and `_`, starting with a letter.
fn check_name(name: &str) -> Result<(), String> {
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "`{name}` is not a valid name: use lowercase letters, digits, `-` and `_`, starting with a letter (e.g. my-app)"
        ));
    }
    if matches!(
        name,
        "paquets" | "test" | "std" | "core" | "alloc" | "proc_macro"
    ) {
        return Err(format!("`{name}` is reserved, choose another name"));
    }
    Ok(())
}

/// Every file of the project, with its content.
fn files(options: &Options) -> Vec<(&'static str, String)> {
    let name = options.name.as_str();
    let krate = name.replace('-', "_");
    let title = title(name);
    let source = match &options.path {
        Some(dir) => format!("path = {:?}", dir.display().to_string()),
        None => SOURCE.to_owned(),
    };
    let fill = |template: &str| {
        template
            .replace("{{name}}", name)
            .replace("{{crate}}", &krate)
            .replace("{{title}}", &title)
            .replace("{{paquets}}", &format!("{{ {source} }}"))
            .replace(
                "{{paquets_client}}",
                &format!("{{ {source}, default-features = false, features = [\"client\"] }}"),
            )
    };
    let (manifest, dockerfile) = if options.minimal {
        (
            include_str!("templates/Cargo.minimal.toml.tpl"),
            include_str!("templates/Dockerfile.minimal.tpl"),
        )
    } else {
        (
            include_str!("templates/Cargo.toml.tpl"),
            include_str!("templates/Dockerfile.tpl"),
        )
    };
    let main = fill(include_str!("templates/main.rs.tpl"));
    let readme = fill(include_str!("templates/README.md.tpl"));
    let (main, readme) = if options.minimal {
        (
            main.replace("{{use_counter}}", "")
                .replace("{{counter_card}}", ""),
            readme
                .replace("{{readme_islands}}", "")
                .replace("{{readme_lib}}", ""),
        )
    } else {
        (
            main.replace("{{use_counter}}", &format!("\nuse {krate}::counter;\n"))
                .replace("{{counter_card}}", COUNTER_CARD),
            readme
                .replace("{{readme_islands}}", " and WebAssembly islands")
                .replace(
                    "{{readme_lib}}",
                    "- `src/lib.rs`: components shared with the browser, and the islands (WebAssembly).\n",
                ),
        )
    };

    let mut files = vec![
        ("Cargo.toml", fill(manifest)),
        ("src/main.rs", main),
        (
            "public/favicon.svg",
            fill(include_str!("templates/favicon.svg.tpl")),
        ),
        ("Dockerfile", fill(dockerfile)),
        ("heroku.yml", fill(include_str!("templates/heroku.yml.tpl"))),
        ("app.json", fill(include_str!("templates/app.json.tpl"))),
        (
            ".github/workflows/ci.yml",
            fill(include_str!("templates/ci.yml.tpl")),
        ),
        (
            ".github/workflows/heroku.yml",
            fill(include_str!("templates/heroku-workflow.yml.tpl")),
        ),
        ("README.md", readme),
        (".gitignore", fill(include_str!("templates/gitignore.tpl"))),
        (
            ".dockerignore",
            fill(include_str!("templates/dockerignore.tpl")),
        ),
    ];
    if !options.minimal {
        files.push(("src/lib.rs", fill(include_str!("templates/lib.rs.tpl"))));
    }
    files
}

const COUNTER_CARD: &str = r#"                .child(
                    div()
                        .class("card")
                        .child(h2().text("An island"))
                        .child(
                            p().class("muted")
                                .text("Interactive in the browser, in Rust."),
                        )
                        .child(island("counter", "0", counter)),
                )
"#;

/// `my-app` → `My App`.
fn title(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |c| {
                c.to_uppercase().chain(chars).collect::<String>()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `git init` on `master` (the branch the CI merges into), when git is there and
/// the project is not already inside a repository.
fn init_git(dir: &Path) -> &'static str {
    let inside = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if inside {
        return "";
    }
    let init = Command::new("git")
        .args(["init", "--quiet", "--initial-branch=master"])
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if init {
        ", git repository on master"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(minimal: bool) -> Options {
        Options {
            name: "my-app".into(),
            minimal,
            path: None,
        }
    }

    #[test]
    fn names_are_checked() {
        assert!(check_name("my-app").is_ok());
        assert!(check_name("blog2").is_ok());
        assert!(check_name("My-App").is_err());
        assert!(check_name("2fast").is_err());
        assert!(check_name("my app").is_err());
        assert!(check_name("").is_err());
        assert!(check_name("paquets").is_err());
    }

    #[test]
    fn titles() {
        assert_eq!(title("my-app"), "My App");
        assert_eq!(title("todo_list"), "Todo List");
    }

    #[test]
    fn a_full_project_has_islands_and_no_placeholder_left() {
        let files = files(&options(false));
        let names: Vec<&str> = files.iter().map(|(n, _)| *n).collect();
        for expected in [
            "Cargo.toml",
            "src/main.rs",
            "src/lib.rs",
            "Dockerfile",
            "app.json",
        ] {
            assert!(names.contains(&expected), "{expected}");
        }
        for (name, content) in &files {
            for placeholder in [
                "{{name",
                "{{crate",
                "{{title",
                "{{paquets",
                "{{use_",
                "{{counter",
                "{{readme",
            ] {
                assert!(!content.contains(placeholder), "{placeholder} in {name}");
            }
        }
        let main = &files.iter().find(|(n, _)| *n == "src/main.rs").unwrap().1;
        assert!(main.contains("use my_app::counter;"));
        assert!(main.contains(r#"island("counter""#));
        let docker = &files.iter().find(|(n, _)| *n == "Dockerfile").unwrap().1;
        assert!(docker.contains("wasm-release/my_app.wasm"));
        let manifest = &files.iter().find(|(n, _)| *n == "Cargo.toml").unwrap().1;
        assert!(manifest.contains(r#"features = ["client"]"#));
        // GitHub expressions are not template placeholders.
        let ci = &files
            .iter()
            .find(|(n, _)| *n == ".github/workflows/ci.yml")
            .unwrap()
            .1;
        assert!(ci.contains("${{ github.ref_name }}"));
    }

    #[test]
    fn a_minimal_project_has_one_source_file() {
        let files = files(&options(true));
        assert!(!files.iter().any(|(n, _)| *n == "src/lib.rs"));
        let main = &files.iter().find(|(n, _)| *n == "src/main.rs").unwrap().1;
        assert!(!main.contains("counter"));
        let docker = &files.iter().find(|(n, _)| *n == "Dockerfile").unwrap().1;
        assert!(!docker.contains("wasm"));
    }
}
