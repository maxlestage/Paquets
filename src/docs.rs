//! The documentation of the routes: the OpenAPI document and Swagger UI of
//! [`App::api_docs`](crate::App::api_docs), and the route list of `paquets routes`.

use memoire_orm::{ColumnType, TableSchema};
use securite::headers::ContentSecurityPolicy;
use serde_json::{Map, Value, json};
use vitesse::openapi::{OpenApi, Operation};
use vitesse::{Bytes, Response, header};

/// The extension that marks the HTML pages of Paquets (auth, admin…): they are in
/// the route list, not in the API document.
pub(crate) const PAGE_MARK: &str = "x-paquets-page";

/// The Swagger UI release the documentation page loads.
const SWAGGER: &str = "https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.17.14";

/// One route of the application, as `paquets routes` lists it
/// (see [`App::routes`](crate::App::routes)).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RouteInfo {
    /// `GET`, `POST`…
    pub method: String,
    /// The pattern, with `:name` parameters (`/api/todos/:id`).
    pub path: String,
    /// The summary of its documentation, if any.
    pub summary: String,
    /// Whether it is an HTML page of Paquets (auth, admin…) rather than an API route.
    pub page: bool,
}

/// Every documented-able route (static files and `app.all` routes excepted), by path.
pub(crate) fn routes_of(app: &vitesse::App) -> Vec<RouteInfo> {
    let document = app.openapi_document(&OpenApi::new("routes", "0"));
    let mut routes = Vec::new();
    if let Some(paths) = document["paths"].as_object() {
        for (path, operations) in paths {
            let Some(operations) = operations.as_object() else {
                continue;
            };
            for (method, operation) in operations {
                if method == "parameters" {
                    continue;
                }
                routes.push(RouteInfo {
                    method: method.to_uppercase(),
                    path: colon_params(path),
                    summary: operation["summary"].as_str().unwrap_or_default().to_owned(),
                    page: operation[PAGE_MARK] == Value::Bool(true),
                });
            }
        }
    }
    routes
}

/// `/todos/{id}` → `/todos/:id`.
fn colon_params(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut chars = path.chars();
    while let Some(c) = chars.next() {
        if c == '{' {
            out.push(':');
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The route list printed by `paquets routes`.
pub(crate) fn route_table(routes: &[RouteInfo]) -> String {
    let width = routes
        .iter()
        .map(|r| r.path.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let mut out = format!("{:<7} {:<width$}  {}\n", "METHOD", "PATH", "DESCRIPTION");
    for route in routes {
        let summary = if route.page && !route.summary.is_empty() {
            format!("{} (page)", route.summary)
        } else {
            route.summary.clone()
        };
        out.push_str(format!("{:<7} {:<width$}  {summary}", route.method, route.path).trim_end());
        out.push('\n');
    }
    out.push_str(&format!("\n{} routes\n", routes.len()));
    out
}

/// The short name of a type: `my_app::models::Todo` → `Todo`.
pub(crate) fn type_name<T>() -> &'static str {
    let full = std::any::type_name::<T>();
    let base = full.split('<').next().unwrap_or(full);
    base.rsplit("::").next().unwrap_or(base)
}

/// The JSON Schema of a model's rows.
pub(crate) fn json_schema(schema: &TableSchema) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for column in schema.columns {
        let mut property = match column.kind {
            ColumnType::Bool => json!({ "type": "boolean" }),
            ColumnType::SmallInt | ColumnType::Int => {
                json!({ "type": "integer", "format": "int32" })
            }
            ColumnType::BigInt => json!({ "type": "integer", "format": "int64" }),
            ColumnType::Real | ColumnType::Double => json!({ "type": "number" }),
            ColumnType::Text => json!({ "type": "string" }),
            ColumnType::Bytes => json!({ "type": "array", "items": { "type": "integer" } }),
            ColumnType::Uuid => json!({ "type": "string", "format": "uuid" }),
            ColumnType::Timestamp | ColumnType::TimestampTz => {
                json!({ "type": "string", "format": "date-time" })
            }
            ColumnType::Date => json!({ "type": "string", "format": "date" }),
            ColumnType::Time => json!({ "type": "string", "format": "time" }),
            _ => json!({}),
        };
        if column.nullable
            && let Some(kind) = property.get("type").cloned()
        {
            property["type"] = json!([kind, "null"]);
        }
        if column.primary_key && column.auto_increment {
            property["readOnly"] = json!(true);
        }
        if !column.nullable && !(column.primary_key && column.auto_increment) {
            required.push(Value::String(column.field.to_owned()));
        }
        properties.insert(column.field.to_owned(), property);
    }
    json!({ "type": "object", "properties": properties, "required": required })
}

/// The documentation of the six routes of `app.api::<M>()`.
pub(crate) fn api_operations(name: &str) -> [Operation; 6] {
    let schema = json!({ "$ref": format!("#/components/schemas/{name}") });
    let op = |summary: String| Operation::new().summary(summary).tag(name);
    [
        op(format!("List the {name} rows"))
            .query(
                "limit",
                "Rows per page (1 to 1000, 100 by default)",
                json!({ "type": "integer" }),
            )
            .query("offset", "Rows to skip", json!({ "type": "integer" }))
            .response_json(
                200,
                "The rows, in key order (total in X-Total-Count)",
                json!({ "type": "array", "items": schema }),
            ),
        op(format!("Create a {name}"))
            .body(schema.clone())
            .response_json(201, "The new row, with its key", schema.clone())
            .response(409, "A unique value is already used"),
        op(format!("Get a {name}"))
            .response_json(200, "The row", schema.clone())
            .response(404, "No such row"),
        op(format!("Replace a {name}"))
            .body(schema.clone())
            .response_json(200, "The row", schema.clone())
            .response(404, "No such row"),
        op(format!("Change some fields of a {name}"))
            .body(json!({ "type": "object" }))
            .response_json(200, "The row", schema.clone())
            .response(404, "No such row"),
        op(format!("Delete a {name}"))
            .response(204, "Deleted")
            .response(404, "No such row"),
    ]
}

/// The settings of [`App::api_docs`](crate::App::api_docs).
pub(crate) struct ApiDocs {
    pub title: String,
    pub version: String,
    pub json: String,
    pub ui: String,
}

/// Serves the OpenAPI document of the API routes (Paquets' pages left out) and the
/// Swagger UI page, with a Content-Security-Policy that lets it load.
pub(crate) fn install(app: &mut vitesse::App, docs: &ApiDocs, schemas: &[(String, Value)]) {
    let mut spec = OpenApi::new(&docs.title, &docs.version);
    for (name, schema) in schemas {
        spec = spec.schema(name, schema.clone());
    }
    let mut document = app.openapi_document(&spec);
    if let Some(paths) = document["paths"].as_object_mut() {
        for operations in paths.values_mut() {
            if let Some(operations) = operations.as_object_mut() {
                operations.retain(|_, op| op[PAGE_MARK] != Value::Bool(true));
            }
        }
        paths.retain(|_, ops| {
            ops.as_object()
                .is_some_and(|o| o.keys().any(|k| k != "parameters"))
        });
    }
    let body = Bytes::from(serde_json::to_vec_pretty(&document).unwrap_or_default());
    let hidden = || Operation::new().hidden();
    app.get(&docs.json, move |_| {
        let body = body.clone();
        async move { Response::new().content_type("application/json").send(body) }
    })
    .doc(hidden());
    let (page, policy) = swagger_page(&docs.title, &docs.json);
    let (page, policy) = (Bytes::from(page), policy);
    app.get(&docs.ui, move |_| {
        let (page, policy) = (page.clone(), policy.clone());
        async move {
            Response::new()
                .header(header::CONTENT_SECURITY_POLICY, policy)
                .html(page)
        }
    })
    .doc(hidden());
}

/// The Swagger UI page and its policy.
fn swagger_page(title: &str, spec_url: &str) -> (String, String) {
    let title = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let url = serde_json::to_string(spec_url)
        .unwrap_or_default()
        .replace('<', "\\u003c");
    let script = format!(
        "window.onload=()=>{{window.ui=SwaggerUIBundle({{url:{url},dom_id:\"#swagger-ui\"}});}};"
    );
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title>\
<link rel=\"stylesheet\" href=\"{SWAGGER}/swagger-ui.css\"></head><body><div id=\"swagger-ui\"></div>\
<script src=\"{SWAGGER}/swagger-ui-bundle.js\" crossorigin></script><script>{script}</script></body></html>"
    );
    let hash =
        ContentSecurityPolicy::hashes_for_inline_scripts(&format!("<script>{script}</script>"))
            .pop()
            .unwrap_or_default();
    let policy = format!(
        "default-src 'self'; script-src https://cdn.jsdelivr.net {hash}; \
         style-src https://cdn.jsdelivr.net 'unsafe-inline'; img-src 'self' data: https:; \
         font-src https://cdn.jsdelivr.net data:; connect-src 'self'; object-src 'none'; \
         base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
    );
    (html, policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_are_written_with_a_colon() {
        assert_eq!(
            colon_params("/todos/{id}/tags/{tag}"),
            "/todos/:id/tags/:tag"
        );
        assert_eq!(colon_params("/"), "/");
    }

    #[test]
    fn type_names_are_short() {
        assert_eq!(type_name::<String>(), "String");
        assert_eq!(type_name::<Vec<u8>>(), "Vec");
    }

    #[test]
    fn the_swagger_page_is_allowed_by_its_policy() {
        let (html, policy) = swagger_page("<API>", "/openapi.json");
        assert!(html.contains("&lt;API&gt;"));
        let hash = ContentSecurityPolicy::hashes_for_inline_scripts(&html);
        assert_eq!(hash.len(), 1);
        assert!(policy.contains(&hash[0]));
    }

    #[test]
    fn route_tables_are_aligned() {
        let table = route_table(&[
            RouteInfo {
                method: "GET".into(),
                path: "/".into(),
                summary: String::new(),
                page: false,
            },
            RouteInfo {
                method: "POST".into(),
                path: "/login".into(),
                summary: "Log in".into(),
                page: true,
            },
        ]);
        assert!(table.contains("POST    /login  Log in (page)"), "{table}");
        assert!(table.ends_with("2 routes\n"));
    }
}
