//! A complete app: a to-do list as a page (server-rendered, with a form) and as a
//! REST API, stored in the database of `DATABASE_URL` (in memory by default).
//!
//!     cargo run --example todo                                  # http://localhost:3000
//!     DATABASE_URL=sqlite://todo.db cargo run --example todo    # kept in a file
//!
//! The website of the repository (`site/`) includes the same demo at `/demo`.

use paquets::prelude::*;

#[model(table = "todos")]
struct Todo {
    id: i64,
    title: String,
    done: bool,
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
    Ok(Page::new("To-do list")
        .description("A to-do list built with Paquets: a server-rendered page and a REST API.")
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
                        .child(ul().attr("style", "list-style:none;padding:0").children(rows))
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
                                        .attr("style", "flex:1"),
                                )
                                .child(button().text("Add")),
                        ),
                )
                .child(
                    p().class("muted")
                        .text("The same data as JSON: GET/POST /api/todos, GET/PUT/PATCH/DELETE /api/todos/:id."),
                ),
        )
        .body(footer().text("Server: vitesse · Security: securite · Database: memoire-orm · Pages: active")))
}

async fn add(req: Request) -> Result<Redirect> {
    let form: NewTodo = req.form().await?;
    let title = form.title.trim();
    if title.is_empty() || title.len() > 200 {
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

fn app() -> App {
    let mut app = App::new();
    // Without a website in public/, the home page is the demo.
    app.get("/", |_| async { Redirect::to("/demo") });
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
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_demo_works() {
        let client = app().test().await;
        let page = client.get("/demo").await;
        assert_eq!(page.status(), 200);
        assert!(page.text().contains("Try Paquets"));

        let res = client
            .post("/demo")
            .header("content-type", "application/x-www-form-urlencoded")
            .body("title=Write+tests")
            .await;
        assert_eq!(res.status(), 303);
        assert!(client.get("/demo").await.text().contains("Write tests"));

        assert_eq!(client.post("/demo/1/toggle").await.status(), 303);
        let todo: Todo = client.get("/api/todos/1").await.json();
        assert!(todo.done);
    }
}
