//! An installable app in several languages (its own test binary: the manifest is global
//! to the process): the installed app opens in the language of the page.

use paquets::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.get("/:lang/", |req: Request| async move {
        let lang = req.param("lang").unwrap_or("en").to_owned();
        Page::new("Hello").lang(lang).body(h1().text("Hello"))
    });
    app.pwa(Pwa::new("Hello").start_url("/{lang}/"));
    app
}

#[tokio::test]
async fn each_language_has_its_manifest() {
    let client = app().test().await;

    let html = client.get("/fr/").await.text();
    assert!(
        html.contains(r#"<link rel="manifest" href="/fr/manifest.webmanifest"/>"#),
        "{html}"
    );

    let res = client.get("/fr/manifest.webmanifest").await;
    assert_eq!(res.status(), 200);
    let manifest: serde_json::Value = res.json();
    assert_eq!(manifest["start_url"], "/fr/");
    let es: serde_json::Value = client.get("/es/manifest.webmanifest").await.json();
    assert_eq!(es["start_url"], "/es/");
    // Without a language, the English start page.
    let default: serde_json::Value = client.get("/manifest.webmanifest").await.json();
    assert_eq!(default["start_url"], "/en/");
    assert_eq!(
        client.get("/f%22r/manifest.webmanifest").await.status(),
        404
    );

    // The worker keeps the pages visited; no placeholder left in it.
    let worker = client.get("/sw.js").await.text();
    assert!(!worker.contains("{lang}"), "{worker}");
}
