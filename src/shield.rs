//! `securite` in front of `vitesse`: the shield is a tower layer working on
//! `http::Request` heads, vitesse has its own request type. The bridge hands the
//! shield a copy of the head, then gives the rest of the chain the original request
//! with whatever the guards changed (request id header, CSRF token, CSP nonce,
//! client IP, authenticated principal…).

use std::convert::Infallible;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::task::{Context, Poll};

use securite::body::SecureBody;
use securite::{ClientIpSource, PeerAddr, Shield, ShieldBuilder};
use tower_service::Service;
use vitesse::{Body, BoxFuture, IntoResponse, Middleware, Next, Request, Response, header};

/// The protection used by [`App::new`](crate::App::new): securite's recommended
/// preset (request id, protocol hygiene, method filter, 2 MiB body limit, rate
/// limit of 20 requests per second per IP with bursts of 100, bot blocker, WAF,
/// CSRF via Fetch Metadata, security headers, hardened cookies), reading the client
/// IP from `X-Forwarded-For` on Heroku.
pub fn default_shield() -> Shield {
    let ip = if std::env::var_os("DYNO").is_some() {
        ClientIpSource::heroku()
    } else {
        ClientIpSource::Peer
    };
    ShieldBuilder::recommended().client_ip(ip).build()
}

/// The first global middleware of every app. The shield is chosen when the app
/// starts (`App::security` may be called after routes and middlewares).
pub(crate) struct ShieldMiddleware(pub(crate) Arc<OnceLock<Option<Shield>>>);

impl Middleware for ShieldMiddleware {
    fn handle(&'static self, req: Request, next: Next) -> BoxFuture<Response> {
        match self.0.get() {
            Some(Some(shield)) => Box::pin(protect(shield, req, next)),
            _ => next.run(req),
        }
    }
}

async fn protect(shield: &'static Shield, req: Request, next: Next) -> Response {
    let mut head = http::Request::new(());
    *head.method_mut() = req.method().clone();
    *head.uri_mut() = req.uri().clone();
    *head.version_mut() = req.version();
    *head.headers_mut() = req.headers().clone();
    if let Some(addr) = req.remote_addr() {
        head.extensions_mut().insert(PeerAddr(addr));
    }

    let error = Arc::new(Mutex::new(None));
    let mut service = tower_layer::Layer::layer(
        shield,
        Rest {
            req: Some(req),
            next: Some(next),
            error: error.clone(),
        },
    );
    let future = service.call(head);
    drop(service);
    let (parts, body) = match future.await {
        Ok(res) => res.into_parts(),
        Err(never) => match never {},
    };
    let body = match body {
        SecureBody::Inner { body } => body,
        SecureBody::Full { data } => data.map(Body::from).unwrap_or_default(),
    };
    let res = Response::from_http(http::Response::from_parts(parts, body));

    // An error response keeps its error, for `app.on_error`, with the headers the
    // shield added.
    let error = error.lock().unwrap_or_else(PoisonError::into_inner).take();
    match error {
        None => res,
        Some(error) => {
            let mut out = error.into_response();
            out.set_status(res.status_code());
            for (name, value) in res.headers() {
                if name != header::CONTENT_TYPE && name != header::CONTENT_LENGTH {
                    out.headers_mut().insert(name.clone(), value.clone());
                }
            }
            out
        }
    }
}

/// The rest of the vitesse chain, as the service wrapped by the shield.
struct Rest {
    req: Option<Request>,
    next: Option<Next>,
    error: Arc<Mutex<Option<vitesse::Error>>>,
}

impl Service<http::Request<()>> for Rest {
    type Response = http::Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<Result<http::Response<Body>, Infallible>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, head: http::Request<()>) -> Self::Future {
        let (Some(mut req), Some(next)) = (self.req.take(), self.next.take()) else {
            return Box::pin(async {
                Ok(vitesse::Error::internal("shield called the app twice")
                    .into_response()
                    .into_http())
            });
        };
        let (parts, ()) = head.into_parts();
        *req.headers_mut() = parts.headers;
        req.extensions_mut().extend(parts.extensions);
        let error = self.error.clone();
        Box::pin(async move {
            let mut res = next.run(req).await;
            if let Some(err) = res.take_error() {
                *error.lock().unwrap_or_else(PoisonError::into_inner) = Some(err);
            }
            Ok(res.into_http())
        })
    }
}
