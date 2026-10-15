//! `app.admin(…)`, the API documentation, health checks, metrics, migrations and the
//! route list.

use paquets::prelude::*;

#[model(table = "products")]
struct Product {
    id: i64,
    name: String,
    price: f64,
    available: bool,
    note: Option<String>,
    #[memoire(unique)]
    code: String,
    api_token: String,
}

fn app() -> App {
    let mut app = App::new();
    app.admin(
        Admin::new()
            .title("Back office")
            .per_page(2)
            .model::<Product>()
            .allow(|req: &Request| req.header("x-admin") == Some("yes")),
    );
    app.api::<Product>("/api/products");
    app.api_docs("Shop API");
    app.health("/health");
    app.metrics("/metrics");
    app
}

fn product(name: &str, code: &str, available: bool) -> Vec<(&'static str, String)> {
    let mut form = vec![
        ("name", name.to_owned()),
        ("price", "9.5".to_owned()),
        ("code", code.to_owned()),
        ("note", String::new()),
    ];
    if available {
        form.push(("available", "true".to_owned()));
    }
    form
}

#[tokio::test]
async fn the_admin_panel_lists_creates_edits_and_deletes() {
    let client = app().test().await;
    assert_eq!(client.get("/admin").await.status(), 403);
    let admin = |path: &str| client.get(path).header("x-admin", "yes");

    let res = admin("/admin").await;
    assert_eq!(res.status(), 200);
    assert!(res.text().contains("Back office"));
    assert!(res.text().contains(r#"href="/admin/products""#));

    let res = admin("/admin/products/new").await;
    assert_eq!(res.status(), 200);
    let html = res.text();
    assert!(html.contains(r#"name="price""#) && html.contains(r#"step="any""#));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(
        !html.contains(r#"name="api_token""#),
        "secrets are not editable"
    );

    for (name, code, available) in [
        ("Apple", "A1", true),
        ("Banana", "B2", false),
        ("Cherry", "C3", true),
    ] {
        let res = client
            .post("/admin/products/new")
            .header("x-admin", "yes")
            .form(&product(name, code, available))
            .await;
        assert_eq!(res.status(), 303, "{}", res.text());
        assert_eq!(
            res.header("location"),
            Some("/admin/products?notice=created")
        );
    }
    // Validation errors and unique values come back in the form.
    let mut invalid = product("Durian", "D4", true);
    invalid[1].1 = "cheap".into();
    let res = client
        .post("/admin/products/new")
        .header("x-admin", "yes")
        .form(&invalid)
        .await;
    assert_eq!(res.status(), 422);
    assert!(res.text().contains("Enter a number"));
    let res = client
        .post("/admin/products/new")
        .header("x-admin", "yes")
        .form(&product("Again", "A1", true))
        .await;
    assert_eq!(res.status(), 422);
    assert!(res.text().contains("already used"), "{}", res.text());

    // Pages of two rows, search, sort and filters.
    let html = admin("/admin/products?notice=created").await.text();
    assert!(html.contains("The row was created."));
    assert!(html.contains("Apple") && html.contains("Banana") && !html.contains("Cherry"));
    assert!(html.contains("3 row(s) · page 1 of 2"));
    let html = admin("/admin/products?page=2").await.text();
    assert!(html.contains("Cherry") && !html.contains("Apple"));
    let html = admin("/admin/products?q=an").await.text();
    assert!(html.contains("Banana") && !html.contains("Apple"));
    let html = admin("/admin/products?sort=name&dir=desc").await.text();
    assert!(html.find("Cherry").unwrap() < html.find("Banana").unwrap());
    let html = admin("/admin/products?available=false").await.text();
    assert!(html.contains("Banana") && !html.contains("Apple"));

    // Edit.
    let html = admin("/admin/products/1").await.text();
    assert!(html.contains(r#"value="Apple""#));
    let mut changed = product("Green apple", "A1", false);
    changed.push(("note", "crunchy".into()));
    let res = client
        .post("/admin/products/1")
        .header("x-admin", "yes")
        .form(&changed)
        .await;
    assert_eq!(res.status(), 303);
    let api: serde_json::Value = client.get("/api/products/1").await.json();
    assert_eq!(api["name"], "Green apple");
    assert_eq!(api["available"], false);

    // Delete, after a confirmation.
    assert_eq!(admin("/admin/products/2/delete").await.status(), 200);
    let res = client
        .post("/admin/products/2/delete")
        .header("x-admin", "yes")
        .await;
    assert_eq!(res.status(), 303);
    assert_eq!(client.get("/api/products/2").await.status(), 404);
    assert_eq!(admin("/admin/products/99").await.status(), 404);
}

#[tokio::test]
async fn the_api_is_documented_in_one_line() {
    let client = app().test().await;
    let res = client.get("/openapi.json").await;
    assert_eq!(res.status(), 200);
    let doc: serde_json::Value = res.json();
    assert_eq!(doc["info"]["title"], "Shop API");
    let schema = &doc["components"]["schemas"]["Product"];
    assert_eq!(schema["properties"]["price"]["type"], "number");
    assert_eq!(
        schema["properties"]["note"]["type"],
        json!(["string", "null"])
    );
    assert!(doc["paths"]["/api/products/{id}"]["patch"].is_object());
    assert_eq!(
        doc["paths"]["/api/products"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/Product"
    );
    // The admin pages are not API routes.
    assert!(doc["paths"].get("/admin").is_none());
    assert!(doc["paths"]["/health"]["get"].is_object());

    let res = client.get("/docs").await;
    assert_eq!(res.status(), 200);
    assert!(res.text().contains("SwaggerUIBundle"));
    let csp = res.header("content-security-policy").unwrap();
    assert!(csp.contains("https://cdn.jsdelivr.net"), "{csp}");
}

#[tokio::test]
async fn health_and_metrics() {
    let client = app().test().await;
    let res = client.get("/health").await;
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"]["kind"], "memory");
    // Metrics: for this machine (the test client is local), or with a token.
    let res = client.get("/metrics").await;
    assert_eq!(res.status(), 200);
    assert!(
        res.text()
            .contains("paquets_requests_total{status=\"2xx\"}")
    );
    let res = client
        .get("/metrics")
        .header("x-forwarded-for", "203.0.113.7")
        .await;
    assert_eq!(res.status(), 403);
}

#[test]
fn routes_are_listed() {
    let routes = app().routes();
    let has =
        |method: &str, path: &str| routes.iter().any(|r| r.method == method && r.path == path);
    assert!(has("GET", "/admin/products"));
    assert!(has("POST", "/admin/products/:id/delete"));
    assert!(has("DELETE", "/api/products/:id"));
    assert!(has("GET", "/health"));
    let admin = routes.iter().find(|r| r.path == "/admin").unwrap();
    assert!(admin.page);
}

#[tokio::test]
async fn migrations_run_once_at_startup() {
    #[model(table = "notes")]
    struct Note {
        id: i64,
        text: String,
    }
    let mut app = App::new();
    app.migrate(Migrator::new().add(Migration::new(1, "create_notes").create_table::<Note>()));
    app.get("/notes", |req: Request| async move {
        Ok::<_, Error>(Note::count(req.db()).await?.to_string())
    });
    let client = app.test().await;
    let res = client.get("/notes").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.text(), "0");
}
