//! What the four packages added lately, reached through Paquets: Memoire's relations,
//! full-text search, indexes over several columns and transactions that retry; the options
//! of Vitesse (sessions, compression) behind the shield; Active's view transitions,
//! prefetching and 3D (`paquets::three`, `paquets::ecs`).

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

#[model(table = "accounts")]
struct Account {
    id: i64,
    balance: i64,
}

/// Moves `amount` from account 1 to account 2, all or nothing (Prisma's `$transaction`).
async fn transfer(req: Request) -> Result<String> {
    let amount: i64 = req.param("amount").unwrap_or("0").parse().unwrap_or(0);
    req.db()
        .transaction(|tx| {
            Box::pin(async move {
                let mut from = Account::find(tx, 1)
                    .await?
                    .ok_or(paquets::orm::Error::NotFound)?;
                let mut to = Account::find(tx, 2)
                    .await?
                    .ok_or(paquets::orm::Error::NotFound)?;
                from.balance -= amount;
                to.balance += amount;
                to.update(tx).await?;
                if from.balance < 0 {
                    // After a write: the transaction must undo it.
                    return Err(paquets::orm::Error::Validation("not enough money".into()));
                }
                from.update(tx).await?;
                Ok(())
            })
        })
        .await?;
    let balances = Account::all(req.db()).await?;
    Ok(balances
        .iter()
        .map(|a| a.balance.to_string())
        .collect::<Vec<_>>()
        .join(" "))
}

#[tokio::test]
async fn transactions_commit_together_or_not_at_all() {
    let mut app = App::new();
    app.model::<Account>();
    app.on_start(|db| async move {
        Account {
            id: 1,
            balance: 100,
        }
        .insert(db)
        .await?;
        Account { id: 2, balance: 0 }.insert(db).await?;
        Ok(())
    });
    app.post("/transfer/:amount", transfer);
    let client = app.test().await;
    assert_eq!(client.post("/transfer/30").await.text(), "70 30");
    // Too much: the credit already written is undone.
    let refused = client.post("/transfer/500").await;
    assert_ne!(refused.status(), 200);
    assert_eq!(client.post("/transfer/0").await.text(), "70 30");
}

#[tokio::test]
async fn pages_animate_and_prefetch_the_next_page() {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Home")
            .view_transitions()
            .prefetch(Prefetch::Hover)
            .body(a().href("/about").text("About"))
    });
    let client = app.test().await;
    let res = client.get("/").await;
    let html = res.text();
    assert!(
        html.contains(r#"<script type="speculationrules">"#),
        "{html}"
    );
    let policy = res.header("content-security-policy").unwrap().to_owned();
    // The rules may run: their hash is in the page's policy, behind the shield.
    let rules = html
        .split(r#"<script type="speculationrules">"#)
        .nth(1)
        .unwrap();
    let rules = &rules[..rules.find("</script>").unwrap()];
    let hash = paquets::securite::headers::ContentSecurityPolicy::hashes_for_inline_scripts(
        &format!("<script>{rules}</script>"),
    );
    assert!(policy.contains(&hash[0]), "{policy}");
}

#[test]
fn the_3d_of_active_runs_on_the_server_too() {
    use paquets::ecs::prelude::*;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins).add_systems(
        Startup,
        |mut commands: Commands<'_, '_>,
         mut meshes: ResMut<'_, Assets<Mesh>>,
         mut materials: ResMut<'_, Assets<StandardMaterial>>| {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            ));
            commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        },
    );
    app.update();
    let frame = RenderFrame::extract(app.world(), Vec2::new(800.0, 600.0)).unwrap();
    assert_eq!(frame.mesh_count(), 1);

    // And Three.js's way: a scene, a camera, what the renderer would draw.
    use paquets::three::{
        BoxGeometry, Mesh as ThreeMesh, MeshStandardMaterial, PerspectiveCamera, Scene,
    };
    let mut scene = Scene::new();
    scene.add(ThreeMesh::new(
        BoxGeometry::new(1.0, 1.0, 1.0),
        MeshStandardMaterial::new(),
    ));
    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0);
    assert_eq!(scene.frame(&camera, 1.0).meshes.len(), 1);
}
