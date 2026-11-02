//! A field added to a model becomes a column at the next start, without a migration file.

use paquets::prelude::*;

mod v1 {
    use paquets::prelude::*;

    #[model(table = "notes")]
    pub struct Note {
        pub id: i64,
        pub title: String,
    }
}

mod v2 {
    use paquets::prelude::*;

    #[model(table = "notes")]
    pub struct Note {
        pub id: i64,
        pub title: String,
        pub tag: Option<String>,
    }
}

#[tokio::test]
async fn a_new_field_becomes_a_column_at_the_next_start() {
    let dir = std::env::temp_dir().join(format!(
        "paquets-push-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let url = format!("sqlite://{}", dir.join("app.db").display());

    // First version of the app: one note.
    let mut app = App::new();
    app.database(url.as_str());
    app.api::<v1::Note>("/api/notes");
    let c = app.test().await;
    let res = c
        .post("/api/notes")
        .json(&json!({ "title": "first" }))
        .await;
    assert!(res.status().is_success(), "{}", res.text());

    // The struct gained a field: the column is added at start, the old row reads as null.
    let mut app = App::new();
    app.database(url.as_str());
    app.api::<v2::Note>("/api/notes");
    let c = app.test().await;
    let res = c
        .post("/api/notes")
        .json(&json!({ "title": "second", "tag": "new" }))
        .await;
    assert!(res.status().is_success(), "{}", res.text());
    let all: serde_json::Value = c.get("/api/notes").await.json();
    let text = all.to_string();
    assert!(
        text.contains("\"first\"") && text.contains("\"new\""),
        "{text}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
