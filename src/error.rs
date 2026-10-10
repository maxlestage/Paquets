//! One error type for the whole application.

use std::any::Any;
use std::borrow::Cow;
use std::fmt;

use vitesse::{IntoResponse, Response, StatusCode};

/// `Result<T, paquets::Error>`: the return type of handlers that can fail.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error that becomes an HTTP response.
///
/// `?` converts every error into it:
///
/// - database errors get the right status: a missing row is a `404`, a duplicate
///   unique value a `409`, an invalid value a `400`;
/// - errors from the request (`req.json()`, `req.param_as()`…) keep their status;
/// - any other error becomes a `500` whose details are logged, never sent to the
///   client.
///
/// ```
/// use paquets::prelude::*;
///
/// #[model]
/// struct Todo {
///     id: i64,
///     title: String,
/// }
///
/// async fn show(req: Request) -> Result<Json<Todo>> {
///     let id: i64 = req.param_as("id")?;           // 400 if `id` is not a number
///     let todo = Todo::get(req.db(), id).await?;  // 404 if there is no such row
///     Ok(Json(todo))
/// }
/// ```
pub struct Error(vitesse::Error);

impl Error {
    /// An error with a status and a message sent to the client.
    pub fn new(status: impl vitesse::IntoStatus, message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::new(status, message))
    }

    /// `400 Bad Request`.
    pub fn bad_request(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::bad_request(message))
    }

    /// `401 Unauthorized`.
    pub fn unauthorized(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::unauthorized(message))
    }

    /// `403 Forbidden`.
    pub fn forbidden(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::forbidden(message))
    }

    /// `404 Not Found`.
    pub fn not_found(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::not_found(message))
    }

    /// `409 Conflict`.
    pub fn conflict(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::conflict(message))
    }

    /// `422 Unprocessable Content`.
    pub fn unprocessable(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::unprocessable(message))
    }

    /// `500 Internal Server Error`: the message is logged, not sent to the client.
    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Error(vitesse::Error::internal(message))
    }

    /// The HTTP status.
    pub fn status(&self) -> StatusCode {
        self.0.status()
    }

    /// The message sent to the client.
    pub fn message(&self) -> &str {
        self.0.message()
    }

    /// The underlying `vitesse` error.
    pub fn into_inner(self) -> vitesse::Error {
        self.0
    }
}

impl<E: Into<vitesse::Error> + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        if let Some(db) = (&err as &dyn Any).downcast_ref::<memoire_orm::Error>() {
            if let Some(status) = database_status(db) {
                return Error(vitesse::Error::new(status, db.to_string()));
            }
        }
        Error(err.into())
    }
}

/// The status of database errors caused by the request; `None` for server errors.
fn database_status(err: &memoire_orm::Error) -> Option<StatusCode> {
    use memoire_orm::Error as Db;
    match err {
        Db::NotFound => Some(StatusCode::NOT_FOUND),
        Db::UniqueViolation { .. } | Db::Constraint(_) | Db::Conflict(_) => {
            Some(StatusCode::CONFLICT)
        }
        Db::UnexpectedNull { .. } | Db::Conversion(_) => Some(StatusCode::BAD_REQUEST),
        _ => None,
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_errors_get_their_status() {
        let not_found: Error = memoire_orm::Error::NotFound.into();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);
        let duplicate: Error = memoire_orm::Error::UniqueViolation {
            table: "users".into(),
            column: "email".into(),
        }
        .into();
        assert_eq!(duplicate.status(), StatusCode::CONFLICT);
        let broken: Error = memoire_orm::Error::NoSuchTable("users".into()).into();
        assert_eq!(broken.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn request_errors_keep_their_status() {
        let err: Error = vitesse::Error::bad_request("nope").into();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(err.message(), "nope");
        let io: Error = std::io::Error::other("disk").into();
        assert_eq!(io.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
