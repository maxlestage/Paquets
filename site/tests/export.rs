//! The static export for GitHub Pages, behind the `/Paquets` base path.

use std::path::Path;

use paquets_site::catalog::PAGES;

fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// The HTML without the content of `<code>` elements (code samples may show links).
fn without_code(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<code") {
        out.push_str(&rest[..start]);
        match rest[start..].find("</code>") {
            Some(end) => rest = &rest[start + end + 7..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[tokio::test(flavor = "multi_thread")]
async fn exports_every_page_behind_the_base_path() {
    let out = std::env::temp_dir().join(format!("paquets-site-export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let written = paquets_site::server::export::export(
        &out,
        "/Paquets",
        Some("https://maxlestage.github.io/Paquets".into()),
    )
    .await
    .unwrap();
    assert!(written > 3 * (2 + PAGES.len()));
    for file in [
        "index.html",
        "en/index.html",
        "fr/docs/index.html",
        "es/docs/quick-start/index.html",
        "404.html",
        "en/404/index.html",
        "fr/404/index.html",
        "sitemap.xml",
        "robots.txt",
        "search/en.txt",
        "site.css",
        "assets/logo.svg",
        ".nojekyll",
    ] {
        assert!(out.join(file).is_file(), "{file}");
    }
    let home = std::fs::read_to_string(out.join("fr/index.html")).unwrap();
    assert!(
        home.contains(r#"<link rel="canonical" href="https://maxlestage.github.io/Paquets/fr/"/>"#)
    );
    assert!(home.contains("data-base=\"/Paquets\""));
    assert!(!home.contains("/demo"), "no live demo on a static host");
    let search = std::fs::read_to_string(out.join("search/fr.txt")).unwrap();
    assert!(search.contains("\t/Paquets/fr/docs/routing/\t"));
    let robots = std::fs::read_to_string(out.join("robots.txt")).unwrap();
    assert!(robots.contains("Sitemap: https://maxlestage.github.io/Paquets/sitemap.xml"));
    // The root and `404.html` pick the language in the browser.
    let root = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(root.contains("navigator.languages"));
    assert!(root.contains(r#"href="/Paquets/fr/""#));
    // French without JavaScript, and when the browser speaks none of the languages.
    assert!(root.contains(r#"content="0; url=/Paquets/fr/""#), "{root}");
    assert!(root.contains("(c||'fr')"));
    // The installed app opens in the language of the page it was installed from.
    assert!(home.contains(r#"<link rel="manifest" href="/Paquets/fr/manifest.webmanifest"/>"#));
    let manifest = std::fs::read_to_string(out.join("fr/manifest.webmanifest")).unwrap();
    assert!(
        manifest.contains(r#""start_url":"/Paquets/fr/""#),
        "{manifest}"
    );
    let missing = std::fs::read_to_string(out.join("404.html")).unwrap();
    assert!(missing.contains("navigator.languages") && missing.contains(r#""404/""#));
    assert!(missing.contains(r#"href="/Paquets/es/404/""#));

    // Every link of every page stays under the base path.
    let mut all = Vec::new();
    files(&out, &mut all);
    for file in all
        .iter()
        .filter(|f| f.extension().is_some_and(|e| e == "html"))
    {
        let html = without_code(&std::fs::read_to_string(file).unwrap());
        for attr in ["href=\"/", "src=\"/", "action=\"/", "from \"/", "path:\"/"] {
            for rest in html.split(attr).skip(1) {
                assert!(
                    rest.starts_with("Paquets/"),
                    "{}: {attr}{}",
                    file.display(),
                    &rest[..rest.len().min(40)]
                );
            }
        }
    }
    std::fs::remove_dir_all(&out).unwrap();
}
