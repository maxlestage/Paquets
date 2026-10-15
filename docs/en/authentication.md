# Authentication

`app.auth(Auth::new())` adds accounts to an app in one line, like Laravel Breeze: sign up, log in, log out, forgotten passwords, two-factor codes, and the pages that go with them. Everything relies on [Securite](https://securite-2db4c1994be7.herokuapp.com/): Argon2id hashing, login throttling, signed cookies and links, TOTP codes.

## One line

```rust
use paquets::prelude::*;
use paquets::auth::{self, Auth};

async fn dashboard(req: Request) -> String {
    let user = req.user().expect("checked by login_required");
    format!("Hello {}", user.name)
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.auth(Auth::new().name("My app").admins(["me@example.com"]));
    app.get("/dashboard", dashboard.with(auth::login_required));
    app.run()
}
```

Or `paquets make:auth` in a project (see [Generators](generators.md)).

## The pages

| Route | Page |
|---|---|
| `GET/POST /register` | creates an account (name, e-mail, password twice), then logs in |
| `GET/POST /login` | logs in, then goes to `?next=` (a local path only) or to the home page |
| `POST /logout` | logs out |
| `GET/POST /forgot-password` | sends a password-reset link |
| `GET/POST /reset-password?token=…` | chooses a new password |
| `GET /account` | the account: two-factor codes, log out |
| `GET/POST /two-factor` | the code asked after the password, once two-factor codes are on |

The pages are rendered on the server with the default theme, with their own Content-Security-Policy and `noindex`. Their fields are checked with the same rules as Active's forms (`required`, `email`, `min_length`), and invalid forms come back with an error under each field (status `422`).

## In handlers

`req.user()` (the `AuthExt` trait of the prelude) is the logged-in [`User`](https://docs.rs/paquets/latest/paquets/auth/struct.User.html), or `None`:

```rust
use paquets::prelude::*;

async fn hello(req: Request) -> String {
    match req.user() {
        Some(user) => format!("Hello {}", user.name),
        None => "Hello stranger".into(),
    }
}
```

Two middlewares protect routes: `auth::login_required` (visitors are sent to the login page, then come back; API clients get a `401`) and `auth::admin_required` (`403` for users who are not administrators). Users are rows of the `users` table (`id`, `email`, `name`, `admin`, `created_at`…), created at startup: query them like any [model](database.md). The password hash, the two-factor secret and the session version never appear in JSON.

## Settings

| Method | Default |
|---|---|
| `.prefix("/auth")` | the routes at the root (`/login`) |
| `.home("/dashboard")` | `/`, after logging in without `?next=` |
| `.name("My app")` | the title of the pages and of the authenticator apps |
| `.url("https://example.com")` | `APP_URL`, then the request's `Host`: the address of e-mailed links |
| `.secret(…)` | `APP_KEY` (32 bytes or more), or a random key at startup |
| `.previous_secret(…)` | none: old `APP_KEY`s still accepted after a rotation |
| `.registration(false)` | visitors can sign up |
| `.two_factor(false)` | users can turn on two-factor codes |
| `.admins([…])` | the e-mail addresses of the administrators |
| `.session_ttl(duration)` | 30 days |
| `.hasher(PasswordHasher…)` | Argon2id, OWASP parameters |
| `.throttle(LoginThrottle…)` | 5 failures per account and IP, then 15 minutes |
| `.mailer(|link| async {…})` | prints the link in the terminal (debug builds) |

In production, set `APP_KEY` (`openssl rand -base64 48`): without it, a random key is drawn at startup (a warning says so, in every build), so sessions and two-factor codes stop working at every restart. To change it, pass the old one to `.previous_secret(…)` for a while. Set `APP_URL` too, so that reset links never trust the `Host` of a request.

### Sending the reset e-mail

```rust
use paquets::auth::{Auth, ResetLink};

let auth = Auth::new().mailer(|link: ResetLink| async move {
    // Send `link.url` to `link.user.email` with the mail crate of your choice.
    println!("to {}: {}", link.user.email, link.url);
    Ok(())
});
```

The forgotten-password page gives the same answer whether the address has an account or not.

## How it protects you

- **Passwords** are hashed with Argon2id, on tokio's blocking threads; unknown addresses take as long as known ones (`verify_dummy`), so response times reveal nothing.
- **Brute force**: failed logins, reset requests and two-factor codes are counted per account and IP; the sixth failure answers `429` with `Retry-After`.
- **Sessions** are cookies signed with HMAC-SHA256, `HttpOnly`, `SameSite=Lax`, `Secure` over HTTPS. Changing the password ends every session.
- **Reset links** are signed, expire after one hour and work once.
- **Two-factor codes** are TOTP codes (RFC 6238): any authenticator app, with the key shown on the account page.
- **Two-factor secrets are encrypted at rest** with XChaCha20-Poly1305 (Securite's `Encrypter`), with a key derived from `APP_KEY` (HMAC-SHA256, separate from the signing key) and bound to the user's row. A secret stored in clear by an earlier version is still read, and encrypted at the next two-factor login.
- **Logs**: `Debug` of `User` shows `<redacted>` instead of the password hash and the two-factor secret.
- **CSRF**: the shield rejects cross-site form posts (Fetch Metadata).

## In tests

Argon2 is slow on purpose, and slower without optimizations: give tests a fast hasher.

```rust
use paquets::prelude::*;
use paquets::auth::{Auth, PasswordHasher};

fn app() -> App {
    let mut app = App::new();
    app.auth(
        Auth::new()
            .secret("a test secret that is long enough to sign cookies")
            .hasher(PasswordHasher::insecure_fast_for_tests()),
    );
    app
}
```

`paquets make:auth` also compiles Argon2 with optimizations in debug builds (`[profile.dev.package.argon2]` in `Cargo.toml`).

## Next steps

- [Admin panel](admin.md): administrators manage the models in the browser.
- [Security](security.md): the shield in front of every request.
