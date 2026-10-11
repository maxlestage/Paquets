//! The whole site through Paquets' in-memory test client: every page of every
//! language, the redirections, the search index, the sitemap, static files and the
//! live demo.

use std::sync::Arc;

use paquets::test::TestClient;
use paquets_site::catalog::PAGES;
use paquets_site::routes::{Lang, Route};
use paquets_site::server::{Site, app};

async fn client() -> TestClient {
    app(Arc::new(Site::load(true, None))).test().await
}

#[tokio::test]
async fn every_page_renders_in_every_language() {
    let client = client().await;
    let routes = Route::all();
    assert_eq!(routes.len(), 3 * (2 + PAGES.len()));
    for route in routes {
        let path = route.path();
        let res = client.get(&path).header("host", "example.com").await;
        assert_eq!(res.status(), 200, "{path}");
        let html = res.text();
        let code = route.lang.code();
        assert!(
            html.starts_with(&format!("<!DOCTYPE html><html lang=\"{code}\">")),
            "{path}"
        );
        assert!(
            html.contains(&format!(
                r#"<link rel="canonical" href="http://example.com{path}"/>"#
            )),
            "{path}"
        );
        for lang in Lang::ALL {
            let alternate = route.with_lang(lang).path();
            assert!(
                html.contains(&format!(
                    r#"hreflang="{}" href="http://example.com{alternate}""#,
                    lang.code()
                )),
                "{path}: {alternate}"
            );
        }
        // Every page has an island (the search of the navigation), so the browser
        // bundle starts the page motion everywhere.
        assert!(
            html.contains(r#"<active-island data-island="search""#),
            "{path}"
        );
        assert!(html.contains("/site.css?v="), "{path}");
        assert!(html.contains("class=\"footer\""), "{path}");
        // The page's own policy allows its inline style (active's island style).
        let csp = res
            .header("content-security-policy")
            .unwrap_or_default()
            .to_owned();
        assert!(csp.contains("'sha256-"), "{path}: {csp}");
        assert!(
            csp.contains("https://fonts.googleapis.com"),
            "{path}: {csp}"
        );
    }
}

#[tokio::test]
async fn the_home_page_has_every_section() {
    let client = client().await;
    for lang in Lang::ALL {
        let res = client.get(&format!("/{}/", lang.code())).await;
        let html = res.text();
        let t = lang.texts();
        for id in [
            "top", "packages", "code", "features", "numbers", "start", "deploy", "docs",
        ] {
            assert!(
                html.contains(&format!("id=\"{id}\"")),
                "{}: #{id}",
                lang.code()
            );
        }
        for island in ["showcase", "terminal", "search"] {
            assert!(
                html.contains(&format!("data-island=\"{island}\"")),
                "{island}"
            );
        }
        assert!(html.contains(t.hero_lead));
        assert!(html.contains("cargo install --git https://github.com/maxlestage/Paquets paquets"));
        assert!(html.contains("© 2026"));
        // The signature footer: the author, the license, the ecosystem.
        assert!(html.contains(r#"class="footer__name""#));
        assert!(html.contains("Maxime Nathan Lestage"));
        assert!(
            html.contains("All rights reserved")
                || html.contains("Tous droits réservés")
                || html.contains("Todos los derechos reservados")
        );
        assert!(html.contains(r#"href="/LICENSE.txt""#));
        // The source is on GitHub, the only way to get the crate until it is on crates.io.
        assert!(html.contains("https://github.com/maxlestage/Paquets"));
        assert!(!html.contains("MIT OR Apache-2.0"));
        assert!(html.contains(&format!("v{}", paquets::VERSION)));
        for project in [
            "Active",
            "Vitesse",
            "Securite",
            "Memoire",
            "Paquets",
            "FerroSwift",
        ] {
            assert!(
                html.contains(&format!(
                    r#"href="https://maxlestage.github.io/{project}/""#
                )),
                "{project}"
            );
        }
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains(r#"<meta name="author" content="Maxime Nathan Lestage"/>"#));
        // The preloader plays once per visit (a cookie remembers it).
        assert!(html.contains("class=\"loader\""));
        assert!(
            res.header("set-cookie")
                .unwrap_or_default()
                .contains("paquets-intro=1")
        );
        let again = client
            .get(&format!("/{}/", lang.code()))
            .header("cookie", "paquets-intro=1")
            .await;
        assert!(!again.text().contains("class=\"loader\""));
    }
}

#[tokio::test]
async fn docs_pages_have_navigation() {
    let client = client().await;
    let html = client.get("/fr/docs/routing/").await.text();
    assert!(html.contains("class=\"doc-side\""));
    assert!(html.contains("aria-current=\"page\""));
    assert!(
        html.contains("href=\"/fr/docs/requests-responses/\""),
        "next page"
    );
    assert!(
        html.contains("href=\"/fr/docs/project-structure/\""),
        "previous page"
    );
    assert!(html.contains("https://github.com/maxlestage/Paquets"));
    let hub = client.get("/es/docs/").await.text();
    for page in &PAGES {
        assert!(
            hub.contains(&format!("href=\"/es/docs/{}/\"", page.slug)),
            "{}",
            page.slug
        );
    }
}

#[tokio::test]
async fn redirections_and_missing_pages() {
    let client = client().await;
    let res = client
        .get("/")
        .header("accept-language", "es-ES,es;q=0.9,en;q=0.8")
        .await;
    assert_eq!(res.status(), 302);
    assert_eq!(res.header("location"), Some("/es/"));
    // French when the browser speaks none of the site's languages.
    let res = client.get("/").await;
    assert_eq!(res.header("location"), Some("/fr/"));
    let res = client.get("/").header("accept-language", "de-DE").await;
    assert_eq!(res.header("location"), Some("/fr/"));
    let res = client
        .get("/")
        .header("accept-language", "en-US,en;q=0.9")
        .await;
    assert_eq!(res.header("location"), Some("/en/"));
    let res = client.get("/fr/docs").await;
    assert_eq!(res.status(), 301);
    assert_eq!(res.header("location"), Some("/fr/docs/"));
    for path in ["/fr/docs/nope/", "/de/", "/en/a/b/c", "/nothing-here"] {
        let res = client.get(path).await;
        assert_eq!(res.status(), 404, "{path}");
        assert!(res.text().contains("class=\"nf\""), "{path}");
    }
    assert!(
        client
            .get("/fr/oups/")
            .await
            .text()
            .contains("Ce carton est vide.")
    );
}

#[tokio::test]
async fn search_index_sitemap_and_robots() {
    let client = client().await;
    let index = client.get("/search/fr.txt").await;
    assert_eq!(index.status(), 200);
    let text = index.text();
    assert_eq!(
        text.lines().filter(|l| l.starts_with("p\t")).count(),
        PAGES.len()
    );
    assert!(text.contains("\t/fr/docs/security/\t"));
    assert_eq!(client.get("/search/de.txt").await.status(), 404);

    let sitemap = client
        .get("/sitemap.xml")
        .header("host", "example.com")
        .await
        .text();
    assert_eq!(sitemap.matches("<url>").count(), Route::all().len());
    assert!(sitemap.contains("<loc>http://example.com/es/docs/faq/</loc>"));
    let robots = client
        .get("/robots.txt")
        .header("host", "example.com")
        .await
        .text();
    assert!(robots.contains("Sitemap: http://example.com/sitemap.xml"));
}

#[tokio::test]
async fn static_files_are_served() {
    let client = client().await;
    let css = client.get("/site.css").await;
    assert_eq!(css.status(), 200);
    assert!(css.text().contains("--violet: #6d4aff"));
    let logo = client.get("/assets/logo.svg").await;
    assert_eq!(logo.status(), 200);
    assert!(logo.text().contains("<svg"));
    // The license is served by the site, identical to the repository's.
    let license = client.get("/LICENSE.txt").await;
    assert_eq!(license.status(), 200);
    assert_eq!(license.text(), include_str!("../../LICENSE"));
}

#[tokio::test]
async fn the_live_demo_works() {
    let client = client().await;
    let page = client.get("/demo").await;
    assert_eq!(page.status(), 200);
    assert!(page.text().contains("Try Paquets"));
    let res = client
        .post("/demo")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("title=Ship+the+site")
        .await;
    assert_eq!(res.status(), 303);
    assert!(client.get("/demo").await.text().contains("Ship the site"));
    let api = client.get("/api/todos").await;
    assert_eq!(api.status(), 200);
    assert_eq!(api.header("x-total-count"), Some("4"));
    assert_eq!(client.post("/demo/1/toggle").await.status(), 303);
    // The home page of the running server links to it.
    assert!(client.get("/en/").await.text().contains("href=\"/demo\""));
}

#[tokio::test]
async fn the_site_is_an_installable_app() {
    let client = client().await;
    // One manifest per language: the installed app opens in the language of the page.
    for lang in ["fr", "en", "es"] {
        let res = client.get(&format!("/{lang}/manifest.webmanifest")).await;
        assert_eq!(res.status(), 200);
        let manifest = res.text();
        assert!(manifest.contains(r#""name":"Paquets""#), "{manifest}");
        assert!(
            manifest.contains(&format!(r#""start_url":"/{lang}/""#)),
            "{manifest}"
        );
    }
    let worker = client.get("/sw.js").await.text();
    assert!(!worker.contains("{lang}"), "{worker}");
    assert!(worker.contains("/pkg/paquets_site_bg.wasm") || !std::path::Path::new("pkg").is_dir());
    assert!(
        worker.contains(r#"const NETWORK_ONLY = ["/api/todos"];"#),
        "{worker}"
    );
    let home = client.get("/fr/").await.text();
    assert!(home.contains(r#"<link rel="manifest" href="/fr/manifest.webmanifest"/>"#));
    let home = client.get("/es/").await.text();
    assert!(home.contains(r#"<link rel="manifest" href="/es/manifest.webmanifest"/>"#));
    assert!(home.contains(r#"navigator.serviceWorker.register("/sw.js")"#));
    assert_eq!(home.matches(r#"name="theme-color""#).count(), 1);
    assert_eq!(client.get("/paquets/icon-512.png").await.status(), 200);
}
