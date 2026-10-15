//! Authentication in one line, like Laravel Breeze: sign up, log in, log out,
//! forgotten passwords and two-factor codes, with pages ready to use.
//!
//! ```
//! use paquets::prelude::*;
//! use paquets::auth::{self, Auth};
//!
//! let mut app = App::new();
//! app.auth(Auth::new().name("My app"));
//! app.get(
//!     "/dashboard",
//!     (|req: Request| async move {
//!         let user = req.user().expect("checked by login_required");
//!         format!("Hello {}", user.name)
//!     })
//!     .with(auth::login_required),
//! );
//! ```
//!
//! Everything is built on [securite](https://securite-2db4c1994be7.herokuapp.com/):
//!
//! - passwords are hashed with **Argon2id** ([`PasswordHasher`]), off the async
//!   threads, and unknown e-mail addresses take as long as known ones;
//! - failed logins, reset requests and two-factor codes are **throttled** per
//!   account and IP ([`LoginThrottle`]: 5 failures, then 15 minutes);
//! - the session is a **signed cookie** (`HttpOnly`, `SameSite=Lax`, HMAC-SHA256
//!   with [`Signer`]), invalidated when the password changes;
//! - the password-reset link is **signed** and expires after an hour, and works once;
//! - optional **two-factor** codes (TOTP, [`Totp`]): any authenticator app.
//!
//! | Route | Page |
//! |---|---|
//! | `GET/POST /register` | sign up (unless [`Auth::registration`] is off) |
//! | `GET/POST /login` | log in, then the `?next=` page or [`Auth::home`] |
//! | `POST /logout` | log out |
//! | `GET/POST /forgot-password` | asks for a reset link |
//! | `GET/POST /reset-password?token=…` | chooses a new password |
//! | `GET /account` | the account: two-factor setup, log out |
//! | `GET/POST /two-factor` | the code asked after the password, when enabled |
//!
//! The users are the [`User`] model (table `users`), created at startup. In handlers,
//! [`req.user()`](AuthExt::user) is the logged-in user; [`login_required`] and
//! [`admin_required`] protect routes.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use active::Element;
use active::forms::{Rule, email, min_length, required};
use active::html::{a, button, div, form, h1, input, label, li, p, strong, ul};
use memoire_orm::Model;
use securite::crypt::Encrypter;
use securite::signing::Signer;
use serde::Deserialize;
use vitesse::openapi::Operation;
use vitesse::{BoxFuture, Cookie, Next, Request, Response, SameSite, StatusCode, json};

pub use securite::password::PasswordHasher;
pub use securite::throttle::LoginThrottle;
pub use securite::totp::Totp;

use crate::{Db, Page, RequestExt};

/// The name of the session cookie.
pub const SESSION_COOKIE: &str = "paquets_session";
/// The cookie that remembers, for five minutes, who typed the right password
/// and still has to give a two-factor code.
const PENDING_COOKIE: &str = "paquets_two_factor";

/// A user account, in the `users` table.
///
/// The password hash, the two-factor secret and the session version are never
/// serialized (they stay out of JSON responses), and `Debug` redacts the hash and the
/// secret, so that logs never hold them.
///
/// Written without `#[model]` (which derives `Debug`), with the same derives.
#[derive(Clone, serde::Serialize, serde::Deserialize, memoire_orm::Model)]
#[serde(crate = "::paquets::serde")]
#[memoire(crate = "::paquets::orm", table = "users")]
pub struct User {
    /// The key.
    #[serde(default)]
    pub id: i64,
    /// The e-mail address, lowercased, unique: the login.
    #[memoire(unique)]
    pub email: String,
    /// The name shown in the pages.
    pub name: String,
    /// The Argon2id hash of the password (PHC string).
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    /// The TOTP secret, when two-factor codes are enabled, encrypted with
    /// XChaCha20-Poly1305 (securite's `Encrypter`, a key derived from `APP_KEY`, bound
    /// to the user's id). A base32 secret stored in clear by an earlier version is
    /// still read, and encrypted at the next successful two-factor login.
    #[serde(skip_serializing, default)]
    pub two_factor_secret: Option<String>,
    /// May use the admin panel ([`Admin`](crate::admin::Admin)) and the routes of
    /// [`admin_required`].
    pub admin: bool,
    /// Changes when the password changes: older sessions and reset links stop working.
    #[serde(skip_serializing, default)]
    pub session_version: i64,
    /// When the account was created (Unix time, seconds).
    pub created_at: i64,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted = |present: bool| if present { "<redacted>" } else { "<none>" };
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("name", &self.name)
            .field("password_hash", &redacted(!self.password_hash.is_empty()))
            .field(
                "two_factor_secret",
                &redacted(self.two_factor_secret.is_some()),
            )
            .field("admin", &self.admin)
            .field("session_version", &self.session_version)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl User {
    /// Whether the user gives a two-factor code after the password.
    pub fn two_factor_enabled(&self) -> bool {
        self.two_factor_secret.is_some()
    }

    /// The user with this e-mail address (case and spaces ignored).
    pub async fn find_by_email(db: &Db, address: &str) -> crate::Result<Option<User>> {
        let address = normalize_email(address);
        Ok(User::query()
            .filter(User::EMAIL.eq(address))
            .first(db)
            .await?)
    }
}

/// Reads the logged-in user in handlers.
pub trait AuthExt {
    /// The logged-in user, if any (needs [`App::auth`](crate::App::auth)).
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// async fn hello(req: Request) -> String {
    ///     match req.user() {
    ///         Some(user) => format!("Hello {}", user.name),
    ///         None => "Hello stranger".into(),
    ///     }
    /// }
    /// ```
    fn user(&self) -> Option<&User>;
}

impl AuthExt for Request {
    fn user(&self) -> Option<&User> {
        self.get::<User>()
    }
}

/// What a password-reset e-mail needs: the user and the signed link.
#[derive(Debug, Clone)]
pub struct ResetLink {
    /// Who asked for it.
    pub user: User,
    /// The absolute URL of the reset page, valid for one hour and one use.
    pub url: String,
}

type Mailer = Arc<dyn Fn(ResetLink) -> BoxFuture<crate::Result<()>> + Send + Sync>;

/// The configuration of [`App::auth`](crate::App::auth).
#[must_use]
pub struct Auth {
    prefix: String,
    home: String,
    name: String,
    url: Option<String>,
    secret: Option<String>,
    previous_secrets: Vec<String>,
    hasher: PasswordHasher,
    throttle: LoginThrottle,
    registration: bool,
    two_factor: bool,
    admins: Vec<String>,
    session_ttl: Duration,
    mailer: Option<Mailer>,
}

impl Default for Auth {
    fn default() -> Self {
        Auth::new()
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("prefix", &self.prefix)
            .field("home", &self.home)
            .field("name", &self.name)
            .field("registration", &self.registration)
            .field("two_factor", &self.two_factor)
            .finish_non_exhaustive()
    }
}

impl Auth {
    /// Sign up, log in, log out and password resets, with the routes at the root
    /// (`/login`…), back to `/` after logging in, and two-factor codes available.
    ///
    /// The secret that signs the cookies and links comes from the `APP_KEY`
    /// environment variable (32 bytes or more: `openssl rand -base64 48`). Without
    /// it, a random secret is drawn at startup: sessions end when the server stops.
    pub fn new() -> Self {
        Auth {
            prefix: String::new(),
            home: "/".into(),
            name: "Paquets".into(),
            url: None,
            secret: None,
            previous_secrets: Vec::new(),
            hasher: PasswordHasher::new(),
            throttle: LoginThrottle::new(5, Duration::from_secs(15 * 60)),
            registration: true,
            two_factor: true,
            admins: Vec::new(),
            session_ttl: Duration::from_secs(30 * 24 * 3600),
            mailer: None,
        }
    }

    /// Puts the routes under a prefix: `"/auth"` gives `/auth/login`…
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.trim_end_matches('/').to_owned();
        self
    }

    /// Where users go after logging in or signing up, without `?next=` (`/`).
    pub fn home(mut self, path: &str) -> Self {
        self.home = path.to_owned();
        self
    }

    /// The name of the application, in the pages and in authenticator apps.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    /// The public address of the site (`https://example.com`), for the links sent
    /// by e-mail. By default the `APP_URL` environment variable, then the `Host`
    /// of the request (set it in production: a forged `Host` would send the link
    /// elsewhere).
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.trim_end_matches('/').to_owned());
        self
    }

    /// The secret that signs sessions and links (32 bytes or more), instead of
    /// `APP_KEY`.
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    /// A secret used before the current one (key rotation): sessions, links and
    /// two-factor secrets made with it are still accepted. New ones use the current
    /// secret. Call it once per old secret.
    pub fn previous_secret(mut self, secret: impl Into<String>) -> Self {
        self.previous_secrets.push(secret.into());
        self
    }

    /// Another password hasher: a pepper, other Argon2 costs, or
    /// [`PasswordHasher::insecure_fast_for_tests`] in tests.
    pub fn hasher(mut self, hasher: PasswordHasher) -> Self {
        self.hasher = hasher;
        self
    }

    /// Another limit of failed attempts (5 per 15 minutes by default).
    pub fn throttle(mut self, throttle: LoginThrottle) -> Self {
        self.throttle = throttle;
        self
    }

    /// Whether visitors can create an account (`true`). Without it, accounts are
    /// created by the admin panel or by your code.
    pub fn registration(mut self, open: bool) -> Self {
        self.registration = open;
        self
    }

    /// Whether users can enable two-factor codes on their account page (`true`).
    pub fn two_factor(mut self, enabled: bool) -> Self {
        self.two_factor = enabled;
        self
    }

    /// The e-mail addresses of the administrators: they get [`User::admin`] when
    /// they sign up or log in.
    pub fn admins<I, S>(mut self, emails: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.admins = emails
            .into_iter()
            .map(|e| normalize_email(e.as_ref()))
            .collect();
        self
    }

    /// How long a session lasts (30 days).
    pub fn session_ttl(mut self, ttl: Duration) -> Self {
        self.session_ttl = ttl;
        self
    }

    /// Sends the password-reset links (by e-mail, with the crate of your choice).
    /// Without it, development builds print the link in the terminal.
    ///
    /// ```
    /// use paquets::auth::{Auth, ResetLink};
    ///
    /// let auth = Auth::new().mailer(|link: ResetLink| async move {
    ///     println!("to {}: {}", link.user.email, link.url); // send an e-mail here
    ///     Ok(())
    /// });
    /// ```
    pub fn mailer<F, Fut>(mut self, send: F) -> Self
    where
        F: Fn(ResetLink) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = crate::Result<()>> + Send + 'static,
    {
        self.mailer = Some(Arc::new(move |link| Box::pin(send(link))));
        self
    }
}

/// The configuration, in the application's state.
pub(crate) struct AuthState {
    config: Auth,
    session: Signer,
    pending: Signer,
    reset: Signer,
    setup: Signer,
    /// Encrypts the two-factor secrets in the database.
    vault: Encrypter,
}

impl AuthState {
    fn path(&self, page: &str) -> String {
        format!("{}{page}", self.config.prefix)
    }
}

/// The routes, the middleware that reads the session, and the `users` table.
pub(crate) fn install(app: &mut vitesse::App, config: Auth) {
    let secret = config
        .secret
        .clone()
        .or_else(|| std::env::var("APP_KEY").ok().filter(|k| !k.trim().is_empty()))
        .unwrap_or_else(|| {
            eprintln!(
                "📦 Paquets auth: no APP_KEY: a random key is used, so sessions and two-factor codes \
                 stop working when the server restarts (set APP_KEY to 32+ random bytes: \
                 `openssl rand -base64 48`)"
            );
            securite::totp::recovery_codes(8).concat()
        });
    assert!(
        secret.len() >= securite::signing::MIN_SECRET_BYTES,
        "paquets auth: the secret (APP_KEY) must be at least 32 bytes long"
    );
    let signer = |purpose: &str| {
        let purpose = format!("paquets-auth-{purpose}");
        config
            .previous_secrets
            .iter()
            .fold(Signer::new(&secret).purpose(&purpose), |signer, old| {
                signer.fallback_key(old)
            })
    };
    // A key of its own for encryption, derived from the secret: never the signing key.
    let vault = config.previous_secrets.iter().fold(
        Encrypter::new(&derive_key(&secret, "two-factor-secrets"))
            .purpose("paquets-auth-two-factor-secret"),
        |vault, old| vault.fallback_key(&derive_key(old, "two-factor-secrets")),
    );
    let state = AuthState {
        vault,
        session: signer("session"),
        pending: signer("two-factor"),
        reset: signer("password-reset"),
        setup: signer("two-factor-setup"),
        config,
    };
    let page = |summary: &str| {
        Operation::new()
            .summary(summary)
            .tag("auth")
            .extend(json!({ crate::docs::PAGE_MARK: true }))
    };
    let p = |path: &str| state.path(path);
    if state.config.registration {
        app.get(&p("/register"), register_page)
            .doc(page("Sign-up page"));
        app.post(&p("/register"), register).doc(page("Sign up"));
    }
    app.get(&p("/login"), login_page).doc(page("Login page"));
    app.post(&p("/login"), login).doc(page("Log in"));
    app.post(&p("/logout"), logout).doc(page("Log out"));
    app.get(&p("/forgot-password"), forgot_page)
        .doc(page("Forgotten password page"));
    app.post(&p("/forgot-password"), forgot)
        .doc(page("Send a password-reset link"));
    app.get(&p("/reset-password"), reset_page)
        .doc(page("Password-reset page"));
    app.post(&p("/reset-password"), reset)
        .doc(page("Choose a new password"));
    app.get(&p("/account"), account.with_login())
        .doc(page("Account page"));
    if state.config.two_factor {
        app.get(&p("/two-factor"), challenge_page)
            .doc(page("Two-factor code page"));
        app.post(&p("/two-factor"), challenge)
            .doc(page("Check a two-factor code"));
        app.post(&p("/two-factor/enable"), enable_two_factor.with_login())
            .doc(page("Enable two-factor codes"));
        app.post(&p("/two-factor/disable"), disable_two_factor.with_login())
            .doc(page("Disable two-factor codes"));
    }
    app.state(state);
    app.middleware(|mut req: Request, next: Next| async move {
        if let Some(user) = session_user(&req).await {
            req.set(user);
        }
        next.run(req).await
    });
}

/// `.with(login_required)`, for the handlers of this module.
trait WithLogin: vitesse::Handler + Sized {
    fn with_login(self) -> vitesse::Chained {
        vitesse::HandlerExt::with(self, login_required)
    }
}

impl<H: vitesse::Handler> WithLogin for H {}

/// A 32-byte key for `use_`, derived from the application secret with
/// HMAC-SHA256 (the secret is the HMAC key, the use the message): each use gets an
/// independent key, and none of them is the signing key.
fn derive_key(secret: &str, use_: &str) -> [u8; 32] {
    use hmac::{Hmac, KeyInit, Mac};
    let mut mac = <Hmac<sha2::Sha256> as KeyInit>::new_from_slice(secret.as_bytes())
        .expect("HMAC takes any key length");
    mac.update(b"paquets-auth/key/v1/");
    mac.update(use_.as_bytes());
    mac.finalize().into_bytes().into()
}

/// The context an encrypted two-factor secret is bound to: it does not open in
/// another row.
fn secret_context(user: &User) -> String {
    format!("user:{}", user.id)
}

impl AuthState {
    /// Encrypts a base32 TOTP secret for the database.
    fn seal_secret(&self, user: &User, secret: &str) -> String {
        self.vault
            .encrypt_in(&secret_context(user), secret.as_bytes(), None)
    }

    /// The TOTP of a user, and whether it was stored in clear (an older version).
    fn totp_of(&self, user: &User) -> Option<(Totp, bool)> {
        let stored = user.two_factor_secret.as_deref()?;
        match self.vault.decrypt_in(&secret_context(user), stored) {
            Ok(plain) => {
                let plain = String::from_utf8(plain).ok()?;
                Some((Totp::from_base32(&plain).ok()?, false))
            }
            // A secret written in clear before encryption: base32 only.
            Err(_) if is_base32(stored) => Some((Totp::from_base32(stored).ok()?, true)),
            Err(_) => None,
        }
    }
}

fn is_base32(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_uppercase() || (b'2'..=b'7').contains(&b) || b == b'=')
}

fn state(req: &Request) -> &'static AuthState {
    req.state::<AuthState>()
}

/// The user of the session cookie, if it is valid and still current.
async fn session_user(req: &Request) -> Option<User> {
    let state = req.try_state::<AuthState>()?;
    let token = req.cookie(SESSION_COOKIE)?;
    let (id, version) = parse_claim(&state.session.unsign(token).ok()?)?;
    let user = User::find(req.db(), id).await.ok()??;
    (user.session_version == version).then_some(user)
}

/// `"42:3"` → `(42, 3)`: a user id and its session version.
fn parse_claim(claim: &str) -> Option<(i64, i64)> {
    let (id, version) = claim.split_once(':')?;
    Some((id.parse().ok()?, version.parse().ok()?))
}

fn claim(user: &User) -> String {
    format!("{}:{}", user.id, user.session_version)
}

/// Lets only logged-in users through: the others are sent to the login page (or
/// get a `401` when they do not ask for HTML).
///
/// ```
/// use paquets::prelude::*;
/// use paquets::auth::{Auth, login_required};
///
/// let mut app = App::new();
/// app.auth(Auth::new());
/// app.get("/settings", (|_| async { "settings" }).with(login_required));
/// ```
pub async fn login_required(req: Request, next: Next) -> Response {
    if req.user().is_some() {
        next.run(req).await
    } else {
        to_login(&req)
    }
}

/// Lets only administrators ([`User::admin`]) through: `403` for the others,
/// the login page for visitors.
pub async fn admin_required(req: Request, next: Next) -> Response {
    match req.user() {
        Some(user) if user.admin => next.run(req).await,
        Some(_) => crate::Error::forbidden("administrators only").into_response(),
        None => to_login(&req),
    }
}

use vitesse::IntoResponse as _;

/// The login page, coming back here afterwards; a `401` for API clients.
pub(crate) fn to_login(req: &Request) -> Response {
    let wants_html = req
        .header("accept")
        .is_some_and(|a| a.contains("text/html"));
    match req.try_state::<AuthState>() {
        Some(state) if wants_html => {
            let here = match req.uri().path_and_query() {
                Some(pq) => pq.as_str().to_owned(),
                None => req.path().to_owned(),
            };
            see_other(&format!(
                "{}?next={}",
                state.path("/login"),
                encode_component(&here)
            ))
        }
        _ => crate::Error::unauthorized("log in first").into_response(),
    }
}

/// Every field of the forms of this module (each form uses some of them).
#[derive(Deserialize, Default)]
#[serde(default)]
struct Fields {
    name: String,
    email: String,
    password: String,
    password_confirmation: String,
    next: String,
    token: String,
    code: String,
    secret: String,
}

impl fmt::Debug for Fields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fields")
            .field("email", &self.email)
            .finish_non_exhaustive()
    }
}

/// The first message of the first rule that refuses `value`.
fn check(value: &str, rules: &[Rule]) -> Option<String> {
    rules.iter().find_map(|rule| rule.check(value))
}

/// The errors of the sign-up form, field by field (the rules of `active::forms`,
/// the same ones a browser form would run).
fn registration_errors(fields: &Fields) -> Vec<(&'static str, String)> {
    let mut errors = Vec::new();
    if let Some(e) = check(fields.name.trim(), &[required("Enter your name")]) {
        errors.push(("name", e));
    }
    if let Some(e) = check(
        fields.email.trim(),
        &[
            required("Enter your e-mail address"),
            email("This is not an e-mail address"),
        ],
    ) {
        errors.push(("email", e));
    }
    errors.extend(password_errors(fields));
    errors
}

fn password_errors(fields: &Fields) -> Vec<(&'static str, String)> {
    let mut errors = Vec::new();
    if let Some(e) = check(&fields.password, &[min_length(8, "At least 8 characters")]) {
        errors.push(("password", e));
    } else if fields.password != fields.password_confirmation {
        errors.push((
            "password_confirmation",
            "The two passwords are different".into(),
        ));
    }
    errors
}

fn normalize_email(address: &str) -> String {
    address.trim().to_lowercase()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Runs the slow password work on tokio's blocking threads.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    match tokio::task::spawn_blocking(work).await {
        Ok(value) => value,
        Err(error) => std::panic::resume_unwind(error.into_panic()),
    }
}

/// A local path to go back to (never another site).
fn safe_next<'a>(next: &'a str, home: &'a str) -> &'a str {
    let local = next.starts_with('/') && !next.starts_with("//") && !next.contains('\\');
    if local { next } else { home }
}

fn see_other(location: &str) -> Response {
    Response::new()
        .status(StatusCode::SEE_OTHER)
        .header("location", location)
}

fn session_cookie(req: &Request, state: &AuthState, user: &User) -> Cookie {
    Cookie::new(
        SESSION_COOKIE,
        state
            .session
            .sign_with_ttl(&claim(user), state.config.session_ttl),
    )
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(req.secure())
    .max_age(state.config.session_ttl)
}

/// Logs `user` in: the session cookie, then the `next` page.
fn log_in(req: &Request, state: &AuthState, user: &User, next: &str) -> Response {
    see_other(safe_next(next, &state.config.home))
        .cookie(session_cookie(req, state, user))
        .clear_cookie(PENDING_COOKIE)
}

/// Marks the configured administrators.
async fn promote(db: &Db, state: &AuthState, user: &mut User) -> crate::Result<()> {
    if !user.admin && state.config.admins.contains(&user.email) {
        user.admin = true;
        user.update(db).await?;
    }
    Ok(())
}

// ----- Pages ---------------------------------------------------------------------

const STYLE: &str = ".auth{max-width:28rem;margin:3rem auto}\
.auth .error{color:#d33}.auth .notice{border-left:3px solid currentColor;padding-left:.75rem}\
.auth form{display:grid;gap:.75rem}.auth .links{display:flex;gap:1rem;flex-wrap:wrap}\
.auth code{word-break:break-all}";

fn layout(state: &AuthState, title: &str, content: Vec<Element>) -> Page {
    Page::new(format!("{title} · {}", state.config.name))
        .style(STYLE)
        .seo(active::Seo::new().noindex(true))
        .body(
            active::html::main().class("auth").child(
                div()
                    .class("card")
                    .child(h1().text(title.to_owned()))
                    .children(content),
            ),
        )
}

fn error_text(message: &str) -> Element {
    p().class("error")
        .attr("role", "alert")
        .text(message.to_owned())
}

fn notice(message: &str) -> Element {
    p().class("notice")
        .attr("role", "status")
        .text(message.to_owned())
}

fn field(
    title: &str,
    name: &'static str,
    kind: &'static str,
    value: &str,
    errors: &[(&'static str, String)],
) -> Element {
    let mut control = input()
        .attr("type", kind)
        .attr("name", name)
        .attr("id", name)
        .attr("required", "");
    if kind == "password" {
        control = control.attr("autocomplete", "current-password");
    } else {
        control = control.attr("value", value.to_owned());
    }
    if kind == "email" {
        control = control.attr("autocomplete", "email");
    }
    let mut wrapper = label()
        .attr("for", name)
        .child(strong().text(title.to_owned()))
        .child(control);
    if let Some((_, message)) = errors.iter().find(|(f, _)| *f == name) {
        wrapper = wrapper.child(error_text(message));
    }
    wrapper
}

fn hidden(name: &'static str, value: &str) -> Element {
    input()
        .attr("type", "hidden")
        .attr("name", name)
        .attr("value", value.to_owned())
}

fn submit(text: &str) -> Element {
    button().attr("type", "submit").text(text.to_owned())
}

fn post_form(action: String) -> Element {
    form().attr("method", "post").attr("action", action)
}

fn links(items: Vec<(String, &str)>) -> Element {
    p().class("links").children(
        items
            .into_iter()
            .map(|(href, text)| a().href(href).text(text.to_owned())),
    )
}

fn login_view(state: &AuthState, fields: &Fields, error: Option<&str>, status: u16) -> Page {
    let mut content = Vec::new();
    if let Some(message) = error {
        content.push(error_text(message));
    }
    content.push(
        post_form(state.path("/login"))
            .child(field("E-mail", "email", "email", &fields.email, &[]))
            .child(field("Password", "password", "password", "", &[]))
            .child(hidden("next", &fields.next))
            .child(submit("Log in")),
    );
    let mut more = vec![(state.path("/forgot-password"), "Forgot your password?")];
    if state.config.registration {
        more.push((state.path("/register"), "Create an account"));
    }
    content.push(links(more));
    layout(state, "Log in", content).status(status)
}

async fn login_page(req: Request) -> Page {
    let fields = Fields {
        next: req.query("next").unwrap_or_default().into_owned(),
        ..Fields::default()
    };
    let notice_text = req
        .query("reset")
        .map(|_| "Your password was changed: log in with the new one.");
    let state = state(&req);
    let mut page = login_view(state, &fields, None, 200);
    if let Some(text) = notice_text {
        page = page.body(div().class("auth").child(notice(text)));
    }
    page
}

async fn login(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let fields: Fields = req.form().await?;
    let address = normalize_email(&fields.email);
    let key = LoginThrottle::key(&address, req.ip());
    if let Err(wait) = state.config.throttle.check(&key) {
        return Ok(too_many(state, wait, |message| {
            login_view(state, &fields, Some(message), 429)
        }));
    }
    let db = req.db();
    let user = User::find_by_email(db, &address).await?;
    let hasher = state.config.hasher.clone();
    let password = fields.password.clone();
    let hash = user.as_ref().map(|u| u.password_hash.clone());
    let valid = blocking(move || match hash {
        Some(hash) => hasher.verify(&password, &hash),
        None => hasher.verify_dummy(&password),
    })
    .await;
    let Some(mut user) = user.filter(|_| valid) else {
        state.config.throttle.hit(&key);
        return Ok(login_view(
            state,
            &fields,
            Some("These credentials do not match our records."),
            422,
        )
        .into_response());
    };
    state.config.throttle.clear(&key);
    promote(db, state, &mut user).await?;
    if user.two_factor_enabled() && state.config.two_factor {
        let pending = Cookie::new(
            PENDING_COOKIE,
            state
                .pending
                .sign_with_ttl(&claim(&user), Duration::from_secs(300)),
        )
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(req.secure())
        .max_age(Duration::from_secs(300));
        let next = safe_next(&fields.next, &state.config.home);
        return Ok(see_other(&format!(
            "{}?next={}",
            state.path("/two-factor"),
            encode_component(next)
        ))
        .cookie(pending));
    }
    Ok(log_in(&req, state, &user, &fields.next))
}

/// The page of a locked key, with the wait.
fn too_many(state: &AuthState, wait: Duration, view: impl FnOnce(&str) -> Page) -> Response {
    let minutes = wait.as_secs().div_ceil(60).max(1);
    let message = format!("Too many attempts. Try again in {minutes} minute(s).");
    let _ = state;
    view(&message)
        .into_response()
        .header("retry-after", wait.as_secs().max(1).to_string())
}

fn register_view(
    state: &AuthState,
    fields: &Fields,
    errors: &[(&'static str, String)],
    status: u16,
) -> Page {
    let content = vec![
        post_form(state.path("/register"))
            .child(field("Name", "name", "text", &fields.name, errors))
            .child(field("E-mail", "email", "email", &fields.email, errors))
            .child(field("Password", "password", "password", "", errors))
            .child(field(
                "Confirm the password",
                "password_confirmation",
                "password",
                "",
                errors,
            ))
            .child(hidden("next", &fields.next))
            .child(submit("Create my account")),
        links(vec![(state.path("/login"), "I already have an account")]),
    ];
    layout(state, "Create an account", content).status(status)
}

async fn register_page(req: Request) -> Page {
    let fields = Fields {
        next: req.query("next").unwrap_or_default().into_owned(),
        ..Fields::default()
    };
    register_view(state(&req), &fields, &[], 200)
}

async fn register(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let fields: Fields = req.form().await?;
    let mut errors = registration_errors(&fields);
    let db = req.db();
    let address = normalize_email(&fields.email);
    if errors.is_empty() && User::find_by_email(db, &address).await?.is_some() {
        errors.push(("email", "This address already has an account".into()));
    }
    if !errors.is_empty() {
        return Ok(register_view(state, &fields, &errors, 422).into_response());
    }
    let hasher = state.config.hasher.clone();
    let password = fields.password.clone();
    let hash = blocking(move || hasher.hash(&password))
        .await
        .map_err(|e| crate::Error::bad_request(e.to_string()))?;
    let mut user = User {
        id: 0,
        email: address,
        name: fields.name.trim().to_owned(),
        password_hash: hash,
        two_factor_secret: None,
        admin: false,
        session_version: 0,
        created_at: now(),
    };
    user.admin = state.config.admins.contains(&user.email);
    if let Err(error) = user.insert(db).await {
        if matches!(error, memoire_orm::Error::UniqueViolation { .. }) {
            let errors = [("email", "This address already has an account".to_owned())];
            return Ok(register_view(state, &fields, &errors, 422).into_response());
        }
        return Err(error.into());
    }
    Ok(log_in(&req, state, &user, &fields.next))
}

async fn logout(req: Request) -> Response {
    let home = match req.try_state::<AuthState>() {
        Some(state) => state.path("/login"),
        None => "/".to_owned(),
    };
    see_other(&home).clear_cookie(SESSION_COOKIE)
}

fn forgot_view(state: &AuthState, fields: &Fields, message: Option<Element>, status: u16) -> Page {
    let mut content = vec![p().class("muted").text(
        "Give the address of your account: we will send you a link to choose a new password.",
    )];
    content.extend(message);
    content.push(
        post_form(state.path("/forgot-password"))
            .child(field("E-mail", "email", "email", &fields.email, &[]))
            .child(submit("Send the link")),
    );
    content.push(links(vec![(
        state.path("/login"),
        "Back to the login page",
    )]));
    layout(state, "Forgotten password", content).status(status)
}

async fn forgot_page(req: Request) -> Page {
    forgot_view(state(&req), &Fields::default(), None, 200)
}

async fn forgot(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let fields: Fields = req.form().await?;
    let address = normalize_email(&fields.email);
    let key = format!("reset|{}", LoginThrottle::key(&address, req.ip()));
    if let Err(wait) = state.config.throttle.check(&key) {
        return Ok(too_many(state, wait, |message| {
            forgot_view(state, &fields, Some(error_text(message)), 429)
        }));
    }
    state.config.throttle.hit(&key);
    if let Some(user) = User::find_by_email(req.db(), &address).await? {
        let token = state
            .reset
            .sign_with_ttl(&claim(&user), Duration::from_secs(3600));
        let url = format!(
            "{}{}?token={}",
            base_url(&req, state),
            state.path("/reset-password"),
            encode_component(&token)
        );
        let link = ResetLink { user, url };
        match &state.config.mailer {
            Some(send) => send(link).await?,
            None if cfg!(debug_assertions) => {
                println!("📧 Password reset for {}: {}", link.user.email, link.url);
            }
            None => eprintln!(
                "📦 Paquets auth: a password reset was asked, but no mailer is configured (Auth::mailer)"
            ),
        }
    }
    // The same answer whether the account exists or not.
    let sent = notice("If an account exists for this address, a link is on its way.");
    Ok(forgot_view(state, &Fields::default(), Some(sent), 200).into_response())
}

fn base_url(req: &Request, state: &AuthState) -> String {
    if let Some(url) = &state.config.url {
        return url.clone();
    }
    if let Ok(url) = std::env::var("APP_URL")
        && !url.trim().is_empty()
    {
        return url.trim().trim_end_matches('/').to_owned();
    }
    format!("{}://{}", req.protocol(), req.host().unwrap_or("localhost"))
}

fn reset_view(
    state: &AuthState,
    token: &str,
    errors: &[(&'static str, String)],
    status: u16,
) -> Page {
    let content = vec![
        post_form(state.path("/reset-password"))
            .child(field("New password", "password", "password", "", errors))
            .child(field(
                "Confirm the password",
                "password_confirmation",
                "password",
                "",
                errors,
            ))
            .child(hidden("token", token))
            .child(submit("Change my password")),
    ];
    layout(state, "Choose a new password", content).status(status)
}

fn invalid_link(state: &AuthState) -> Response {
    layout(
        state,
        "Link expired",
        vec![
            error_text("This link is invalid, expired or already used."),
            links(vec![(state.path("/forgot-password"), "Ask for a new link")]),
        ],
    )
    .status(400)
    .into_response()
}

/// The user of a reset token, if the token is valid and unused.
async fn reset_user(req: &Request, state: &AuthState, token: &str) -> crate::Result<Option<User>> {
    let Some((id, version)) = state
        .reset
        .unsign(token)
        .ok()
        .as_deref()
        .and_then(parse_claim)
    else {
        return Ok(None);
    };
    let user = User::find(req.db(), id).await?;
    Ok(user.filter(|u| u.session_version == version))
}

async fn reset_page(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let token = req.query("token").unwrap_or_default().into_owned();
    if reset_user(&req, state, &token).await?.is_none() {
        return Ok(invalid_link(state));
    }
    Ok(reset_view(state, &token, &[], 200).into_response())
}

async fn reset(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let fields: Fields = req.form().await?;
    let Some(mut user) = reset_user(&req, state, &fields.token).await? else {
        return Ok(invalid_link(state));
    };
    let errors = password_errors(&fields);
    if !errors.is_empty() {
        return Ok(reset_view(state, &fields.token, &errors, 422).into_response());
    }
    let hasher = state.config.hasher.clone();
    let password = fields.password.clone();
    user.password_hash = blocking(move || hasher.hash(&password))
        .await
        .map_err(|e| crate::Error::bad_request(e.to_string()))?;
    // Every session and link made before is now invalid.
    user.session_version += 1;
    user.update(req.db()).await?;
    Ok(see_other(&format!("{}?reset=1", state.path("/login"))).clear_cookie(SESSION_COOKIE))
}

// ----- Account and two-factor codes ------------------------------------------------

fn account_view(state: &AuthState, user: &User, message: Option<Element>, status: u16) -> Page {
    let mut content = vec![
        ul().child(li().text(format!("Name: {}", user.name)))
            .child(li().text(format!("E-mail: {}", user.email)))
            .child(li().text(if user.admin {
                "Role: administrator"
            } else {
                "Role: user"
            })),
    ];
    content.extend(message);
    if state.config.two_factor {
        if user.two_factor_enabled() {
            content.push(p().text("Two-factor codes are on."));
            content.push(
                post_form(state.path("/two-factor/disable"))
                    .child(field("Current code", "code", "text", "", &[]))
                    .child(
                        button()
                            .attr("type", "submit")
                            .class("ghost")
                            .text("Turn off two-factor codes"),
                    ),
            );
        } else {
            let totp = Totp::generate();
            let secret = totp.secret_base32();
            let uri = totp.provisioning_uri(&state.config.name, &user.email);
            content.push(p().text(
                "Two-factor codes: add this key to an authenticator app, then type its code.",
            ));
            content.push(
                p().child(
                    a().href(uri)
                        .child(active::html::code().text(secret.clone())),
                ),
            );
            content.push(
                post_form(state.path("/two-factor/enable"))
                    .child(hidden(
                        "secret",
                        &state.setup.sign_with_ttl(&secret, Duration::from_secs(900)),
                    ))
                    .child(field("Code", "code", "text", "", &[]))
                    .child(submit("Turn on two-factor codes")),
            );
        }
    }
    content.push(
        post_form(state.path("/logout")).child(
            button()
                .attr("type", "submit")
                .class("ghost")
                .text("Log out"),
        ),
    );
    layout(state, "My account", content).status(status)
}

fn current(req: &Request) -> crate::Result<User> {
    req.user()
        .cloned()
        .ok_or_else(|| crate::Error::unauthorized("log in first"))
}

async fn account(req: Request) -> crate::Result<Page> {
    let user = current(&req)?;
    let state = state(&req);
    Ok(account_view(state, &user, None, 200))
}

async fn enable_two_factor(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let mut user = current(&req)?;
    let fields: Fields = req.form().await?;
    let totp = state
        .setup
        .unsign(&fields.secret)
        .ok()
        .and_then(|secret| Totp::from_base32(&secret).ok());
    let Some(totp) = totp.filter(|t| t.verify(fields.code.trim())) else {
        let message = error_text("This code is not right: try the next one.");
        return Ok(account_view(state, &user, Some(message), 422).into_response());
    };
    user.two_factor_secret = Some(state.seal_secret(&user, &totp.secret_base32()));
    user.update(req.db()).await?;
    let message = notice("Two-factor codes are on: you will be asked for one at each login.");
    Ok(account_view(state, &user, Some(message), 200).into_response())
}

async fn disable_two_factor(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let mut user = current(&req)?;
    let fields: Fields = req.form().await?;
    if !verify_code(state, &user, &fields.code) {
        let message = error_text("This code is not right.");
        return Ok(account_view(state, &user, Some(message), 422).into_response());
    }
    user.two_factor_secret = None;
    user.update(req.db()).await?;
    let message = notice("Two-factor codes are off.");
    Ok(account_view(state, &user, Some(message), 200).into_response())
}

fn verify_code(state: &AuthState, user: &User, code: &str) -> bool {
    state
        .totp_of(user)
        .is_some_and(|(totp, _)| totp.verify(code.trim()))
}

fn challenge_view(state: &AuthState, next: &str, error: Option<&str>, status: u16) -> Page {
    let mut content = vec![
        p().class("muted")
            .text("Type the code of your authenticator app."),
    ];
    if let Some(message) = error {
        content.push(error_text(message));
    }
    content.push(
        post_form(state.path("/two-factor"))
            .child(
                field("Code", "code", "text", "", &[]).child(
                    input()
                        .attr("type", "hidden")
                        .attr("name", "next")
                        .attr("value", next.to_owned()),
                ),
            )
            .child(submit("Continue")),
    );
    layout(state, "Two-factor code", content).status(status)
}

/// The user who typed the right password and must now give a code.
async fn pending_user(req: &Request, state: &AuthState) -> crate::Result<Option<User>> {
    let claim = req
        .cookie(PENDING_COOKIE)
        .and_then(|token| state.pending.unsign(token).ok());
    let Some((id, version)) = claim.as_deref().and_then(parse_claim) else {
        return Ok(None);
    };
    Ok(User::find(req.db(), id)
        .await?
        .filter(|u| u.session_version == version))
}

async fn challenge_page(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    if pending_user(&req, state).await?.is_none() {
        return Ok(see_other(&state.path("/login")));
    }
    let next = req.query("next").unwrap_or_default().into_owned();
    Ok(challenge_view(state, &next, None, 200).into_response())
}

async fn challenge(req: Request) -> crate::Result<Response> {
    let state = state(&req);
    let Some(user) = pending_user(&req, state).await? else {
        return Ok(see_other(&state.path("/login")));
    };
    let fields: Fields = req.form().await?;
    let key = format!("two-factor|{}", user.id);
    match state
        .config
        .throttle
        .attempt(&key, || verify_code(state, &user, &fields.code))
    {
        Ok(true) => {
            // A secret stored in clear by an earlier version: encrypted now.
            if let Some((totp, true)) = state.totp_of(&user) {
                let mut user = user.clone();
                user.two_factor_secret = Some(state.seal_secret(&user, &totp.secret_base32()));
                user.update(req.db()).await?;
            }
            Ok(log_in(&req, state, &user, &fields.next))
        }
        Ok(false) => Ok(
            challenge_view(state, &fields.next, Some("This code is not right."), 422)
                .into_response(),
        ),
        Err(wait) => Ok(too_many(state, wait, |message| {
            challenge_view(state, &fields.next, Some(message), 429)
        })),
    }
}

/// Percent-encodes a query parameter.
pub(crate) fn encode_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_pages_stay_on_the_site() {
        assert_eq!(safe_next("/dashboard?tab=1", "/"), "/dashboard?tab=1");
        assert_eq!(safe_next("https://evil.example", "/"), "/");
        assert_eq!(safe_next("//evil.example", "/"), "/");
        assert_eq!(safe_next("/\\evil.example", "/"), "/");
        assert_eq!(safe_next("", "/home"), "/home");
    }

    #[test]
    fn claims_round_trip() {
        assert_eq!(parse_claim("42:3"), Some((42, 3)));
        assert_eq!(parse_claim("42"), None);
        assert_eq!(parse_claim("x:1"), None);
    }

    #[test]
    fn registration_is_validated_with_the_rules_of_active() {
        let fields = Fields {
            name: "Ada".into(),
            email: "not an address".into(),
            password: "short".into(),
            ..Fields::default()
        };
        let errors = registration_errors(&fields);
        let names: Vec<&str> = errors.iter().map(|(f, _)| *f).collect();
        assert_eq!(names, ["email", "password"]);
        let fields = Fields {
            name: "Ada".into(),
            email: "ada@example.com".into(),
            password: "long enough".into(),
            password_confirmation: "long enough".into(),
            ..Fields::default()
        };
        assert!(registration_errors(&fields).is_empty());
    }

    #[test]
    fn derived_keys_are_independent() {
        let secret = "a secret that is long enough for the tests ......";
        assert_ne!(derive_key(secret, "a"), derive_key(secret, "b"));
        assert_ne!(
            derive_key(secret, "a"),
            derive_key("another secret, also long enough", "a")
        );
        assert_eq!(derive_key(secret, "a"), derive_key(secret, "a"));
        assert!(is_base32("JBSWY3DPEHPK3PXP"));
        assert!(!is_base32("AQIDBA-x_y"));
    }

    #[test]
    fn debug_redacts_the_secrets() {
        let user = User {
            id: 1,
            email: "ada@example.com".into(),
            name: "Ada".into(),
            password_hash: "$argon2id$v=19$secret-hash".into(),
            two_factor_secret: Some("JBSWY3DPEHPK3PXP".into()),
            admin: false,
            session_version: 0,
            created_at: 0,
        };
        let debug = format!("{user:?}");
        assert!(debug.contains("ada@example.com"));
        assert!(
            !debug.contains("argon2") && !debug.contains("JBSWY3DP"),
            "{debug}"
        );
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn components_are_encoded() {
        assert_eq!(encode_component("/a b?c=d&e"), "/a%20b%3Fc%3Dd%26e");
    }
}
