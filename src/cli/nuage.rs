//! `paquets nuage create [name] [description]` and `paquets nuage list`: the applications
//! of a Nuage DB server, through its management API (`memoire_orm::NuageAdmin`).
//!
//! `create` makes the application (a database of its own on the server) and writes its two
//! lines, `NUAGE_URL` and `NUAGE_TOKEN`, in the project's `.env`, which the app reads: the
//! next `paquets dev` runs on it. The server comes from `NUAGE_URL` (the environment or
//! `.env`), the management token from `NUAGE_ADMIN_TOKEN`, in the environment only: it opens
//! every application of the server, so it never goes in a project's file.

pub const USAGE: &str = "\
  paquets nuage create [name] [description]
                                   a new application on the Nuage DB server (NUAGE_URL,
                                   NUAGE_ADMIN_TOKEN): its NUAGE_URL and NUAGE_TOKEN go in
                                   .env; the name is the package's by default; --print
                                   shows them instead
  paquets nuage list               the applications of the server";

/// `.env` with `KEY=value` set for each pair: an existing line of the key is replaced
/// (the first; later ones are removed), a new key is added at the end.
pub fn with(text: &str, pairs: &[(&str, &str)]) -> String {
    let mut done = vec![false; pairs.len()];
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let key = trimmed
            .strip_prefix("export ")
            .unwrap_or(trimmed)
            .split('=')
            .next()
            .unwrap_or_default()
            .trim();
        match pairs.iter().position(|(k, _)| *k == key) {
            Some(i) if !done[i] => {
                out.push(format!("{}={}", pairs[i].0, pairs[i].1));
                done[i] = true;
            }
            Some(_) => {}
            None => out.push(line.to_owned()),
        }
    }
    for (i, (k, v)) in pairs.iter().enumerate() {
        if !done[i] {
            out.push(format!("{k}={v}"));
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// A Nuage DB application name from a package name: lowercase letters, digits, `-` and `_`,
/// 63 at most.
pub fn app_name(package: &str) -> String {
    let name: String = package
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    name.trim_matches('-').chars().take(63).collect()
}

/// `.gitignore` lists `.env`, or gets it.
fn ignore_env() -> Result<(), String> {
    let path = std::path::Path::new(".gitignore");
    if !path.exists() && !std::path::Path::new(".git").exists() {
        return Ok(());
    }
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text
        .lines()
        .any(|l| matches!(l.trim(), ".env" | "/.env" | ".env*" | "*.env"))
    {
        return Ok(());
    }
    let mut text = text;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("# The settings of this computer (NUAGE_TOKEN...): never committed\n.env\n");
    std::fs::write(path, text).map_err(|e| format!(".gitignore: {e}"))?;
    println!("   .gitignore: .env added (its token must not be committed)");
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    #[cfg(feature = "nuage")]
    {
        imp::run(args)
    }
    #[cfg(not(feature = "nuage"))]
    {
        let _ = args;
        Err("this paquets was built without its `nuage` feature".into())
    }
}

#[cfg(feature = "nuage")]
mod imp {
    use paquets::orm::{Error, NuageAdmin, nuage::NuageError};

    use super::{USAGE, app_name, ignore_env, with};

    fn admin() -> Result<NuageAdmin, String> {
        let url = paquets::env("NUAGE_URL").ok_or(
            "set NUAGE_URL to your Nuage DB server (https://…), in the environment or in .env",
        )?;
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(format!(
                "NUAGE_URL={url} is a folder (Nuage DB's local mode): `paquets nuage` needs a server, https://…"
            ));
        }
        let token = std::env::var("NUAGE_ADMIN_TOKEN")
            .ok()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .ok_or(
                "set NUAGE_ADMIN_TOKEN, the server's management token, in the environment \
                 (not in .env: it opens every application of the server). On the server's \
                 computer, `sudo nuage apps create <name>` needs none",
            )?;
        NuageAdmin::new(&url, &token).map_err(|e| e.to_string())
    }

    fn explain(e: Error, url: &str, name: &str) -> String {
        if e.is_unique_violation() {
            return format!(
                "{url} already has an application `{name}`: pick another name (`paquets nuage create {name}-2`), \
                 or make it a new token in the console"
            );
        }
        if let Error::Database(d) = &e
            && let Some(n) = d.downcast_ref::<NuageError>()
        {
            if n.is_unauthorized() {
                return format!(
                    "{url} refused NUAGE_ADMIN_TOKEN: use the NUAGE_ADMIN_TOKEN of the server's settings"
                );
            }
            if n.status == 403 || n.status == 404 {
                return format!(
                    "{url}: {} (a Nuage DB server of 0.4.2 or later, with NUAGE_ADMIN_TOKEN set)",
                    n.message
                );
            }
        }
        e.to_string()
    }

    fn block_on<T>(f: impl std::future::Future<Output = T>) -> Result<T, String> {
        Ok(tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?
            .block_on(f))
    }

    pub fn run(args: &[String]) -> Result<(), String> {
        let print = args.iter().any(|a| a == "--print");
        let args: Vec<&str> = args
            .iter()
            .map(String::as_str)
            .filter(|a| *a != "--print")
            .collect();
        match args.as_slice() {
            ["create", rest @ ..] => {
                let name = match rest.first() {
                    Some(name) => (*name).to_owned(),
                    None => std::fs::read_to_string("Cargo.toml")
                        .ok()
                        .and_then(|m| crate::cli::tools::package_name(&m))
                        .map(|p| app_name(&p))
                        .filter(|n| !n.is_empty())
                        .ok_or("give a name (`paquets nuage create shop`), or run it in a project folder")?,
                };
                let description = rest.get(1..).map(|d| d.join(" ")).unwrap_or_default();
                let admin = admin()?;
                let url = paquets::env("NUAGE_URL").unwrap_or_default();
                let made = block_on(admin.create_app(&name, &description))?
                    .map_err(|e| explain(e, &url, &name))?;
                if print {
                    println!("{}", made.env());
                    return Ok(());
                }
                let before = std::fs::read_to_string(".env").unwrap_or_default();
                let after = with(
                    &before,
                    &[("NUAGE_URL", &made.url), ("NUAGE_TOKEN", &made.token)],
                );
                std::fs::write(".env", after).map_err(|e| format!(".env: {e}"))?;
                println!(
                    "📦 application `{}` created on {}, with its own database",
                    made.app.name, made.url
                );
                println!("   .env: NUAGE_URL and NUAGE_TOKEN written: `paquets dev` runs on it");
                ignore_env()?;
                println!(
                    "   production: set the same two variables on your host, e.g. on Heroku:\n   \
                     heroku config:set NUAGE_URL={} NUAGE_TOKEN={}",
                    made.url, made.token
                );
                Ok(())
            }
            ["list"] => {
                let url = paquets::env("NUAGE_URL").unwrap_or_default();
                let apps = block_on(admin()?.apps())?.map_err(|e| explain(e, &url, ""))?;
                for app in apps {
                    let tokens = app.tokens.len() + usize::from(app.env_token);
                    let mut line = format!("{}\t{tokens} token(s)", app.name);
                    if !app.description.is_empty() {
                        line.push_str(&format!("\t{}", app.description));
                    }
                    println!("{line}");
                }
                Ok(())
            }
            _ => Err(format!("usage:\n{USAGE}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writing_keeps_the_rest() {
        let before =
            "# my app\nAPP_KEY=k\nNUAGE_URL=http://old\nexport NUAGE_TOKEN=old\nNUAGE_URL=dup\n";
        let after = with(
            before,
            &[
                ("NUAGE_URL", "https://db.example.com"),
                ("NUAGE_TOKEN", "nuage_x"),
                ("NUAGE_BRANCH", "preview"),
            ],
        );
        assert_eq!(
            after,
            "# my app\nAPP_KEY=k\nNUAGE_URL=https://db.example.com\nNUAGE_TOKEN=nuage_x\nNUAGE_BRANCH=preview\n"
        );
        assert_eq!(with("", &[("A", "1")]), "A=1\n");
    }

    #[test]
    fn names_from_packages() {
        assert_eq!(app_name("my-app"), "my-app");
        assert_eq!(app_name("My App.rs"), "my-app-rs");
        assert_eq!(app_name("moncap-gps"), "moncap-gps");
        assert_eq!(app_name(&"a".repeat(80)).len(), 63);
    }
}
