//! The whole stack, through the in-memory test client: routing (vitesse), security
//! (securite), database and REST API (memoire-orm), pages (active).

use paquets::prelude::*;

#[model(table = "todos")]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

#[model]
struct Member {
    id: i64,
    #[memoire(unique)]
    email: String,
    nickname: Option<String>,
}

#[data]
struct Greeting {
    name: String,
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Home")
            .description("The home page")
            .body(main().child(h1().text("Hello")))
    });
    app.get("/hello/:name", |req: Request| async move {
        format!("Hello {}!", req.param("name").unwrap_or("you"))
    });
    app.post("/greet", |req: Request| async move {
        let greeting: Greeting = req.json().await?;
        Ok::<_, Error>(Json(
            json!({ "message": format!("Hello {}", greeting.name) }),
        ))
    });
    app.get("/todos/count", |req: Request| async move {
        Ok::<_, Error>(Todo::count(req.db()).await?.to_string())
    });
    app.get("/todos/:id/title", |req: Request| async move {
        let id: i64 = req.param_as("id")?;
        Ok::<_, Error>(Todo::get(req.db(), id).await?.title)
    });
    app.api::<Todo>("/api/todos");
    app.api::<Member>("/api/members");
    app
}

#[tokio::test]
async fn pages_are_complete_and_protected() {
    let client = app().test().await;
    let res = client.get("/").await;
    assert_eq!(res.status(), 200);
    let html = res.text();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Home</title>"));
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(res.header("content-type").unwrap().starts_with("text/html"));
    // The page's own policy, with the hashes of its inline styles.
    let csp = res.header("content-security-policy").unwrap();
    assert!(csp.contains("'sha256-"), "{csp}");
    // securite's headers on every response.
    assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(res.header("x-frame-options"), Some("DENY"));
    assert!(res.header("x-request-id").is_some());
}

#[tokio::test]
async fn routes_and_json() {
    let client = app().test().await;
    assert_eq!(client.get("/hello/Ada").await.text(), "Hello Ada!");
    let res = client.post("/greet").json(&json!({ "name": "Ada" })).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.json::<serde_json::Value>()["message"], "Hello Ada");
    let res = client.post("/greet").json(&json!({ "nope": 1 })).await;
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn rest_api_crud() {
    let client = app().test().await;

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Write docs", "done": false }))
        .await;
    assert_eq!(res.status(), 201, "{}", res.text());
    let created: Todo = res.json();
    assert_eq!(created.id, 1);
    client
        .post("/api/todos")
        .json(&json!({ "title": "Ship", "done": false }))
        .await;

    let res = client.get("/api/todos").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.header("x-total-count"), Some("2"));
    let all: Vec<Todo> = res.json();
    assert_eq!(
        all.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
        ["Write docs", "Ship"]
    );

    let page: Vec<Todo> = client.get("/api/todos?limit=1&offset=1").await.json();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].title, "Ship");

    let res = client.get("/api/todos/1").await;
    assert_eq!(res.json::<Todo>().title, "Write docs");
    assert_eq!(client.get("/api/todos/99").await.status(), 404);
    assert_eq!(client.get("/api/todos/abc").await.status(), 400);

    let res = client
        .patch("/api/todos/1")
        .json(&json!({ "done": true }))
        .await;
    assert_eq!(res.status(), 200, "{}", res.text());
    let patched: Todo = res.json();
    assert!(patched.done);
    assert_eq!(patched.title, "Write docs");

    let res = client
        .put("/api/todos/2")
        .json(&json!({ "title": "Ship v1", "done": true }))
        .await;
    assert_eq!(res.json::<Todo>().title, "Ship v1");
    assert_eq!(
        client
            .put("/api/todos/42")
            .json(&json!({ "title": "x", "done": true }))
            .await
            .status(),
        404
    );

    assert_eq!(client.get("/todos/count").await.text(), "2");
    assert_eq!(client.get("/todos/2/title").await.text(), "Ship v1");
    assert_eq!(client.get("/todos/7/title").await.status(), 404);

    assert_eq!(client.delete("/api/todos/1").await.status(), 204);
    assert_eq!(client.delete("/api/todos/1").await.status(), 404);
    assert_eq!(client.get("/todos/count").await.text(), "1");
}

#[tokio::test]
async fn unique_values_conflict() {
    let client = app().test().await;
    let body = json!({ "email": "ada@example.com" });
    assert_eq!(client.post("/api/members").json(&body).await.status(), 201);
    let res = client.post("/api/members").json(&body).await;
    assert_eq!(res.status(), 409, "{}", res.text());
}

#[tokio::test]
async fn attacks_are_blocked() {
    let client = app().test().await;
    let res = client.get("/hello/x?q=1%27%20OR%20%271%27=%271").await;
    assert_eq!(res.status(), 403, "{}", res.text());
    let res = client.get("/").header("user-agent", "sqlmap/1.7").await;
    assert_eq!(res.status(), 403);
    // A cross-site form post is refused (CSRF), a same-origin one is not.
    let res = client
        .post("/api/todos")
        .header("sec-fetch-site", "cross-site")
        .json(&json!({ "title": "x", "done": false }))
        .await;
    assert_eq!(res.status(), 403);
    let res = client
        .post("/api/todos")
        .header("sec-fetch-site", "same-origin")
        .json(&json!({ "title": "x", "done": false }))
        .await;
    assert_eq!(res.status(), 201);
}

#[tokio::test]
async fn security_can_be_turned_off() {
    let mut app = app();
    app.security(None);
    let client = app.test().await;
    let res = client.get("/").header("user-agent", "sqlmap/1.7").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.header("x-frame-options"), None);
}

#[tokio::test]
async fn startup_hooks_seed_the_database() {
    let mut app = app();
    app.on_start(|db| async move {
        Todo {
            id: 0,
            title: "Seeded".into(),
            done: false,
        }
        .insert(db)
        .await?;
        Ok(())
    });
    let client = app.test().await;
    assert_eq!(client.get("/todos/1/title").await.text(), "Seeded");
}

#[tokio::test]
async fn custom_error_format() {
    let mut app = app();
    app.on_error(|err: Error| res::status(err.status()).json(json!({ "problem": err.message() })));
    let client = app.test().await;
    let res = client.get("/api/todos/99").await;
    assert_eq!(res.status(), 404);
    assert_eq!(
        res.json::<serde_json::Value>()["problem"],
        "record not found"
    );
}
