//! `securite` in front of `vitesse`: the shield of every app, and the
//! Content-Security-Policy that lets the HTML files of `public/` work.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use securite::headers::{ContentSecurityPolicy, SecurityHeaders};
use securite::{ClientIpSource, Shield, ShieldBuilder};
use vitesse::{BoxFuture, Middleware, Next, Request, Response};

/// The protection of [`App::new`](crate::App::new), ready to customize: securite's
/// recommended preset (request id, protocol hygiene, method filter, 2 MiB body
/// limit, rate limit of 20 requests per second per IP with bursts of 100, bot
/// blocker, WAF, CSRF via Fetch Metadata, security headers, hardened cookies),
/// reading the client IP from `X-Forwarded-For` on Heroku.
///
/// ```
/// use paquets::prelude::*;
/// use paquets::securite::prelude::*;
///
/// let mut app = App::new();
/// app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
/// ```
pub fn shield() -> ShieldBuilder {
    let ip = if std::env::var_os("DYNO").is_some() {
        ClientIpSource::heroku()
    } else {
        ClientIpSource::Peer
    };
    ShieldBuilder::recommended().client_ip(ip)
}

/// [`shield()`] as is: the protection used by [`App::new`](crate::App::new).
pub fn default_shield() -> Shield {
    shield().build()
}

/// The default protection of an app that serves `public/`: when its HTML files
/// contain inline scripts or styles (a site built by Trunk, Vite…), the CSP allows
/// exactly those, by hash, plus the stylesheets, scripts and fonts they load from
/// other origins (Google Fonts…). Everything else stays forbidden.
pub(crate) fn shield_for(public: &Path) -> Shield {
    match static_policy(public) {
        Some(csp) => shield()
            .guard(SecurityHeaders::new().content_security_policy(csp))
            .build(),
        None => default_shield(),
    }
}

/// The CSP that lets the HTML files of `dir` work; `None` without HTML files.
pub(crate) fn static_policy(dir: &Path) -> Option<ContentSecurityPolicy> {
    let mut pages = Vec::new();
    collect_html(dir, 0, &mut pages);
    if pages.is_empty() {
        return None;
    }
    let mut scripts = BTreeSet::from(["'self'".to_owned(), "'wasm-unsafe-eval'".to_owned()]);
    let mut styles = BTreeSet::from(["'self'".to_owned()]);
    let mut fonts = BTreeSet::from(["'self'".to_owned(), "data:".to_owned()]);
    for html in &pages {
        scripts.extend(ContentSecurityPolicy::hashes_for_inline_scripts(html));
        styles.extend(ContentSecurityPolicy::hashes_for_inline_styles(html));
        for (tag, attr, origin) in external_origins(html) {
            match (tag, attr) {
                ("script", "src") => scripts.insert(origin),
                ("link", "href") => {
                    if origin == "https://fonts.googleapis.com" {
                        fonts.insert("https://fonts.gstatic.com".to_owned());
                    }
                    styles.insert(origin)
                }
                _ => false,
            };
        }
    }
    Some(
        ContentSecurityPolicy::strict()
            .script_src(scripts)
            .style_src(styles)
            .style_src_attr(["'unsafe-inline'"])
            .font_src(fonts)
            .img_src(["'self'", "data:", "https:"])
            .remove_directive("upgrade-insecure-requests"),
    )
}

/// The `.html` files of `dir` (three levels deep, 200 files at most).
fn collect_html(dir: &Path, depth: usize, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= 200 {
            return;
        }
        let path = entry.path();
        if path.is_dir() && depth < 3 {
            collect_html(&path, depth + 1, out);
        } else if path.extension().is_some_and(|e| e == "html")
            && let Ok(html) = std::fs::read_to_string(&path)
        {
            out.push(html);
        }
    }
}

/// `(tag, attribute, origin)` for every `<script src>` and `<link href>` that
/// points to another site, e.g. `("link", "href", "https://fonts.googleapis.com")`.
fn external_origins(html: &str) -> Vec<(&'static str, &'static str, String)> {
    let mut found = Vec::new();
    for (tag, attr) in [("script", "src"), ("link", "href")] {
        let open = format!("<{tag}");
        let mut rest = html;
        while let Some(start) = rest.find(&open) {
            let after = &rest[start + open.len()..];
            let end = after.find('>').unwrap_or(after.len());
            let attrs = &after[..end];
            if let Some(origin) = attribute(attrs, attr).and_then(origin_of) {
                found.push((tag, attr, origin));
            }
            rest = &after[end..];
        }
    }
    found
}

fn attribute<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = attrs;
    while let Some(i) = rest.find(name) {
        let before = rest[..i].chars().last();
        let after = rest[i + name.len()..].trim_start();
        if before.is_none_or(char::is_whitespace)
            && let Some(value) = after.strip_prefix('=')
        {
            let value = value.trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                return value[1..].split(quote).next();
            }
            return value.split(|c: char| c.is_whitespace()).next();
        }
        rest = &rest[i + name.len()..];
    }
    None
}

/// `https://host[:port]` of an absolute `https://` URL.
pub(crate) fn origin_of(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://")?;
    let host = rest.split(['/', '?', '#']).next()?;
    (!host.is_empty()).then(|| format!("https://{host}"))
}

/// The first global middleware of every app. The shield is chosen when the app
/// starts (`App::security` may be called after routes and middlewares); securite's
/// own Vitesse middleware does the work.
pub(crate) struct ShieldMiddleware(pub(crate) Arc<OnceLock<Option<Shield>>>);

impl Middleware for ShieldMiddleware {
    fn handle(&'static self, req: Request, next: Next) -> BoxFuture<Response> {
        match self.0.get() {
            Some(Some(shield)) => shield.handle(req, next),
            _ => next.run(req),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_external_origins() {
        let html = r#"<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter&display=swap" rel="stylesheet"/>
<link rel="icon" href="/favicon.svg"><script src='https://cdn.example.com/x.js'></script>
<script type="module">import init from "/app.js";init();</script>"#;
        let origins = external_origins(html);
        assert!(origins.contains(&("link", "href", "https://fonts.googleapis.com".into())));
        assert!(origins.contains(&("script", "src", "https://cdn.example.com".into())));
        assert!(!origins.iter().any(|(_, _, o)| o.contains("favicon")));
    }

    #[test]
    fn the_policy_of_a_static_site_allows_its_inline_code_and_fonts() {
        let dir = std::env::temp_dir().join(format!("paquets-public-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        assert!(static_policy(&dir).is_none(), "no HTML yet");
        std::fs::write(
            dir.join("index.html"),
            r#"<html><head><link href="https://fonts.googleapis.com/css2?family=Inter" rel="stylesheet"><script type="module">import init from "/site.js";init();</script></head></html>"#,
        )
        .unwrap();
        std::fs::write(dir.join("docs/page.html"), "<style>p{color:red}</style>").unwrap();
        let csp = static_policy(&dir).unwrap().to_string();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(csp.contains("script-src 'self' 'sha256-"), "{csp}");
        assert!(csp.contains("'wasm-unsafe-eval'"), "{csp}");
        assert!(
            csp.contains("style-src 'self' 'sha256-")
                || csp.contains("https://fonts.googleapis.com"),
            "{csp}"
        );
        assert!(csp.contains("https://fonts.googleapis.com"), "{csp}");
        assert!(
            csp.contains("font-src 'self' data: https://fonts.gstatic.com"),
            "{csp}"
        );
        assert!(
            !csp.contains("unsafe-inline'; ") || csp.contains("style-src-attr"),
            "{csp}"
        );
    }
}
