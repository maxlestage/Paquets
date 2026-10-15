//! An admin panel for your models, like Django's admin: lists with search, sort,
//! filters and pages, forms to create and edit rows, and deletion — generated from
//! the models' schemas, nothing to write.
//!
//! ```
//! use paquets::prelude::*;
//!
//! #[model]
//! struct Todo {
//!     id: i64,
//!     title: String,
//!     done: bool,
//! }
//!
//! let mut app = App::new();
//! app.admin(Admin::new().model::<Todo>()); // http://localhost:3000/admin
//! ```
//!
//! **Who gets in.** By default: the administrators of [`App::auth`](crate::App::auth)
//! ([`User::admin`](crate::auth::User::admin)); in an app without `App::auth`, in
//! development builds only, requests from this machine (so that `paquets dev` opens
//! it right away; never in release builds, never through a proxy). Choose
//! yourself with [`Admin::allow`] or a securite [`Gate`] ([`Admin::gate`]).
//!
//! | Route | Page |
//! |---|---|
//! | `GET /admin` | every model, with its number of rows |
//! | `GET /admin/<table>?q=&sort=&dir=&page=` | the rows: search in text columns, sort, filter booleans, pages |
//! | `GET/POST /admin/<table>/new` | creates a row |
//! | `GET/POST /admin/<table>/:id` | edits a row |
//! | `GET/POST /admin/<table>/:id/delete` | deletes a row, after a confirmation |
//!
//! Generated keys, timestamps and binary columns are read-only; columns whose name
//! contains `password`, `secret` or `token` are never shown.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use active::Element;
use active::html::{
    a, button, div, form, h1, h2, input, label, li, nav, p, span, strong, table, tbody, td,
    textarea, th, thead, tr, ul,
};
use memoire_orm::{
    Column, ColumnDef, ColumnType, Expr, IntoValue, Model, TableSchema, Value, ValueRow,
};
use securite::authz::Gate;
use vitesse::openapi::Operation;
use vitesse::{BoxFuture, IntoResponse, Request, Response, StatusCode, json};

use crate::auth::{AuthExt, User, encode_component};
use crate::{Db, Page, RequestExt};

type Allow = Arc<dyn Fn(&Request) -> bool + Send + Sync>;
type CountFn = fn(&'static Db) -> BoxFuture<memoire_orm::Result<u64>>;
type RegisterFn = fn(&mut vitesse::App, &'static AdminState, usize);
pub(crate) type CreateFn = fn(&'static Db) -> BoxFuture<memoire_orm::Result<()>>;

struct Entry {
    table: &'static str,
    label: String,
    count: CountFn,
    register: RegisterFn,
    create: CreateFn,
}

/// The configuration of [`App::admin`](crate::App::admin).
#[must_use]
pub struct Admin {
    path: String,
    title: String,
    per_page: u64,
    entries: Vec<Entry>,
    allow: Option<Allow>,
}

impl Default for Admin {
    fn default() -> Self {
        Admin::new()
    }
}

impl fmt::Debug for Admin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Admin")
            .field("path", &self.path)
            .field("title", &self.title)
            .field(
                "models",
                &self.entries.iter().map(|e| e.table).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl Admin {
    /// An admin panel at `/admin`, 25 rows per page.
    pub fn new() -> Self {
        Admin {
            path: "/admin".into(),
            title: "Admin".into(),
            per_page: 25,
            entries: Vec::new(),
            allow: None,
        }
    }

    /// Another address than `/admin`.
    pub fn path(mut self, path: &str) -> Self {
        self.path = match path.trim_end_matches('/') {
            "" => "/admin".to_owned(),
            path => path.to_owned(),
        };
        self
    }

    /// The title of the pages (`Admin`).
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /// Rows per page (25).
    pub fn per_page(mut self, rows: u64) -> Self {
        self.per_page = rows.clamp(1, 500);
        self
    }

    /// Manages a model: its table gets a list, forms and deletion (and is created
    /// at startup, like with [`App::model`](crate::App::model)).
    pub fn model<M>(mut self) -> Self
    where
        M: Model,
        M::Key: FromStr + IntoValue + Send,
    {
        fn count<M: Model>(db: &'static Db) -> BoxFuture<memoire_orm::Result<u64>> {
            Box::pin(M::count(db))
        }
        fn create<M: Model>(db: &'static Db) -> BoxFuture<memoire_orm::Result<()>> {
            use memoire_orm::Executor as _;
            Box::pin(db.create_table_if_not_exists::<M>())
        }
        self.entries.push(Entry {
            table: M::SCHEMA.name,
            label: crate::docs::type_name::<M>().to_owned(),
            count: count::<M>,
            register: register::<M>,
            create: create::<M>,
        });
        self
    }

    /// Who may use the panel, instead of the default rule.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// let admin = Admin::new().allow(|req: &Request| {
    ///     req.user().is_some_and(|user| user.email.ends_with("@example.com"))
    /// });
    /// ```
    pub fn allow<F>(mut self, allow: F) -> Self
    where
        F: Fn(&Request) -> bool + Send + Sync + 'static,
    {
        self.allow = Some(Arc::new(allow));
        self
    }

    /// Lets in the logged-in users that a securite [`Gate`] allows to `ability`.
    ///
    /// ```
    /// use paquets::prelude::*;
    /// use paquets::auth::User;
    /// use paquets::securite::authz::Gate;
    ///
    /// let gate = Gate::<User>::new().define("admin", |user| user.admin);
    /// let admin = Admin::new().gate(gate, "admin");
    /// ```
    pub fn gate(self, gate: Gate<User>, ability: &str) -> Self {
        let ability = ability.to_owned();
        self.allow(move |req: &Request| req.user().is_some_and(|u| gate.allows(u, &ability)))
    }
}

/// The panel, shared by its routes.
pub(crate) struct AdminState {
    path: String,
    title: String,
    per_page: u64,
    models: Vec<(&'static str, String, CountFn)>,
    allow: Option<Allow>,
}

/// The routes of the panel. Returns the models' table creators to the app.
pub(crate) fn install(app: &mut vitesse::App, admin: Admin) -> Vec<CreateFn> {
    let state: &'static AdminState = Box::leak(Box::new(AdminState {
        path: admin.path.clone(),
        title: admin.title.clone(),
        per_page: admin.per_page,
        models: admin
            .entries
            .iter()
            .map(|e| (e.table, e.label.clone(), e.count))
            .collect(),
        allow: admin.allow.clone(),
    }));
    app.get(&state.path, move |req: Request| dashboard(req, state))
        .doc(page_doc("Admin: dashboard"));
    for (index, entry) in admin.entries.iter().enumerate() {
        (entry.register)(app, state, index);
    }
    admin.entries.iter().map(|e| e.create).collect()
}

fn page_doc(summary: &str) -> Operation {
    Operation::new()
        .summary(summary)
        .tag("admin")
        .extend(json!({ crate::docs::PAGE_MARK: true }))
}

fn register<M>(app: &mut vitesse::App, state: &'static AdminState, index: usize)
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    let base = format!("{}/{}", state.path, M::SCHEMA.name);
    let item = format!("{base}/:id");
    let name = crate::docs::type_name::<M>();
    app.get(&base, move |req: Request| list::<M>(req, state, index))
        .doc(page_doc(&format!("Admin: the {name} rows")));
    app.get(&format!("{base}/new"), move |req: Request| {
        new_form::<M>(req, state, index)
    })
    .doc(page_doc(&format!("Admin: new {name}")));
    app.post(&format!("{base}/new"), move |req: Request| {
        create::<M>(req, state, index)
    })
    .doc(page_doc(&format!("Admin: create a {name}")));
    app.get(&item, move |req: Request| edit_form::<M>(req, state, index))
        .doc(page_doc(&format!("Admin: edit a {name}")));
    app.post(&item, move |req: Request| save::<M>(req, state, index))
        .doc(page_doc(&format!("Admin: save a {name}")));
    app.get(&format!("{item}/delete"), move |req: Request| {
        confirm::<M>(req, state, index)
    })
    .doc(page_doc(&format!(
        "Admin: confirm the deletion of a {name}"
    )));
    app.post(&format!("{item}/delete"), move |req: Request| {
        delete::<M>(req, state, index)
    })
    .doc(page_doc(&format!("Admin: delete a {name}")));
}

// ----- Access ------------------------------------------------------------------------

/// `None` when the request may use the panel, the answer otherwise.
fn denied(req: &Request, state: &AdminState) -> Option<Response> {
    let allowed = match &state.allow {
        Some(allow) => allow(req),
        None => {
            let auth = req.try_state::<crate::auth::AuthState>().is_some();
            req.user().is_some_and(|u| u.admin)
                || (!auth && cfg!(debug_assertions) && crate::ops::is_local(req))
        }
    };
    if allowed {
        None
    } else if req.user().is_none() && req.try_state::<crate::auth::AuthState>().is_some() {
        Some(crate::auth::to_login(req))
    } else {
        Some(
            Page::new("Forbidden")
                .status(403)
                .body(
                    active::html::main()
                        .class("admin")
                        .child(h1().text("Forbidden"))
                        .child(p().text("This page is for administrators.")),
                )
                .into_response(),
        )
    }
}

// ----- Values ------------------------------------------------------------------------

/// What a column holds that must never be shown.
fn is_secret(column: &ColumnDef) -> bool {
    let name = column.name.to_lowercase();
    ["password", "secret", "token"]
        .iter()
        .any(|word| name.contains(word))
}

/// Whether the forms change this column.
fn editable<M: Model>(index: usize, column: &ColumnDef) -> bool {
    let stamped = [
        M::CREATED_AT_INDEX,
        M::UPDATED_AT_INDEX,
        M::DELETED_AT_INDEX,
    ]
    .contains(&Some(index));
    !(column.primary_key && column.auto_increment)
        && !stamped
        && !is_secret(column)
        && column.kind != ColumnType::Bytes
}

/// A value as text, for the pages and the forms.
fn show(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Text(s) => s.clone(),
        Value::Bytes(b) => format!("({} bytes)", b.len()),
        other => {
            let text = other.to_string();
            match text.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')) {
                Some(inner) => inner.replace("''", "'"),
                None => text,
            }
        }
    }
}

/// The value of a submitted field (`None`: absent, like an unchecked box).
fn parse(column: &ColumnDef, raw: Option<&str>) -> Result<Value, String> {
    if column.kind == ColumnType::Bool {
        return Ok(Value::Bool(
            raw.is_some_and(|v| !v.is_empty() && v != "false"),
        ));
    }
    let raw = raw.unwrap_or_default();
    let blank = raw.trim().is_empty();
    if blank && column.nullable {
        return Ok(Value::Null);
    }
    match column.kind {
        ColumnType::SmallInt | ColumnType::Int => raw
            .trim()
            .parse()
            .map(Value::I32)
            .map_err(|_| "Enter a whole number".to_owned()),
        ColumnType::BigInt => raw
            .trim()
            .parse()
            .map(Value::I64)
            .map_err(|_| "Enter a whole number".to_owned()),
        ColumnType::Real | ColumnType::Double => raw
            .trim()
            .parse()
            .map(Value::F64)
            .map_err(|_| "Enter a number".to_owned()),
        ColumnType::Text => Ok(Value::Text(raw.to_owned())),
        _ if blank => Err("Required".to_owned()),
        _ => Ok(Value::Text(raw.trim().to_owned())),
    }
}

/// A value for a column the create form does not fill.
fn placeholder(column: &ColumnDef) -> Value {
    if column.nullable {
        return Value::Null;
    }
    match column.kind {
        ColumnType::Bool => Value::Bool(false),
        ColumnType::SmallInt | ColumnType::Int => Value::I32(0),
        ColumnType::BigInt => Value::I64(0),
        ColumnType::Real | ColumnType::Double => Value::F64(0.0),
        ColumnType::Bytes => Value::Bytes(Vec::new()),
        ColumnType::Timestamp | ColumnType::TimestampTz => {
            Value::Text("1970-01-01 00:00:00".into())
        }
        ColumnType::Date => Value::Text("1970-01-01".into()),
        ColumnType::Time => Value::Text("00:00:00".into()),
        ColumnType::Json => Value::Text("null".into()),
        _ => Value::Text(String::new()),
    }
}

type FormData = HashMap<String, String>;
type Errors = Vec<(&'static str, String)>;

/// The values of the editable columns, or the errors by field.
fn read_form<M: Model>(data: &FormData) -> Result<Vec<(usize, Value)>, Errors> {
    let mut values = Vec::new();
    let mut errors = Vec::new();
    for (index, column) in M::SCHEMA.columns.iter().enumerate() {
        if !editable::<M>(index, column) {
            continue;
        }
        match parse(column, data.get(column.name).map(String::as_str)) {
            Ok(value) => values.push((index, value)),
            Err(message) => errors.push((column.name, message)),
        }
    }
    if errors.is_empty() {
        Ok(values)
    } else {
        Err(errors)
    }
}

fn key_of<M: Model>(req: &Request) -> crate::Result<M::Key>
where
    M::Key: FromStr,
{
    let raw = req.param("id").unwrap_or_default();
    raw.parse()
        .map_err(|_| crate::Error::not_found(format!("no row `{raw}`")))
}

fn column<M, T: memoire_orm::SqlType>(schema: &'static TableSchema, index: usize) -> Column<M, T> {
    Column::new(schema.name, schema.columns[index].name, index)
}

// ----- Handlers ----------------------------------------------------------------------

async fn dashboard(req: Request, state: &'static AdminState) -> crate::Result<Response> {
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let mut counts = Vec::new();
    for (table, label, count) in &state.models {
        counts.push((*table, label.clone(), count(req.db()).await?));
    }
    let cards = counts.into_iter().map(|(table, label, count)| {
        let base = format!("{}/{table}", state.path);
        div()
            .class("card")
            .child(h2().child(a().href(base.clone()).text(label)))
            .child(
                p().class("muted")
                    .text(format!("{count} row(s) · table {table}")),
            )
            .child(
                p().class("links")
                    .child(a().href(base.clone()).text("View"))
                    .child(a().href(format!("{base}/new")).text("Add")),
            )
    });
    let content = div().class("grid").children(cards);
    Ok(layout(state, &state.title, None, vec![content]).into_response())
}

struct ListQuery {
    page: u64,
    q: String,
    sort: Option<usize>,
    desc: bool,
    filters: Vec<(usize, bool)>,
    notice: Option<String>,
}

impl ListQuery {
    fn read(req: &Request, schema: &TableSchema) -> Self {
        let sort = req
            .query("sort")
            .and_then(|name| schema.column_index(&name))
            .filter(|&i| !is_secret(&schema.columns[i]));
        let filters = schema
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == ColumnType::Bool)
            .filter_map(|(i, c)| match req.query(c.name).as_deref() {
                Some("true" | "1") => Some((i, true)),
                Some("false" | "0") => Some((i, false)),
                _ => None,
            })
            .collect();
        ListQuery {
            page: req
                .query("page")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1)
                .max(1),
            q: req.query("q").unwrap_or_default().trim().to_owned(),
            sort,
            desc: req.query("dir").as_deref() == Some("desc"),
            filters,
            notice: req.query("notice").map(|n| n.into_owned()),
        }
    }

    /// The query string of this list with some parameters changed.
    fn link(&self, schema: &TableSchema, changes: &[(&str, Option<String>)]) -> String {
        let mut params: Vec<(String, String)> = Vec::new();
        if !self.q.is_empty() {
            params.push(("q".into(), self.q.clone()));
        }
        if let Some(i) = self.sort {
            params.push(("sort".into(), schema.columns[i].name.into()));
            if self.desc {
                params.push(("dir".into(), "desc".into()));
            }
        }
        for (i, value) in &self.filters {
            params.push((schema.columns[*i].name.into(), value.to_string()));
        }
        if self.page > 1 {
            params.push(("page".into(), self.page.to_string()));
        }
        for (name, value) in changes {
            params.retain(|(n, _)| n != name);
            if let Some(value) = value {
                params.push(((*name).to_owned(), value.clone()));
            }
        }
        let query: Vec<String> = params
            .iter()
            .map(|(n, v)| format!("{}={}", encode_component(n), encode_component(v)))
            .collect();
        format!("?{}", query.join("&"))
    }
}

async fn list<M>(req: Request, state: &'static AdminState, index: usize) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let schema = M::SCHEMA;
    let params = ListQuery::read(&req, schema);
    let texts: Vec<usize> = schema
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == ColumnType::Text && !is_secret(c))
        .map(|(i, _)| i)
        .collect();
    let mut query = M::query();
    if !params.q.is_empty() && !texts.is_empty() {
        query = query.filter(Expr::any(
            texts
                .iter()
                .map(|&i| column::<M, String>(schema, i).icontains(&params.q)),
        ));
    }
    for &(i, value) in &params.filters {
        query = query.filter(column::<M, bool>(schema, i).eq(value));
    }
    let order = params.sort.unwrap_or(schema.primary_key);
    let order = column::<M, i64>(schema, order);
    query = query.order_by(if params.desc {
        order.desc()
    } else {
        order.asc()
    });
    let page = query
        .paginate(req.db(), params.page, state.per_page)
        .await?;
    let rows: Vec<(String, Vec<String>)> = page
        .items
        .iter()
        .map(|row| {
            let values = (0..schema.columns.len())
                .map(|i| {
                    if is_secret(&schema.columns[i]) {
                        "••••".to_owned()
                    } else {
                        shorten(&show(&row.value_at(i)))
                    }
                })
                .collect();
            (show(&row.key()), values)
        })
        .collect();
    let total_pages = page.total_pages().max(1);
    let base = format!("{}/{}", state.path, schema.name);
    let label = &state.models[index].1;

    // The page, built after the last `.await`.
    let mut content: Vec<Element> = Vec::new();
    if let Some(notice) = &params.notice {
        let text = match notice.as_str() {
            "created" => "The row was created.",
            "saved" => "The changes were saved.",
            "deleted" => "The row was deleted.",
            _ => "",
        };
        if !text.is_empty() {
            content.push(p().class("notice").attr("role", "status").text(text));
        }
    }
    let mut toolbar = div().class("toolbar");
    if !texts.is_empty() {
        let mut search = form()
            .attr("method", "get")
            .attr("action", base.clone())
            .class("row")
            .child(
                input()
                    .attr("type", "search")
                    .attr("name", "q")
                    .attr("value", params.q.clone())
                    .attr("placeholder", "Search"),
            );
        if let Some(i) = params.sort {
            search = search.child(hidden("sort", schema.columns[i].name));
            if params.desc {
                search = search.child(hidden("dir", "desc"));
            }
        }
        toolbar = toolbar.child(search.child(button().attr("type", "submit").text("Search")));
    }
    for (i, c) in schema.columns.iter().enumerate() {
        if c.kind != ColumnType::Bool {
            continue;
        }
        let current = params
            .filters
            .iter()
            .find(|(f, _)| *f == i)
            .map(|(_, v)| *v);
        let choice = |text: &str, value: Option<bool>| {
            let link = params.link(
                schema,
                &[(c.name, value.map(|v| v.to_string())), ("page", None)],
            );
            let element = a().href(link).text(text.to_owned());
            if current == value {
                strong().child(element)
            } else {
                span().child(element)
            }
        };
        toolbar = toolbar.child(
            span()
                .class("filter")
                .child(span().class("muted").text(format!("{}:", c.field)))
                .child(choice("all", None))
                .child(choice("yes", Some(true)))
                .child(choice("no", Some(false))),
        );
    }
    toolbar = toolbar.child(a().class("button").href(format!("{base}/new")).text("Add"));
    content.push(toolbar);

    let headers = schema.columns.iter().enumerate().map(|(i, c)| {
        if is_secret(c) {
            return th().text(c.field);
        }
        let (dir, arrow) = match (
            params.sort == Some(i) || (params.sort.is_none() && i == schema.primary_key),
            params.desc,
        ) {
            (true, false) => (Some("desc".to_owned()), " ↑"),
            (true, true) => (None, " ↓"),
            (false, _) => (None, ""),
        };
        let link = params.link(
            schema,
            &[
                ("sort", Some(c.name.to_owned())),
                ("dir", dir),
                ("page", None),
            ],
        );
        th().child(a().href(link).text(format!("{}{arrow}", c.field)))
    });
    let body = rows.into_iter().map(|(key, values)| {
        let href = format!("{base}/{}", encode_component(&key));
        tr().children(values.into_iter().enumerate().map(|(i, value)| {
            if i == schema.primary_key {
                td().child(a().href(href.clone()).text(value))
            } else {
                td().text(value)
            }
        }))
    });
    content.push(
        div().class("scroll").child(
            table()
                .child(thead().child(tr().children(headers)))
                .child(tbody().children(body)),
        ),
    );
    let mut pager = p().class("pager").child(span().class("muted").text(format!(
        "{} row(s) · page {} of {total_pages}",
        page.total, page.page
    )));
    if page.page > 1 {
        let link = params.link(schema, &[("page", Some((page.page - 1).to_string()))]);
        pager = pager.child(a().href(link).text("← Previous"));
    }
    if page.page < total_pages {
        let link = params.link(schema, &[("page", Some((page.page + 1).to_string()))]);
        pager = pager.child(a().href(link).text("Next →"));
    }
    content.push(pager);
    Ok(layout(state, label, Some(index), content).into_response())
}

fn shorten(text: &str) -> String {
    if text.chars().count() > 80 {
        format!("{}…", text.chars().take(79).collect::<String>())
    } else {
        text.to_owned()
    }
}

fn hidden(name: &str, value: &str) -> Element {
    input()
        .attr("type", "hidden")
        .attr("name", name.to_owned())
        .attr("value", value.to_owned())
}

/// The form of a row: `values` are the current (or submitted) texts, by column.
fn row_form<M: Model>(action: String, values: &[String], errors: &Errors, create: bool) -> Element {
    let schema = M::SCHEMA;
    let mut fields = form().attr("method", "post").attr("action", action);
    for (index, column) in schema.columns.iter().enumerate() {
        let value = values.get(index).cloned().unwrap_or_default();
        let mut field = label()
            .attr("for", column.name)
            .child(strong().text(column.field));
        if !editable::<M>(index, column) {
            let shown = if is_secret(column) {
                "••••".to_owned()
            } else if create {
                "(automatic)".to_owned()
            } else {
                value
            };
            field = field.child(p().class("muted").text(shown));
        } else {
            let control = match column.kind {
                ColumnType::Bool => {
                    let mut checkbox = input()
                        .attr("type", "checkbox")
                        .attr("name", column.name)
                        .attr("id", column.name)
                        .attr("value", "true");
                    if value == "true" {
                        checkbox = checkbox.attr("checked", "");
                    }
                    checkbox
                }
                ColumnType::Json => textarea()
                    .attr("name", column.name)
                    .attr("id", column.name)
                    .attr("rows", "6")
                    .text(value),
                kind => {
                    let (kind_attr, step) = match kind {
                        ColumnType::SmallInt | ColumnType::Int | ColumnType::BigInt => {
                            ("number", Some("1"))
                        }
                        ColumnType::Real | ColumnType::Double => ("number", Some("any")),
                        ColumnType::Date => ("date", None),
                        _ => ("text", None),
                    };
                    let mut control = input()
                        .attr("type", kind_attr)
                        .attr("name", column.name)
                        .attr("id", column.name)
                        .attr("value", value);
                    if let Some(step) = step {
                        control = control.attr("step", step);
                    }
                    if !column.nullable && kind != ColumnType::Text {
                        control = control.attr("required", "");
                    }
                    control
                }
            };
            field = field.child(control);
            if column.nullable {
                field = field.child(span().class("muted").text(" optional"));
            }
        }
        if let Some((_, message)) = errors.iter().find(|(name, _)| *name == column.name) {
            field = field.child(
                p().class("error")
                    .attr("role", "alert")
                    .text(message.clone()),
            );
        }
        fields = fields.child(field);
    }
    fields.child(
        button()
            .attr("type", "submit")
            .text(if create { "Create" } else { "Save" }),
    )
}

fn texts_of<M: Model>(row: &M) -> Vec<String> {
    (0..M::SCHEMA.columns.len())
        .map(|i| show(&row.value_at(i)))
        .collect()
}

fn submitted<M: Model>(data: &FormData) -> Vec<String> {
    M::SCHEMA
        .columns
        .iter()
        .map(|c| match c.kind {
            ColumnType::Bool => data.contains_key(c.name).to_string(),
            _ => data.get(c.name).cloned().unwrap_or_default(),
        })
        .collect()
}

fn form_page<M: Model>(
    state: &AdminState,
    index: usize,
    title: String,
    form: Element,
    error: Option<String>,
    extra: Option<Element>,
    status: u16,
) -> Response {
    let mut content: Vec<Element> = Vec::new();
    if let Some(message) = error {
        content.push(p().class("error").attr("role", "alert").text(message));
    }
    content.push(div().class("card").child(form));
    content.extend(extra);
    let _ = M::SCHEMA;
    layout(state, &title, Some(index), content)
        .status(status)
        .into_response()
}

async fn new_form<M>(
    req: Request,
    state: &'static AdminState,
    index: usize,
) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let base = format!("{}/{}", state.path, M::SCHEMA.name);
    let values: Vec<String> = M::SCHEMA
        .columns
        .iter()
        .map(|c| match c.default {
            Some(default) => default.trim_matches('\'').to_owned(),
            None => String::new(),
        })
        .collect();
    let form = row_form::<M>(format!("{base}/new"), &values, &Vec::new(), true);
    let title = format!("New {}", state.models[index].1);
    Ok(form_page::<M>(state, index, title, form, None, None, 200))
}

/// The message of a database error, for the form; `None` for server errors.
fn database_message(error: &memoire_orm::Error) -> Option<String> {
    use memoire_orm::Error as Db;
    match error {
        Db::UniqueViolation { column, .. } => Some(format!("This {column} is already used.")),
        Db::Constraint(message) | Db::Conflict(message) | Db::Conversion(message) => {
            Some(message.clone())
        }
        Db::UnexpectedNull { .. } => Some(error.to_string()),
        _ => None,
    }
}

async fn create<M>(
    req: Request,
    state: &'static AdminState,
    index: usize,
) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let schema = M::SCHEMA;
    let base = format!("{}/{}", state.path, schema.name);
    let data: FormData = req.form().await?;
    let title = format!("New {}", state.models[index].1);
    let fail = |errors: &Errors, error: Option<String>| {
        let form = row_form::<M>(format!("{base}/new"), &submitted::<M>(&data), errors, true);
        form_page::<M>(state, index, title.clone(), form, error, None, 422)
    };
    let values = match read_form::<M>(&data) {
        Ok(values) => values,
        Err(errors) => return Ok(fail(&errors, None)),
    };
    let mut row: Vec<Value> = schema.columns.iter().map(placeholder).collect();
    for (i, value) in values {
        row[i] = value;
    }
    let mut model = match M::from_row(&mut ValueRow(row), 0) {
        Ok(model) => model,
        Err(error) => return Ok(fail(&Vec::new(), Some(error.to_string()))),
    };
    if let Err(error) = model.insert(req.db()).await {
        return match database_message(&error) {
            Some(message) => Ok(fail(&Vec::new(), Some(message))),
            None => Err(error.into()),
        };
    }
    Ok(see_other(&format!("{base}?notice=created")))
}

fn see_other(location: &str) -> Response {
    Response::new()
        .status(StatusCode::SEE_OTHER)
        .header("location", location)
}

async fn edit_form<M>(
    req: Request,
    state: &'static AdminState,
    index: usize,
) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let key = key_of::<M>(&req)?;
    let row = M::get(req.db(), key).await?;
    let base = format!("{}/{}", state.path, M::SCHEMA.name);
    let id = show(&row.key());
    let item = format!("{base}/{}", encode_component(&id));
    let form = row_form::<M>(item.clone(), &texts_of(&row), &Vec::new(), false);
    let delete = p().child(
        a().class("danger")
            .href(format!("{item}/delete"))
            .text("Delete this row"),
    );
    let title = format!("{} {id}", state.models[index].1);
    Ok(form_page::<M>(
        state,
        index,
        title,
        form,
        None,
        Some(delete),
        200,
    ))
}

async fn save<M>(req: Request, state: &'static AdminState, index: usize) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let key = key_of::<M>(&req)?;
    let db = req.db();
    let mut row = M::get(db, key).await?;
    let data: FormData = req.form().await?;
    let base = format!("{}/{}", state.path, M::SCHEMA.name);
    let id = show(&row.key());
    let item = format!("{base}/{}", encode_component(&id));
    let title = format!("{} {id}", state.models[index].1);
    let mut texts = texts_of(&row);
    for (i, text) in submitted::<M>(&data).into_iter().enumerate() {
        if editable::<M>(i, &M::SCHEMA.columns[i]) {
            texts[i] = text;
        }
    }
    let fail = |errors: &Errors, error: Option<String>| {
        let form = row_form::<M>(item.clone(), &texts, errors, false);
        form_page::<M>(state, index, title.clone(), form, error, None, 422)
    };
    let values = match read_form::<M>(&data) {
        Ok(values) => values,
        Err(errors) => return Ok(fail(&errors, None)),
    };
    for (i, value) in values {
        if let Err(error) = row.set_value_at(i, value) {
            return Ok(fail(&Vec::new(), Some(error.to_string())));
        }
    }
    if let Err(error) = row.update(db).await {
        return match database_message(&error) {
            Some(message) => Ok(fail(&Vec::new(), Some(message))),
            None => Err(error.into()),
        };
    }
    Ok(see_other(&format!("{base}?notice=saved")))
}

async fn confirm<M>(
    req: Request,
    state: &'static AdminState,
    index: usize,
) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let key = key_of::<M>(&req)?;
    let row = M::get(req.db(), key).await?;
    let id = show(&row.key());
    let item = format!(
        "{}/{}/{}",
        state.path,
        M::SCHEMA.name,
        encode_component(&id)
    );
    let label = &state.models[index].1;
    let content = vec![
        div()
            .class("card")
            .child(p().text(format!("Delete {label} {id}? This cannot be undone.")))
            .child(
                form()
                    .attr("method", "post")
                    .attr("action", format!("{item}/delete"))
                    .class("row")
                    .child(
                        button()
                            .attr("type", "submit")
                            .class("danger")
                            .text("Delete"),
                    )
                    .child(a().class("button ghost").href(item.clone()).text("Cancel")),
            ),
    ];
    Ok(layout(state, &format!("Delete {label} {id}"), Some(index), content).into_response())
}

async fn delete<M>(
    req: Request,
    state: &'static AdminState,
    index: usize,
) -> crate::Result<Response>
where
    M: Model,
    M::Key: FromStr + IntoValue + Send,
{
    if let Some(answer) = denied(&req, state) {
        return Ok(answer);
    }
    let _ = index;
    let key = key_of::<M>(&req)?;
    if !M::delete_by_key(req.db(), key).await? {
        return Err(crate::Error::not_found("record not found"));
    }
    Ok(see_other(&format!(
        "{}/{}?notice=deleted",
        state.path,
        M::SCHEMA.name
    )))
}

// ----- Layout ------------------------------------------------------------------------

const STYLE: &str = ".admin{max-width:80rem;margin:0 auto;padding:0 1rem}\
.admin .toolbar{display:flex;flex-wrap:wrap;gap:1rem;align-items:center;margin:1rem 0}\
.admin .toolbar form{margin:0}.admin .filter{display:flex;gap:.4rem}\
.admin .scroll{overflow-x:auto}.admin table{width:100%}\
.admin .pager,.admin .links{display:flex;gap:1rem;flex-wrap:wrap;align-items:center}\
.admin .error{color:#d33}.admin .notice{border-left:3px solid currentColor;padding-left:.75rem}\
.admin .danger{color:#fff;background:#c33;border-color:#c33}\
.admin form label{display:block;margin-bottom:.75rem}.admin nav .current{font-weight:700}";

fn layout(state: &AdminState, title: &str, current: Option<usize>, content: Vec<Element>) -> Page {
    let links = state
        .models
        .iter()
        .enumerate()
        .map(|(i, (table, label, _))| {
            let link = a()
                .href(format!("{}/{table}", state.path))
                .text(label.clone());
            if current == Some(i) {
                li().child(link.class("current"))
            } else {
                li().child(link)
            }
        });
    Page::new(format!("{title} · {}", state.title))
        .style(STYLE)
        .seo(active::Seo::new().noindex(true))
        .body(
            active::html::header().child(
                nav()
                    .child(
                        strong().child(
                            a().href(state.path.clone())
                                .text(format!("⚙️ {}", state.title)),
                        ),
                    )
                    .child(ul().class("row").children(links)),
            ),
        )
        .body(
            active::html::main()
                .class("admin")
                .child(h1().text(title.to_owned()))
                .children(content),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(name: &'static str, kind: ColumnType, nullable: bool) -> ColumnDef {
        ColumnDef {
            name,
            field: name,
            kind,
            nullable,
            primary_key: false,
            auto_increment: false,
            unique: false,
            indexed: false,
            fulltext: false,
            sql_type: None,
            default: None,
            references: None,
        }
    }

    #[test]
    fn form_values_follow_the_column_types() {
        let done = column("done", ColumnType::Bool, false);
        assert_eq!(parse(&done, Some("true")), Ok(Value::Bool(true)));
        assert_eq!(parse(&done, None), Ok(Value::Bool(false)));
        let count = column("count", ColumnType::BigInt, false);
        assert_eq!(parse(&count, Some(" 42 ")), Ok(Value::I64(42)));
        assert!(parse(&count, Some("x")).is_err());
        assert!(parse(&count, Some("")).is_err());
        let note = column("note", ColumnType::Text, true);
        assert_eq!(parse(&note, Some("")), Ok(Value::Null));
        assert_eq!(parse(&note, Some("hi")), Ok(Value::Text("hi".into())));
        let price = column("price", ColumnType::Double, false);
        assert_eq!(parse(&price, Some("1.5")), Ok(Value::F64(1.5)));
    }

    #[test]
    fn values_are_shown_as_text() {
        assert_eq!(show(&Value::Null), "");
        assert_eq!(show(&Value::Text("it's".into())), "it's");
        assert_eq!(show(&Value::I64(7)), "7");
        assert_eq!(show(&Value::Bool(true)), "true");
    }

    #[test]
    fn secrets_are_recognized() {
        assert!(is_secret(&column("password_hash", ColumnType::Text, false)));
        assert!(is_secret(&column("api_token", ColumnType::Text, false)));
        assert!(!is_secret(&column("title", ColumnType::Text, false)));
    }
}
