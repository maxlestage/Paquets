//! The server: pages, a REST API and a database, with security built in.
//!
//!     paquets dev      # http://localhost:3000

use paquets::prelude::*;
{{use_counter}}
/// A table of the database, also sent and received as JSON.
#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

/// The data of the "new todo" form.
#[data]
struct NewTodo {
    title: String,
}

/// The home page, rendered on the server with its SEO tags.
async fn home(req: Request) -> Result<Page> {
    let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
    let list = ul().children(todos.into_iter().map(|todo| {
        li().text(if todo.done {
            format!("✅ {}", todo.title)
        } else {
            todo.title
        })
    }));
    Ok(Page::new("{{title}}")
        .description("A full-stack Rust app built with Paquets.")
        .head(r#"<link rel="icon" href="/favicon.svg"/>"#)
        .body(
            header()
                .child(strong().text("📦 {{title}}"))
                .child(nav().child(a().href("/api/todos").text("API"))),
        )
        .body(
            // `html::main()`: in this file, `main` is the program's function.
            html::main()
                .class("stack")
                .child(
                    section()
                        .class("hero")
                        .child(h1().text("Hello 👋"))
                        .child(p().text("Edit src/main.rs and reload the page.")),
                )
{{counter_card}}                .child(
                    div()
                        .class("card")
                        .child(h2().text("Todos"))
                        .child(list)
                        .child(
                            form()
                                .attr("method", "post")
                                .attr("action", "/todos")
                                .class("row")
                                .child(
                                    input()
                                        .attr("name", "title")
                                        .attr("placeholder", "Something to do")
                                        .attr("required", ""),
                                )
                                .child(button().text("Add")),
                        ),
                ),
        )
        .body(footer().text("Made with Paquets 📦")))
}

/// Adds a todo from the form, then goes back to the home page.
async fn add(req: Request) -> Result<Redirect> {
    let form: NewTodo = req.form().await?;
    let mut todo = Todo {
        id: 0,
        title: form.title,
        done: false,
    };
    todo.insert(req.db()).await?;
    Ok(Redirect::see_other("/"))
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", home);
    app.post("/todos", add);
    app.api::<Todo>("/api/todos"); // GET, POST, PUT, PATCH, DELETE
    app
}

fn main() -> std::io::Result<()> {
    app().run() // $PORT on Heroku, 3000 otherwise
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_page_and_api() {
        let client = app().test().await;
        assert_eq!(client.get("/").await.status(), 200);

        let res = client
            .post("/api/todos")
            .json(&json!({ "title": "Try Paquets", "done": false }))
            .await;
        assert_eq!(res.status(), 201);
        assert!(client.get("/").await.text().contains("Try Paquets"));
    }
}
