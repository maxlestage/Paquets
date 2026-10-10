//! An installable app (its own test binary: the manifest is global to the process).

use paquets::prelude::*;

#[model]
struct Note {
    id: i64,
    text: String,
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Notes").body(h1().text("Notes"))
    });
    app.api::<Note>("/api/notes");
    app.pwa(
        Pwa::new("My Notes")
            .short_name("Notes")
            .theme_color("#6d4aff"),
    );
    app
}

#[tokio::test]
async fn an_installable_app() {
    let client = app().test().await;

    let res = client.get("/manifest.webmanifest").await;
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.header("content-type"),
        Some("application/manifest+json")
    );
    let manifest: serde_json::Value = res.json();
    assert_eq!(manifest["name"], "My Notes");
    assert_eq!(manifest["short_name"], "Notes");
    assert_eq!(manifest["display"], "standalone");
    let icons = manifest["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 3, "the Paquets box, by default");
    for icon in icons {
        let res = client.get(icon["src"].as_str().unwrap()).await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.header("content-type"), Some("image/png"));
        assert_eq!(&res.bytes()[1..4], b"PNG");
    }

    let res = client.get("/sw.js").await;
    assert_eq!(res.status(), 200);
    assert!(
        res.header("content-type")
            .unwrap()
            .starts_with("text/javascript")
    );
    let worker = res.text();
    assert!(
        worker.contains(r#"const PRECACHE = ["/","/paquets/icon-192.png""#),
        "{worker}"
    );
    assert!(worker.contains(r#"const NETWORK_ONLY = ["/api/notes"];"#));
    assert!(worker.contains(r#"const PREFIX = "my-notes-";"#));

    let res = client.get("/").await;
    let html = res.text();
    assert!(html.contains(r#"<link rel="manifest" href="/manifest.webmanifest"/>"#));
    assert!(html.contains(r##"<meta name="theme-color" content="#6d4aff"/>"##));
    assert!(html.contains(r#"navigator.serviceWorker.register("/sw.js")"#));
    // The registration script is allowed by the page's policy.
    let csp = res.header("content-security-policy").unwrap();
    let hashes =
        paquets::securite::headers::ContentSecurityPolicy::hashes_for_inline_scripts(&html);
    assert_eq!(hashes.len(), 1);
    assert!(csp.contains(&hashes[0]), "{csp}");
}
