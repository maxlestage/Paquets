# Security

Every Paquets app is protected from its first line: `App::new()` puts the recommended shield of [Securite](https://maxlestage.github.io/Securite/) in front of every request. Attacks are blocked before your code runs, and every response gets the security headers browsers need. This page explains what the shield does, how to change it, and what remains your job, such as logging users in.

## What the shield does

The default shield runs ten guards, in this order, for every request (about 6 µs in total):

| Guard | Name | Protects against | When it blocks |
|---|---|---|---|
| Request id | `request_id` | — | never: it gives every request an `X-Request-Id`, also sent back in the response |
| Hygiene | `hygiene` | malformed and oversized requests | URI over 8 KiB, more than 100 headers, `..` segments, bad encoding: `400` |
| Method filter | `method` | unusual HTTP methods, method-override tricks | methods other than `GET HEAD POST PUT PATCH DELETE OPTIONS`: `405` |
| Body limit | `body_limit` | huge uploads | announced body over 2 MiB: `413` |
| Rate limit | `rate_limit` | floods, brute force, scraping | more than 20 requests per second per IP, with bursts up to 100: `429` |
| Bot blocker | `bot_blocker` | attack tools | known scanners (sqlmap, nikto, nuclei…): `403` |
| Firewall (WAF) | `waf` | SQL injection, XSS, command injection, Log4Shell, SSRF, path traversal… | an attack pattern in the URL, the query string or a header: `403` |
| CSRF | `csrf` | cross-site request forgery | a `POST`, `PUT`, `PATCH` or `DELETE` sent by a browser from another website: `403` |
| Security headers | `security_headers` | XSS, clickjacking, MIME sniffing, downgrade to HTTP | never: it adds headers to every response |
| Cookie hardening | `cookies` | stolen session cookies | never: it adds `Secure`, `HttpOnly` and `SameSite=Lax` to every cookie |

The startup line confirms it: `security: on (10 guards)`.

### What a blocked request looks like

A blocked request never reaches your handlers. The client gets a short, generic message (no details that would help an attacker), in the standard "problem details" JSON format:

```http
HTTP/1.1 429 Too Many Requests
content-type: application/problem+json
retry-after: 1
ratelimit-limit: 100
ratelimit-remaining: 0

{"type":"about:blank","title":"Too Many Requests","status":429,"code":"rate_limited","detail":"Too many requests, slow down.","request_id":"222caeb2-c3fa-4692-a3de-b6650499864c"}
```

| Code | Status | Guard |
|---|---|---|
| `waf_blocked` | `403` | firewall |
| `csrf_rejected` | `403` | CSRF |
| `rate_limited` | `429` | rate limit |
| `bot_blocked` | `403` | bot blocker |

The request log shows the status (`GET /?q=... 403 0.05 ms`). The `request_id` lets you find a request in your logs when a user reports a problem.

### Security headers

Every response, including errors and blocked requests, carries:

| Header | Value | Effect |
|---|---|---|
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains` | browsers use HTTPS only, for two years |
| `X-Content-Type-Options` | `nosniff` | no guessing of content types |
| `X-Frame-Options` | `DENY` | your pages can't be shown in a frame (clickjacking) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | full URLs are not leaked to other sites |
| `Permissions-Policy` | camera, microphone, geolocation… disabled | third-party code can't use them |
| `Cross-Origin-Opener-Policy`, `Cross-Origin-Resource-Policy` | `same-origin` | isolation from other sites |
| `Content-Security-Policy` | see below | which scripts, styles and resources may load |

## Security headers and the Content-Security-Policy

The Content-Security-Policy (CSP) tells the browser what a page may load and run. Even if an attacker slips some HTML into a page, a strict CSP stops their script from running. Paquets sets it in three ways:

| Response | Policy |
|---|---|
| A [`Page`](pages.md) | its own policy, computed when the page is rendered: the page's inline styles and scripts are allowed by their SHA-256 hash, plus your own files, `https:` images and WebAssembly (see [Pages](pages.md#the-content-security-policy)) |
| The HTML files of `public/` | when `public/` contains `.html` files, the default shield reads them at startup and allows their inline scripts and styles (by hash), the external stylesheets and scripts they link to (Google Fonts included, with `https://fonts.gstatic.com` for its fonts), WebAssembly and `https:` images |
| Everything else (JSON, text, `Html(...)`) | the shield's policy: same-origin resources only, no inline scripts or styles |

The automatic policy for `public/` comes with the default shield. If you replace the shield with `app.security(...)`, set a policy yourself when your static HTML needs one, with the `SecurityHeaders` guard:

```rust
use paquets::securite::headers::ContentSecurityPolicy;
use paquets::securite::prelude::*;

let csp = ContentSecurityPolicy::strict()
    .add_sources("script-src", ["https://cdn.example.com"])
    .remove_directive("upgrade-insecure-requests");
app.security(
    paquets::shield()
        .guard(SecurityHeaders::new().content_security_policy(csp))
        .build(),
);
```

## The client IP behind a proxy

The rate limit counts requests **per IP address**, so the shield must know the real address of each visitor. On Heroku, the app is behind Heroku's router, and every connection comes from the router: the real address is in the `X-Forwarded-For` header. The default shield detects Heroku (the `DYNO` environment variable) and reads that header, from the right, the safe way. Elsewhere, it uses the address of the connection.

On other platforms behind a proxy, tell the shield where to find the address, otherwise every visitor shares the proxy's address, and the rate limit applies to all of them together:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // Render, AWS load balancers, a single Nginx
        // .client_ip(ClientIpSource::fly())        // Fly.io
        // .client_ip(ClientIpSource::cloudflare()) // behind Cloudflare
        .build(),
);
```

> [!WARNING]
> Only trust a forwarding header when your app can be reached **only** through the proxy that sets it. Otherwise anyone can send the header and choose their address. See Securite's [client IP guide](https://maxlestage.github.io/Securite/en/docs/client-ip/).

Handlers can read the address the shield found with `req.extensions().get::<ClientIp>()` (see [Requests and responses](requests-responses.md#the-clients-ip-address)).

## Changing the shield

`paquets::shield()` returns the default shield as a builder, ready to be changed, with the Heroku detection included. Change what you need, call `.build()`, and give the result to `app.security(...)`:

```rust
use paquets::prelude::*;
use paquets::securite::prelude::*;

let mut app = App::new();
app.security(
    paquets::shield()
        .guard(RateLimit::per_minute(600)) // replaces the default rate limit
        .without("bot_blocker")            // removes a guard, by name
        .build(),
);
```

| Builder method | Effect |
|---|---|
| `.guard(g)` | adds a guard, or replaces the guard with the same name |
| `.without(name)` | removes a guard: `"rate_limit"`, `"waf"`, `"csrf"`… |
| `.client_ip(source)` | where the client IP comes from (see above) |
| `.error_format(format)` | how blocked requests are answered: `ErrorFormat::ProblemJson` (default), `PlainText`, `Empty` |
| `.on_event(f)` | a function called for every blocked request |
| `.build()` | the finished shield |

Import the guards with `use paquets::securite::prelude::*;`. `app.security(...)` can be called anywhere before `run()`: the shield is chosen when the app starts.

### Common recipes

**A busier API.** Raise the rate limit, per minute or with a custom burst:

```rust
app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
app.security(paquets::shield().guard(RateLimit::new(Quota::per_second(50).burst(200))).build());
```

**Bigger uploads.** Raise the shield's limit and the read limit together (see [Requests and responses](requests-responses.md#body-size)):

```rust
app.body_limit(10 * 1024 * 1024);
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

**A front end on another origin.** Allow it in CORS, and trust it in the CSRF guard (see [REST API](rest-api.md#calling-the-api-from-a-browser)):

```rust
app.security(
    paquets::shield()
        .guard(Cors::new().allow_origin("https://app.example.com"))
        .guard(Csrf::new().trusted_origin("https://app.example.com"))
        .build(),
);
```

**A form posted back by another site.** Some sign-in and payment providers send the visitor back to your site with a form `POST`. Exempt that path from the CSRF guard:

```rust
app.security(paquets::shield().guard(Csrf::new().exempt("/auth/callback")).build());
```

Webhooks sent by servers (GitHub, Stripe…) don't need this: requests that don't come from a browser pass the CSRF guard.

**See what is blocked.** Print every blocked request:

```rust
app.security(
    paquets::shield()
        .on_event(|e| {
            if e.enforced {
                eprintln!("[security] {} blocked {} {} ({})", e.guard, e.method, e.path, e.status);
            }
        })
        .build(),
);
```

```text
[security] waf blocked GET / (403 Forbidden)
```

**Try a guard without blocking.** Most guards have a monitor mode, which reports through `on_event` (with `e.enforced == false`) instead of blocking: `.guard(Waf::new().monitor())`.

### Presets

Securite has three presets. `paquets::shield()` is the recommended one, plus the Heroku detection:

| Preset | For | Differences with the recommended preset |
|---|---|---|
| `ShieldBuilder::recommended()` | websites and APIs used by browsers | — |
| `ShieldBuilder::strict()` | sensitive sites | adds an HTTPS redirect, automatic bans after repeated blocks, a concurrency limit per IP and a content-type check; lower limits (1 MiB, 10 requests per second); `SameSite=Strict` cookies; `Cross-Origin-Embedder-Policy: require-corp` (images and scripts from other sites must allow it) |
| `ShieldBuilder::api()` | JSON APIs used with tokens, not cookies | no CSRF guard, JSON bodies only, a CSP that allows nothing, `Cache-Control: no-store` |

The presets don't know about Heroku: add the client IP yourself.

```rust
app.security(ShieldBuilder::strict().client_ip(ClientIpSource::heroku()).build());
```

`Shield::strict()` and `Shield::api()` are the same presets, already built.

### Turning the shield off

```rust
app.security(None);
```

Only do this when another layer already protects the app (a WAF at your CDN, for example), or for a benchmark. The startup line then says `security: off`.

## What the shield doesn't do

- **It doesn't read request bodies.** The firewall inspects the URL, the query string and the headers. JSON and form bodies are your handlers' job: validate them (see [Requests and responses](requests-responses.md#json-bodies)). Two things already protect you there: Memoire sends every value as a bound parameter, so model queries can't be turned into SQL injection, and views escape all text, so data shown in a page can't inject HTML.
- **It doesn't know your users.** Authentication and permissions are up to you (next section).
- **It doesn't provide HTTPS by itself.** Paquets serves plain HTTP; Heroku, a reverse proxy or a CDN provides HTTPS in front of it. The `Strict-Transport-Security` header then keeps browsers on HTTPS.

## Authentication

Paquets doesn't impose a way to log users in. Three common approaches, from the simplest:

### A shared token for an API

For an API used by your own scripts or apps, check a secret token in a middleware: see [REST API](rest-api.md#protecting-the-api).

### A password for an admin section

Securite's `Auth` guard protects paths with HTTP Basic authentication (the browser shows a login box), bearer tokens or API keys:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .guard(Auth::basic("Admin").user("admin", "correct horse battery").protect(["/admin"]))
        .build(),
);

app.get("/admin", |req: Request| async move {
    let name = req.extensions().get::<Principal>().map(|p| p.name.to_string()).unwrap_or_default();
    format!("Hello {name}")
});
```

Every path under `/admin` now needs the user name and password; the others stay public. Read the password from an environment variable (`std::env::var("ADMIN_PASSWORD")`) rather than writing it in the code. Securite's [authentication guide](https://maxlestage.github.io/Securite/en/docs/auth/) covers tokens, API keys and custom checks such as JWT.

### Accounts and sessions

For user accounts, the usual pattern is:

1. store users in a model, with a password **hash** (use a dedicated crate such as `argon2`, never a plain hash or the password itself);
2. at login, check the password, create a random session token (from a cryptographic random generator), store it in a `Session` model, and send it in a cookie: `res::redirect("/").cookie(Cookie::new("session", token))`;
3. in a middleware, read `req.cookie("session")`, look the session up with `req.db()`, and attach the user to the request with `req.set(user)`; handlers read it with `req.get::<User>()`.

The shield takes care of the rest: the cookie gets `Secure`, `HttpOnly` and `SameSite=Lax`, the rate limit slows down password guessing, and the CSRF guard stops other sites from posting forms in your users' name.

## Learn more

Securite's documentation has one page per guard: [WAF](https://maxlestage.github.io/Securite/en/docs/waf/), [rate limit](https://maxlestage.github.io/Securite/en/docs/rate-limit/), [CSRF](https://maxlestage.github.io/Securite/en/docs/csrf/), [security headers](https://maxlestage.github.io/Securite/en/docs/security-headers/), [presets](https://maxlestage.github.io/Securite/en/docs/shield-presets/), and more: IP filter, host filter, automatic bans, webhook signatures. In Paquets, Securite is re-exported as `paquets::securite`.

## Next steps

- [Errors](errors.md): how errors become responses.
- [Configuration](configuration.md): every setting of the app.
- [Deploy to Heroku](deploy-heroku-mobile.md): HTTPS and the client IP are handled for you.
