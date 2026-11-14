//! `.env`: the settings of a project, in a file next to its `Cargo.toml`.
//!
//! Paquets reads its settings (`NUAGE_URL`, `NUAGE_TOKEN`, `NUAGE_BRANCH`, `DATABASE_URL`,
//! `PORT`, `APP_KEY`, `APP_URL`, `METRICS_TOKEN`...) from the environment, then from the file
//! `.env` of the folder the app runs in: `paquets nuage create` writes the two lines of a
//! Nuage DB application there, and `paquets dev` uses them. A variable of the environment
//! always wins over the file: in production, the host's settings (Heroku's Config Vars) are
//! what counts, and `.env` is not deployed (`paquets new` puts it in `.gitignore` and
//! `.dockerignore`).
//!
//! The file is lines of `KEY=value`: `#` comments, blank lines, an optional `export `, and the
//! value as it is or between `"` or `'`. It is read once, the first time a setting is asked.
//! [`env`] reads a setting the same way, for the app's own settings.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The settings file of a project.
pub const FILE: &str = ".env";

/// The setting `name`: the environment's value, else the one of `.env`; `None` when it is
/// empty or missing in both.
///
/// ```no_run
/// let key = paquets::env("STRIPE_KEY").expect("set STRIPE_KEY, in .env while developing");
/// ```
pub fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| file().get(name).cloned())
}

fn file() -> &'static HashMap<String, String> {
    static FILE_VARS: OnceLock<HashMap<String, String>> = OnceLock::new();
    FILE_VARS.get_or_init(|| {
        std::fs::read_to_string(FILE)
            .map(|text| parse(&text).into_iter().collect())
            .unwrap_or_default()
    })
}

/// The `KEY=value` lines of a settings file, in order (later lines win), empty values
/// left out. Lines that are not settings are ignored.
pub(crate) fn parse(text: &str) -> Vec<(String, String)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let line = line.strip_prefix("export ").map_or(line, str::trim_start);
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return None;
            }
            let value = value.trim();
            let value = [('"', '"'), ('\'', '\'')]
                .iter()
                .find_map(|&(a, b)| value.strip_prefix(a)?.strip_suffix(b))
                .filter(|_| value.len() >= 2)
                .unwrap_or(value);
            (!value.trim().is_empty()).then(|| (key.to_owned(), value.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_lines() {
        let vars = parse(
            "\u{feff}# Nuage DB\r\nNUAGE_URL=https://db.example.com\r\nexport NUAGE_TOKEN = \"nuage_a=b\" \n\
             EMPTY=\nnot a setting\nBAD-KEY=1\nPORT='4000'\nPORT=5000\n",
        );
        assert_eq!(
            vars,
            [
                ("NUAGE_URL".to_owned(), "https://db.example.com".to_owned()),
                ("NUAGE_TOKEN".to_owned(), "nuage_a=b".to_owned()),
                ("PORT".to_owned(), "4000".to_owned()),
                ("PORT".to_owned(), "5000".to_owned()),
            ]
        );
        let map: HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(map["PORT"], "5000", "the last line wins");
    }

    #[test]
    fn the_environment_wins() {
        // PATH is always set: the environment answers, whatever a .env file says.
        assert!(env("PATH").is_some());
        assert_eq!(env("PAQUETS_SURELY_NOT_SET_ANYWHERE"), None);
    }
}
