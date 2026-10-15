//! `app.auth(…)`: sign up, log in, protected routes, throttling, password resets,
//! two-factor codes, and the admin panel for administrators.

use std::sync::Mutex;

use paquets::auth::{self, Auth, PasswordHasher, Totp};
use paquets::prelude::*;
use paquets::test::{TestClient, TestResponse};

const SECRET: &str = "a test secret that is long enough to sign cookies";

static LINKS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[model(table = "notes")]
struct Note {
    id: i64,
    text: String,
}

fn app() -> App {
    let mut app = App::new();
    app.auth(
        Auth::new()
            .name("Test")
            .secret(SECRET)
            .url("https://example.com")
            .hasher(PasswordHasher::insecure_fast_for_tests())
            .admins(["ada@example.com"])
            .mailer(|link: auth::ResetLink| async move {
                LINKS.lock().unwrap().push(link.url);
                Ok(())
            }),
    );
    app.admin(Admin::new().model::<Note>());
    app.get(
        "/dashboard",
        (|req: Request| async move {
            let user = req.user().expect("checked by login_required");
            format!("Hello {}", user.name)
        })
        .with(auth::login_required),
    );
    app.get(
        "/staff",
        (|_| async { "staff only" }).with(auth::admin_required),
    );
    // What the database holds for the logged-in user's two-factor secret.
    app.get(
        "/stored-secret",
        (|req: Request| async move {
            let user = req.user().expect("logged in");
            user.two_factor_secret.clone().unwrap_or_default()
        })
        .with(auth::login_required),
    );
    // Writes a secret in clear, as an earlier version of Paquets did.
    app.post(
        "/legacy-secret",
        (|req: Request| async move {
            let mut user = req.user().expect("logged in").clone();
            let secret: String = req.text().await?;
            user.two_factor_secret = Some(secret);
            user.update(req.db()).await?;
            Ok::<_, Error>("stored")
        })
        .with(auth::login_required),
    );
    app
}

/// The `name=value` of the cookie `name` set by a response.
fn cookie(res: &TestResponse, name: &str) -> Option<String> {
    res.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with(&format!("{name}=")) && !v.starts_with(&format!("{name}=;")))
        .map(|v| v.split(';').next().unwrap().to_owned())
}

async fn register(client: &TestClient, name: &str, email: &str) -> String {
    let res = client
        .post("/register")
        .form(&[
            ("name", name),
            ("email", email),
            ("password", "correct horse"),
            ("password_confirmation", "correct horse"),
        ])
        .await;
    assert_eq!(res.status(), 303, "{}", res.text());
    cookie(&res, auth::SESSION_COOKIE).expect("a session cookie")
}

/// The `value` of the `<input name="name">` of a page.
fn input_value(html: &str, name: &str) -> String {
    let at = html.find(&format!(r#"name="{name}""#)).expect(name);
    let rest = &html[at..];
    let start = rest.find(r#"value=""#).unwrap() + 7;
    let end = rest[start..].find('"').unwrap();
    rest[start..start + end].replace("&amp;", "&")
}

#[tokio::test]
async fn sign_up_log_in_and_protected_routes() {
    let client = app().test().await;
    let page = client.get("/register").await;
    assert_eq!(page.status(), 200);
    assert!(page.text().contains(r#"action="/register""#));

    // Visitors are sent to the login page, and come back afterwards.
    let res = client.get("/dashboard").header("accept", "text/html").await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.header("location"), Some("/login?next=/dashboard"));
    // API clients get a 401.
    assert_eq!(client.get("/dashboard").await.status(), 401);

    let session = register(&client, "Ada", "Ada@Example.com").await;
    let res = client.get("/dashboard").header("cookie", &session).await;
    assert_eq!(res.text(), "Hello Ada");
    // The address of `admins(…)` is an administrator.
    let res = client.get("/staff").header("cookie", &session).await;
    assert_eq!(res.status(), 200);

    // The same address again: refused.
    let res = client
        .post("/register")
        .form(&[
            ("name", "Ada"),
            ("email", "ada@example.com"),
            ("password", "another one"),
            ("password_confirmation", "another one"),
        ])
        .await;
    assert_eq!(res.status(), 422);
    assert!(res.text().contains("already has an account"));

    // Invalid fields, with the rules of active's forms.
    let res = client
        .post("/register")
        .form(&[("name", "Bob"), ("email", "bob"), ("password", "short")])
        .await;
    assert_eq!(res.status(), 422);
    assert!(res.text().contains("This is not an e-mail address"));
    assert!(res.text().contains("At least 8 characters"));

    // Log in, to the `next` page.
    let res = client
        .post("/login")
        .form(&[
            ("email", "ada@example.com"),
            ("password", "correct horse"),
            ("next", "/dashboard"),
        ])
        .await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.header("location"), Some("/dashboard"));
    assert!(cookie(&res, auth::SESSION_COOKIE).is_some());

    // Never to another site.
    let res = client
        .post("/login")
        .form(&[
            ("email", "ada@example.com"),
            ("password", "correct horse"),
            ("next", "https://evil.example"),
        ])
        .await;
    assert_eq!(res.header("location"), Some("/"));

    // A forged cookie is ignored.
    let res = client
        .get("/dashboard")
        .header("cookie", "paquets_session=1:0.forged")
        .await;
    assert_eq!(res.status(), 401);

    let res = client.post("/logout").header("cookie", &session).await;
    assert_eq!(res.status(), 303);
    assert!(
        res.headers()
            .get_all("set-cookie")
            .iter()
            .any(|v| v.to_str().unwrap().starts_with("paquets_session=;"))
    );
}

#[tokio::test]
async fn failed_logins_are_throttled() {
    let client = app().test().await;
    register(&client, "Bob", "bob@example.com").await;
    for _ in 0..5 {
        let res = client
            .post("/login")
            .form(&[("email", "bob@example.com"), ("password", "wrong password")])
            .await;
        assert_eq!(res.status(), 422);
        assert!(res.text().contains("do not match"));
    }
    let res = client
        .post("/login")
        .form(&[("email", "bob@example.com"), ("password", "correct horse")])
        .await;
    assert_eq!(res.status(), 429);
    assert!(res.header("retry-after").is_some());
    assert!(res.text().contains("Too many attempts"));
}

#[tokio::test]
async fn passwords_are_reset_with_a_signed_link_that_works_once() {
    let client = app().test().await;
    let old_session = register(&client, "Cleo", "cleo@example.com").await;

    let res = client
        .post("/forgot-password")
        .form(&[("email", "cleo@example.com")])
        .await;
    assert_eq!(res.status(), 200);
    assert!(res.text().contains("a link is on its way"));
    // Unknown addresses get the same answer, and no link.
    let res = client
        .post("/forgot-password")
        .form(&[("email", "nobody@example.com")])
        .await;
    assert!(res.text().contains("a link is on its way"));

    let link = LINKS
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find(|l| l.starts_with("https://example.com/reset-password?token="))
        .cloned()
        .expect("a reset link");
    let path = link.trim_start_matches("https://example.com");
    assert_eq!(client.get(path).await.status(), 200);
    let token = input_value(&client.get(path).await.text(), "token");

    let res = client
        .post("/reset-password")
        .form(&[
            ("token", token.as_str()),
            ("password", "a brand new one"),
            ("password_confirmation", "a brand new one"),
        ])
        .await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.header("location"), Some("/login?reset=1"));

    // The old session and the link are no longer valid.
    let res = client
        .get("/dashboard")
        .header("cookie", &old_session)
        .await;
    assert_eq!(res.status(), 401);
    assert_eq!(client.get(path).await.status(), 400);
    let res = client
        .post("/login")
        .form(&[
            ("email", "cleo@example.com"),
            ("password", "a brand new one"),
        ])
        .await;
    assert_eq!(res.status(), 303);
}

#[tokio::test]
async fn two_factor_codes() {
    let client = app().test().await;
    let session = register(&client, "Dan", "dan@example.com").await;
    let account = client.get("/account").header("cookie", &session).await;
    assert_eq!(account.status(), 200);
    let html = account.text();
    let signed = input_value(&html, "secret");
    let base32 = html
        .split("<code>")
        .nth(1)
        .and_then(|rest| rest.split("</code>").next())
        .unwrap()
        .to_owned();
    let totp = Totp::from_base32(&base32).unwrap();

    let res = client
        .post("/two-factor/enable")
        .header("cookie", &session)
        .form(&[("secret", signed.as_str()), ("code", "000000")])
        .await;
    assert_eq!(res.status(), 422);
    let res = client
        .post("/two-factor/enable")
        .header("cookie", &session)
        .form(&[("secret", signed.as_str()), ("code", &totp.current_code())])
        .await;
    assert_eq!(res.status(), 200);
    assert!(res.text().contains("Two-factor codes are on"));
    // Encrypted at rest: neither the secret nor base32.
    let stored = client
        .get("/stored-secret")
        .header("cookie", &session)
        .await
        .text();
    assert!(!stored.is_empty() && stored != base32 && !stored.contains(&base32));

    // The password is no longer enough.
    let res = client
        .post("/login")
        .form(&[("email", "dan@example.com"), ("password", "correct horse")])
        .await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.header("location"), Some("/two-factor?next=/"));
    assert!(cookie(&res, auth::SESSION_COOKIE).is_none());
    let pending = cookie(&res, "paquets_two_factor").unwrap();

    let res = client
        .post("/two-factor")
        .header("cookie", &pending)
        .form(&[("code", "000000"), ("next", "/dashboard")])
        .await;
    assert_eq!(res.status(), 422);
    let res = client
        .post("/two-factor")
        .header("cookie", &pending)
        .form(&[
            ("code", totp.current_code().as_str()),
            ("next", "/dashboard"),
        ])
        .await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.header("location"), Some("/dashboard"));
    let session = cookie(&res, auth::SESSION_COOKIE).unwrap();
    let res = client.get("/dashboard").header("cookie", &session).await;
    assert_eq!(res.text(), "Hello Dan");
}

#[tokio::test]
async fn the_admin_panel_is_for_administrators() {
    let client = app().test().await;
    // Visitors log in first.
    let res = client.get("/admin").header("accept", "text/html").await;
    assert_eq!(res.status(), 303);
    let user = register(&client, "Eve", "eve@example.com").await;
    assert_eq!(
        client.get("/admin").header("cookie", &user).await.status(),
        403
    );
    assert_eq!(
        client.get("/staff").header("cookie", &user).await.status(),
        403
    );

    let admin = register(&client, "Ada", "ada@example.com").await;
    let res = client.get("/admin").header("cookie", &admin).await;
    assert_eq!(res.status(), 200);
    let html = res.text();
    assert!(html.contains("Note") && html.contains("0 row(s)"), "{html}");

    // The users are not in the panel unless asked; notes are.
    let res = client
        .post("/admin/notes/new")
        .header("cookie", &admin)
        .form(&[("text", "From the admin")])
        .await;
    assert_eq!(res.status(), 303);
    let res = client.get("/admin/notes").header("cookie", &admin).await;
    assert!(res.text().contains("From the admin"));
}

#[tokio::test]
async fn secrets_stored_in_clear_still_work_and_get_encrypted() {
    let client = app().test().await;
    let session = register(&client, "Fay", "fay@example.com").await;
    let totp = Totp::generate();
    let base32 = totp.secret_base32();
    let res = client
        .post("/legacy-secret")
        .header("cookie", &session)
        .body(base32.clone())
        .await;
    assert_eq!(res.status(), 200);

    let res = client
        .post("/login")
        .form(&[("email", "fay@example.com"), ("password", "correct horse")])
        .await;
    let pending = cookie(&res, "paquets_two_factor").expect("a code is asked");
    let res = client
        .post("/two-factor")
        .header("cookie", &pending)
        .form(&[("code", totp.current_code().as_str()), ("next", "/")])
        .await;
    assert_eq!(res.status(), 303);
    let session = cookie(&res, auth::SESSION_COOKIE).unwrap();
    let stored = client
        .get("/stored-secret")
        .header("cookie", &session)
        .await
        .text();
    assert_ne!(stored, base32, "encrypted after the login");
    assert!(!stored.is_empty());
}

#[test]
fn users_are_redacted_in_debug() {
    let user = auth::User {
        id: 7,
        email: "gus@example.com".into(),
        name: "Gus".into(),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA".into(),
        two_factor_secret: Some("JBSWY3DPEHPK3PXP".into()),
        admin: false,
        session_version: 0,
        created_at: 0,
    };
    let debug = format!("{user:?}");
    assert!(debug.contains("gus@example.com"));
    assert!(!debug.contains("argon2id") && !debug.contains("JBSWY3DP"));
}
