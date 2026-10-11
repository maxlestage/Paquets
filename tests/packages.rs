//! What the four packages added lately, reached through Paquets: Memoire's relations,
//! full-text search and indexes over several columns, and the options of Vitesse
//! (sessions, compression) behind the shield.

use paquets::prelude::*;

#[model(table = "authors")]
struct Author {
    id: i64,
    name: String,
}

#[model(table = "books")]
#[memoire(index(author_id, year))]
struct Book {
    id: i64,
    #[memoire(references = "authors(id)")]
    author_id: i64,
    #[memoire(fulltext)]
    title: String,
    year: i32,
}

fn library() -> App {
    let mut app = App::new();
    app.model::<Author>();
    app.model::<Book>();
    app.on_start(|db| async move {
        Author::insert_many(
            db,
            &[
                Author {
                    id: 1,
                    name: "Ursula".into(),
                },
                Author {
                    id: 2,
                    name: "Terry".into(),
                },
            ],
        )
        .await?;
        Book::insert_many(
            db,
            &[
                Book {
                    id: 1,
                    author_id: 1,
                    title: "The Left Hand of Darkness".into(),
                    year: 1969,
                },
                Book {
                    id: 2,
                    author_id: 2,
                    title: "Small Gods".into(),
                    year: 1992,
                },
                Book {
                    id: 3,
                    author_id: 1,
                    title: "The Dispossessed".into(),
                    year: 1974,
                },
            ],
        )
        .await?;
        Ok(())
    });
    // Every author with their books: two queries, whatever the number of authors.
    app.get("/authors", |req: Request| async move {
        let authors = Author::query()
            .order_by(Author::ID.asc())
            .all(req.db())
            .await?;
        let books = authors.load_children::<Book>(req.db()).await?;
        let lines: Vec<String> = authors
            .iter()
            .zip(books)
            .map(|(author, books)| {
                let titles: Vec<String> = books.into_iter().map(|b| b.title).collect();
                format!("{}: {}", author.name, titles.join(", "))
            })
            .collect();
        Ok::<_, Error>(lines.join("\n"))
    });
    app.get("/search", |req: Request| async move {
        let words = req.query("q").unwrap_or_default();
        let books = Book::query()
            .filter(Book::TITLE.matches(words))
            .order_by(Book::ID.asc())
            .all(req.db())
            .await?;
        Ok::<_, Error>(Json(books))
    });
    app
}

#[tokio::test]
async fn relations_without_one_query_per_row() {
    let client = library().test().await;
    assert_eq!(
        client.get("/authors").await.text(),
        "Ursula: The Left Hand of Darkness, The Dispossessed\nTerry: Small Gods"
    );
}

#[tokio::test]
async fn full_text_search() {
    let client = library().test().await;
    let found: Vec<Book> = client.get("/search?q=the%20DARKNESS").await.json();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].title, "The Left Hand of Darkness");
    // Whole words only: "dark" is not "darkness".
    let found: Vec<Book> = client.get("/search?q=dark").await.json();
    assert!(found.is_empty());
}

#[cfg(feature = "session")]
#[tokio::test]
async fn sessions_behind_the_shield() {
    use paquets::vitesse::session::MemoryStore;

    let mut app = App::new();
    app.cookie_secret("0123456789abcdef0123456789abcdef-test");
    app.middleware(middleware::session(MemoryStore::new()));
    app.get("/visits", |req: Request| async move {
        let session = req.session();
        let visits = session.get::<u32>("visits").unwrap_or(0) + 1;
        session.insert("visits", visits)?;
        Ok::<_, Error>(visits.to_string())
    });
    let client = app.test().await;

    let first = client.get("/visits").await;
    assert_eq!(first.text(), "1");
    let set_cookie = first.header("set-cookie").unwrap();
    // The shield hardens the session cookie like every other.
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    let cookie = set_cookie.split(';').next().unwrap();
    assert_eq!(
        client.get("/visits").header("cookie", cookie).await.text(),
        "2"
    );
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compressed_pages() {
    let mut app = App::new();
    app.middleware(middleware::compress());
    app.get("/", |_| async {
        Page::new("Home").body(p().text("Paquets ".repeat(500)))
    });
    let client = app.test().await;

    let res = client.get("/").header("accept-encoding", "gzip, br").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.header("content-encoding"), Some("br"));
    // The shield's headers are still there.
    assert!(res.header("content-security-policy").is_some());
    let res = client.get("/").await;
    assert_eq!(res.header("content-encoding"), None);
}
