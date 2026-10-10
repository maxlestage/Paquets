//! A REST API for a model, in one line: `app.api::<Todo>("/api/todos")`.

use std::str::FromStr;

use memoire_orm::{Column, IntoValue, Model};
use serde::Serialize;
use serde::de::DeserializeOwned;
use vitesse::{Json, Request, Response, StatusCode, res};

use crate::{Error, RequestExt, Result};

/// What [`App::api`](crate::App::api) needs from a model: `#[model]` structs whose
/// primary key can be read from a URL (integers, strings, UUIDs…).
pub trait Resource: Model<Key: FromStr + IntoValue + Send> + Serialize + DeserializeOwned {}

impl<M> Resource for M
where
    M: Model + Serialize + DeserializeOwned,
    M::Key: FromStr + IntoValue + Send,
{
}

pub(crate) fn routes<M: Resource>(app: &mut vitesse::App, path: &str) {
    let base = match path.trim_end_matches('/') {
        "" => "/",
        base => base,
    };
    let item = format!("{}/:id", base.trim_end_matches('/'));
    app.get(base, list::<M>);
    app.post(base, create::<M>);
    app.get(&item, show::<M>);
    app.put(&item, replace::<M>);
    app.patch(&item, update::<M>);
    app.delete(&item, remove::<M>);
}

/// `GET /todos?limit=100&offset=0`: the rows in primary key order, with the total in
/// `X-Total-Count`.
async fn list<M: Resource>(req: Request) -> Result<Response> {
    let limit = number(&req, "limit")?.unwrap_or(100).clamp(1, 1000);
    let offset = number(&req, "offset")?.unwrap_or(0);
    let db = req.db();
    let total = M::count(db).await?;
    let schema = M::SCHEMA;
    // The column type only matters for typed comparisons, not for ordering.
    let key: Column<M, i64> = Column::new(
        schema.name,
        schema.columns[schema.primary_key].name,
        schema.primary_key,
    );
    let rows = M::query()
        .order_by(key.asc())
        .limit(limit)
        .offset(offset)
        .all(db)
        .await?;
    Ok(res::json(rows).header("x-total-count", total.to_string()))
}

/// `GET /todos/:id`
async fn show<M: Resource>(req: Request) -> Result<Json<M>> {
    let row = M::get(req.db(), id::<M>(&req)?).await?;
    Ok(Json(row))
}

/// `POST /todos`: `201 Created` with the row, including its generated key.
async fn create<M: Resource>(req: Request) -> Result<(StatusCode, Json<M>)> {
    let mut row: M = req.json().await?;
    row.insert(req.db()).await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// `PUT /todos/:id`: replaces the whole row.
async fn replace<M: Resource>(req: Request) -> Result<Json<M>> {
    let id = id::<M>(&req)?;
    let mut row: M = req.json().await?;
    row.set_key(id.into_value())?;
    row.update(req.db()).await?;
    Ok(Json(row))
}

/// `PATCH /todos/:id`: changes only the fields present in the JSON body.
async fn update<M: Resource>(req: Request) -> Result<Json<M>> {
    let db = req.db();
    let current = M::get(db, id::<M>(&req)?).await?;
    let serde_json::Value::Object(changes) = req.json::<serde_json::Value>().await? else {
        return Err(Error::bad_request("expected a JSON object"));
    };
    let mut merged = serde_json::to_value(&current)?;
    if let serde_json::Value::Object(fields) = &mut merged {
        fields.extend(changes);
    }
    let mut row: M =
        serde_json::from_value(merged).map_err(|e| Error::bad_request(e.to_string()))?;
    row.set_key(current.key())?;
    row.update(db).await?;
    Ok(Json(row))
}

/// `DELETE /todos/:id`: `204 No Content`, or `404`.
async fn remove<M: Resource>(req: Request) -> Result<StatusCode> {
    if M::delete_by_key(req.db(), id::<M>(&req)?).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Error::not_found("record not found"))
    }
}

fn id<M: Resource>(req: &Request) -> Result<M::Key> {
    let raw = req.param("id").unwrap_or_default();
    raw.parse()
        .map_err(|_| Error::bad_request(format!("invalid id `{raw}`")))
}

fn number(req: &Request, name: &str) -> Result<Option<u64>> {
    match req.query(name) {
        None => Ok(None),
        Some(raw) => raw
            .parse()
            .map(Some)
            .map_err(|_| Error::bad_request(format!("`{name}` must be a positive integer"))),
    }
}
