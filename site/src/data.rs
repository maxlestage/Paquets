//! The data of the site that does not depend on the language: links, the
//! four packages, the code samples (their comments are translated in
//! `i18n`, marked `§0`, `§1`… here) and the figures.

pub const GITHUB: &str = "https://github.com/maxlestage/Paquets";
pub const CRATES_IO: &str = "https://crates.io/crates/paquets";
pub const DOCS_RS: &str = "https://docs.rs/paquets";
pub const HEROKU_DEPLOY: &str = "https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets";
pub const HEROKU_BUTTON: &str = "https://www.herokucdn.com/deploy/button.svg";
pub const AUTHOR: &str = "https://github.com/maxlestage";

/// The command shown (and copied) in the hero.
pub const INSTALL: &str = "cargo install paquets";

/// One of the four packages Paquets is made of.
pub struct Package {
    pub name: &'static str,
    pub repo: &'static str,
    /// The letter on the faces of its cube.
    pub glyph: &'static str,
    /// Its accent color.
    pub color: &'static str,
    /// Its key figure (the label is translated).
    pub figure: &'static str,
    pub icon: &'static str,
}

pub const PACKAGES: [Package; 4] = [
    Package {
        name: "vitesse",
        repo: "https://github.com/maxlestage/Vitesse",
        glyph: "V",
        color: "#ffc24b",
        figure: "291k req/s",
        icon: "bolt",
    },
    Package {
        name: "securite",
        repo: "https://github.com/maxlestage/Securite",
        glyph: "S",
        color: "#ff4fa3",
        figure: "≈ 6 µs",
        icon: "shield",
    },
    Package {
        name: "memoire-orm",
        repo: "https://github.com/maxlestage/Memoire",
        glyph: "M",
        color: "#8b6dff",
        figure: "6 DB",
        icon: "db",
    },
    Package {
        name: "active",
        repo: "https://github.com/maxlestage/Active",
        glyph: "A",
        color: "#22d3ee",
        figure: "36.8 kB",
        icon: "spark",
    },
];

/// A code sample of the showcase: its file name and its code, where `§N`
/// is replaced by the N-th translated comment.
pub struct Sample {
    pub file: &'static str,
    pub code: &'static str,
}

pub const SAMPLES: [Sample; 5] = [
    Sample {
        file: "src/main.rs",
        code: r#"use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new(); // §0

    app.get("/", |_| async {
        Page::new("Home")
            .description("My site, in Rust.")
            .body(h1().text("Hello 👋"))
    });

    app.get("/hello/:name", |req: Request| async move {
        format!("Hello {}!", req.param("name").unwrap_or("you"))
    });

    app.run() // §1
}"#,
    },
    Sample {
        file: "src/main.rs",
        code: r#"use paquets::prelude::*;

#[model] // §0
struct Todo {
    id: i64, // §1
    title: String,
    done: bool,
}

async fn show(req: Request) -> Result<Json<Todo>> {
    let id: i64 = req.param_as("id")?; // §2
    let todo = Todo::get(req.db(), id).await?; // §3
    Ok(Json(todo))
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.api::<Todo>("/api/todos"); // §4
    app.get("/todos/:id", show);
    app.run()
}"#,
    },
    Sample {
        file: "src/pages.rs",
        code: r#"use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

// §0
pub async fn todos(req: Request) -> Result<Page> {
    let todos = Todo::query()
        .filter(Todo::DONE.eq(false))
        .order_by(Todo::ID.asc())
        .all(req.db())
        .await?;

    Ok(Page::new("Todos")
        .description("What is left to do.") // §1
        .body(main()
            .child(h1().text("Todos 📦"))
            .child(ul().children(todos.into_iter().map(|t| li().text(t.title))))))
}"#,
    },
    Sample {
        file: "src/lib.rs · src/main.rs",
        code: r#"// src/lib.rs: §0
use paquets::prelude::*;

pub fn counter(start: &str) -> Node {
    let n = use_state(start.parse::<i32>().unwrap_or(0));
    button()
        .on_click(move |_| n.update(|n| *n += 1))
        .text_dyn(move || format!("Clicked {} times", n.get()))
        .into()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]); // §1
}

// src/main.rs: §2
use my_app::counter;
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Counter").body(section().child(island("counter", "0", counter)))
    });
    app.run()
}"#,
    },
    Sample {
        file: "tests/app.rs",
        code: r#"use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", |_| async { Page::new("Home").body(h1().text("Hello")) });
    app.api::<Todo>("/api/todos");
    app
}

#[tokio::test]
async fn the_whole_stack() {
    let client = app().test().await; // §0
    assert_eq!(client.get("/").await.status(), 200);

    let res = client
        .post("/api/todos")
        .json(&json!({ "title": "Ship it", "done": false }))
        .await;
    assert_eq!(res.status(), 201); // §1
    assert_eq!(res.json::<Todo>().title, "Ship it");
}"#,
    },
];

/// Replaces the `§N` markers of `code` with `// comment`s (the marker is
/// already preceded by `// ` in the code).
pub fn localize(code: &str, comments: &[&str]) -> String {
    let mut out = String::with_capacity(code.len() + 200);
    let mut rest = code;
    while let Some(at) = rest.find('§') {
        out.push_str(&rest[..at]);
        let after = &rest[at + '§'.len_utf8()..];
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        let index: usize = after[..digits].parse().unwrap_or(usize::MAX);
        out.push_str(comments.get(index).copied().unwrap_or(""));
        rest = &after[digits..];
    }
    out.push_str(rest);
    out
}

/// The number of `§N` markers in `code`.
#[cfg(test)]
pub fn markers(code: &str) -> usize {
    code.matches('§').count()
}

/// The icon and the code tag of each feature.
pub const FEATURES: [(&str, &str); 8] = [
    ("box", "use paquets::prelude::*;"),
    ("shield", "App::new()"),
    ("db", "DATABASE_URL"),
    ("page", "Page::new(\"Home\")"),
    ("spark", "island(\"counter\", ..)"),
    ("phone", "git push"),
    ("flask", "app().test().await"),
    ("crate", "cargo add paquets"),
];

/// The figures: value, decimals, prefix, suffix.
pub const NUMBERS: [(f64, usize, &str, &str); 6] = [
    (291.0, 0, "", "k"),
    (2.78, 2, "", "M"),
    (6.0, 0, "≈ ", " µs"),
    (36.8, 1, "", " kB"),
    (6.0, 0, "", ""),
    (3.0, 0, "", ""),
];

/// The command of each step of "from zero to production".
pub const JOURNEY: [&str; 4] = [
    "paquets new my-app",
    "paquets dev",
    "git push",
    "heroku.yml",
];

/// The code marquee.
pub const MARQUEE_CODE: [&str; 8] = [
    "use paquets::prelude::*;",
    "App::new()",
    "#[model]",
    "app.api::<Todo>(\"/api/todos\")",
    "Page::new(\"Home\")",
    "island(\"counter\", \"0\", counter)",
    "app().test().await",
    "app.run()",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localizes_comments() {
        let code = "let a = 1; // §0\nlet b = 2; // §1";
        assert_eq!(
            localize(code, &["one", "two"]),
            "let a = 1; // one\nlet b = 2; // two"
        );
        assert_eq!(markers(code), 2);
        // A marker without a translation disappears.
        assert_eq!(localize("x // §3", &[]), "x // ");
    }

    #[test]
    fn samples_use_the_public_api() {
        for sample in &SAMPLES {
            assert!(sample.code.contains("use paquets::prelude::*;"));
            // `main()` (the <main> element) cannot be called in a file that
            // defines `fn main` (the function would shadow it).
            if sample.code.contains("fn main()") {
                assert!(!sample.code.contains(" main()\n"), "{}", sample.file);
                assert!(!sample.code.contains("(main()"), "{}", sample.file);
            }
        }
    }
}
