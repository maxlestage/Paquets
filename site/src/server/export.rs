//! `paquets-site export <dir>`: the whole site as static files, for GitHub Pages or any
//! static host. Every page comes from the server itself, through Paquets' in-memory
//! test client (`app.test()`), so both are identical.
//!
//! What a static host cannot do is replaced: `/` is the English home page instead of
//! a redirection by `Accept-Language`, `404.html` is the English "not found" page,
//! and the live demo is left out.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{Site, app, site_dir};
use crate::i18n::Lang;
use crate::routes::{Route, base, set_base};

/// The address used for canonical and Open Graph URLs when `SITE_URL` is not set.
fn default_site_url(base: &str) -> String {
    if base.is_empty() {
        "http://localhost:3000".to_owned()
    } else {
        format!("https://maxlestage.github.io{base}")
    }
}

/// Writes the site into `out`, links behind `base` (`""` or `/Paquets`), absolute URLs
/// (canonical, sitemap, Open Graph) on `site_url`. Returns the number of files written.
pub async fn export(out: &Path, base_path: &str, site_url: Option<String>) -> io::Result<usize> {
    set_base(base_path);
    let base = base();
    let out = std::path::absolute(out)?;
    // Paquets finds `pkg/` (the browser bundle) in the current directory.
    let dir = std::path::absolute(site_dir())?;
    std::env::set_current_dir(&dir)?;
    if !dir.join("pkg").is_dir() {
        eprintln!(
            "warning: site/pkg is missing: the pages will not be interactive (see site/README.md)"
        );
    }
    let site_url = site_url
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| default_site_url(base));
    let site = Arc::new(Site::load(false, Some(site_url)));
    let mut app = app(site);
    // Static files have no server to protect, and the export asks for every page at
    // once (the rate limit would stop it).
    app.security(None);
    let client = app.test().await;

    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };
    let get = |path: String, expected: u16| {
        let client = &client;
        async move {
            let res = client.get(&path).await;
            if res.status().as_u16() != expected {
                return Err(io::Error::other(format!(
                    "GET {path}: {} instead of {expected}",
                    res.status()
                )));
            }
            Ok(res.text())
        }
    };

    for route in Route::all() {
        let html = get(route.path(), 200).await?;
        write(
            &format!("{}index.html", route.path()),
            with_base(&html, base).as_bytes(),
        )?;
    }
    let home = get(Route::new(Lang::En, crate::routes::Page::Home).path(), 200).await?;
    write("index.html", with_base(&home, base).as_bytes())?;
    let missing = get("/en/404/".to_owned(), 404).await?;
    write("404.html", with_base(&missing, base).as_bytes())?;
    for lang in Lang::ALL {
        let index = format!("/search/{}.txt", lang.code());
        write(&index, get(index.clone(), 200).await?.as_bytes())?;
    }
    // The PWA: the manifest and the service worker list absolute URLs, the icons are
    // served by Paquets itself.
    for file in ["/manifest.webmanifest", "/sw.js"] {
        let body = get(file.to_owned(), 200).await?;
        write(file, with_base_everywhere(&body, base).as_bytes())?;
    }
    for icon in [
        "/paquets/icon-192.png",
        "/paquets/icon-512.png",
        "/paquets/icon-maskable-512.png",
        "/paquets/apple-touch-icon.png",
    ] {
        let res = client.get(icon).await;
        if res.status() != 200 {
            return Err(io::Error::other(format!("GET {icon}: {}", res.status())));
        }
        write(icon, res.bytes())?;
    }
    for file in ["/sitemap.xml", "/robots.txt"] {
        let body = get(file.to_owned(), 200).await?;
        // No demo on a static host.
        let body = body.replace("Disallow: /demo\n", "");
        write(file, body.as_bytes())?;
    }
    for folder in ["public", "pkg"] {
        let root = dir.join(folder);
        let prefix = if folder == "pkg" { "pkg/" } else { "" };
        for file in files(&root) {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            let url = format!("{prefix}{}", relative.to_string_lossy().replace('\\', "/"));
            // wasm-bindgen's own files, not needed to run the bundle.
            if url.ends_with(".gitignore") || url.ends_with("package.json") {
                continue;
            }
            write(&url, &std::fs::read(&file)?)?;
        }
    }
    // Files are served as they are: no Jekyll processing on GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}

/// The URLs that Paquets writes itself (the islands loader: `/pkg/…`) do not know the
/// base path: they get it here. The links of the site already have it.
pub fn with_base(html: &str, base: &str) -> String {
    if base.is_empty() {
        return html.to_owned();
    }
    html.replace("\"/pkg/", &format!("\"{base}/pkg/"))
        .replace("\"/paquets/", &format!("\"{base}/paquets/"))
        .replace(
            "\"/manifest.webmanifest\"",
            &format!("\"{base}/manifest.webmanifest\""),
        )
        .replace("\"/sw.js\"", &format!("\"{base}/sw.js\""))
}

/// Every absolute URL of a JSON or JavaScript file (`"/…"`) behind `base`.
pub fn with_base_everywhere(text: &str, base: &str) -> String {
    if base.is_empty() {
        return text.to_owned();
    }
    text.replace("\"/", &format!("\"{base}/"))
}

/// Every file under `dir`, recursively.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_the_islands_loader() {
        let html = r#"<link rel="modulepreload" href="/pkg/a.js"/><script type="module">import init from "/pkg/a.js";init({module_or_path:"/pkg/a_bg.wasm"});</script>"#;
        let out = with_base(html, "/Paquets");
        assert_eq!(out.matches("\"/Paquets/pkg/").count(), 3);
        assert_eq!(with_base(html, ""), html);
        assert_eq!(
            default_site_url("/Paquets"),
            "https://maxlestage.github.io/Paquets"
        );
    }
}
