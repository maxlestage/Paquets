//! The live demo of the website, the same as `examples/todo.rs`: a to-do list as a page
//! (server-rendered, with forms) and as a REST API, stored in the database of
//! `DATABASE_URL` (in memory by default). Only the running server has it: the static
//! export for GitHub Pages leaves it out.

use paquets::prelude::*;

#[model(table = "todos")]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[data]
struct NewTodo {
    title: String,
}

/// The to-do list, rendered on the server.
async fn demo(req: Request) -> Result<Page> {
    let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
    let left = todos.iter().filter(|t| !t.done).count();
    let rows = todos.into_iter().map(|todo| {
        let action = format!("/demo/{}/toggle", todo.id);
        li().child(
            form()
                .attr("method", "post")
                .attr("action", action)
                .class("row")
                .child(
                    button()
                        .class("ghost")
                        .attr("aria-label", "toggle")
                        .text(if todo.done { "✅" } else { "⬜" }),
                )
                .child(if todo.done {
                    s().text(todo.title)
                } else {
                    span().text(todo.title)
                }),
        )
    });
    Ok(Page::new("To-do list — Paquets demo")
        .description("A to-do list built with Paquets: a server-rendered page and a REST API.")
        .head(r#"<link rel="icon" href="/assets/logo.svg" type="image/svg+xml"/>"#)
        .body(
            header()
                .child(strong().text("📦 Paquets demo"))
                .child(
                    nav()
                        .child(a().href("/").text("Website"))
                        .child(a().href("/api/todos").text("API")),
                ),
        )
        .body(
            html::main()
                .class("stack")
                .child(h1().text("To-do list"))
                .child(p().class("muted").text(format!("{left} left")))
                .child(
                    div()
                        .class("card stack")
                        .child(
                            ul().attr("style", "list-style:none;padding:0")
                                .children(rows),
                        )
                        .child(
                            form()
                                .attr("method", "post")
                                .attr("action", "/demo")
                                .class("row")
                                .child(
                                    input()
                                        .attr("name", "title")
                                        .attr("placeholder", "Something to do")
                                        .attr("required", "")
                                        .attr("maxlength", "200")
                                        .attr("style", "flex:1"),
                                )
                                .child(button().text("Add")),
                        ),
                )
                .child(p().class("muted").text(
                    "The same data as JSON: GET/POST /api/todos, GET/PUT/PATCH/DELETE /api/todos/:id.",
                )),
        )
        .body(footer().text(
            "Server: vitesse · Security: securite · Database: memoire-orm · Pages: active",
        )))
}

async fn add(req: Request) -> Result<Redirect> {
    let form: NewTodo = req.form().await?;
    let title = form.title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err(Error::bad_request("a title has 1 to 200 characters"));
    }
    let mut todo = Todo {
        id: 0,
        title: title.to_owned(),
        done: false,
    };
    todo.insert(req.db()).await?;
    Ok(Redirect::see_other("/demo"))
}

async fn toggle(req: Request) -> Result<Redirect> {
    let id: i64 = req.param_as("id")?;
    let mut todo = Todo::get(req.db(), id).await?;
    todo.done = !todo.done;
    todo.update(req.db()).await?;
    Ok(Redirect::see_other("/demo"))
}

/// Adds the demo to the app: `/demo`, its forms, `/api/todos`, and three to-dos in an
/// empty database.
pub fn mount(app: &mut App) {
    app.get("/demo", demo);
    app.post("/demo", add);
    app.post("/demo/:id/toggle", toggle);
    app.api::<Todo>("/api/todos");
    app.on_start(|db| async move {
        if Todo::count(db).await? == 0 {
            for title in ["Try Paquets", "Read the docs", "Deploy from my phone"] {
                let mut todo = Todo {
                    id: 0,
                    title: title.into(),
                    done: false,
                };
                todo.insert(db).await?;
            }
        }
        Ok(())
    });
}
