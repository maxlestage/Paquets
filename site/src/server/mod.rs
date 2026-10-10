//! The server: a `paquets::App` with the pages in three languages, the docs search
//! index, the sitemap and the live demo. Static files come from `public/` and the
//! browser bundle from `pkg/` (both found by Paquets in the current directory).

pub mod demo;
pub mod docs;
pub mod export;
pub mod markdown;
pub mod pages;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use paquets::prelude::*;

use crate::i18n::{Lang, pick_lang};
use crate::routes::{self, Page as PageKind, Route, base, link};
use docs::Docs;

pub struct Site {
    pub docs: Docs,
    /// The public address, with the base path (`SITE_URL`); `None`: taken from each
    /// request.
    pub site_url: Option<String>,
    /// The running server, with the live demo; `false` for the static export.
    pub live: bool,
    /// Short hash of `public/site.css`, for `?v=` in its link.
    pub css_version: String,
}

/// The `site/` folder: `SITE_DIR`, the current directory when it has `public/`, else
/// the crate folder.
pub fn site_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SITE_DIR") {
        return PathBuf::from(dir);
    }
    if Path::new("public").is_dir() {
        return PathBuf::from(".");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// FNV-1a, enough to tell versions of a file apart.
pub fn version(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")[..10].to_owned()
}

impl Site {
    /// Reads the docs and the stylesheet. Call [`routes::set_base`] before.
    pub fn load(live: bool, site_url: Option<String>) -> Site {
        let css = std::fs::read(site_dir().join("public/site.css")).unwrap_or_default();
        Site {
            docs: Docs::load(&docs::docs_dir()),
            site_url: site_url
                .map(|u| u.trim().trim_end_matches('/').to_owned())
                .filter(|u| !u.is_empty())
                .map(|u| {
                    if u.ends_with(base()) {
                        u
                    } else {
                        format!("{u}{}", base())
                    }
                }),
            live,
            css_version: version(&css),
        }
    }

    /// The public address of the site, with the base path.
    fn origin(&self, req: &Request) -> String {
        if let Some(url) = &self.site_url {
            return url.clone();
        }
        let header = |name: &str| {
            req.header(name)
                .map(|v| v.split(',').next().unwrap_or("").trim().to_owned())
        };
        let host = header("x-forwarded-host")
            .or_else(|| header("host"))
            .unwrap_or_else(|| "localhost".into());
        if host.is_empty()
            || !host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        {
            return format!("http://localhost{}", base());
        }
        let proto = if header("x-forwarded-proto").as_deref() == Some("https") {
            "https"
        } else {
            "http"
        };
        format!("{proto}://{host}{}", base())
    }
}

/// `/`: the home page of the preferred language of the browser.
fn root(req: &Request) -> Response {
    let lang = pick_lang(req.header("accept-language").unwrap_or(""));
    Redirect::to(link(&format!("/{}/", lang.code())))
        .into_response()
        .header("vary", "Accept-Language")
        .header("cache-control", "no-cache")
}

/// A page of the site: `/{lang}/`, `/{lang}/docs/`, `/{lang}/docs/{slug}/`.
fn page(site: &Site, req: &Request) -> Response {
    let path = req.path();
    let Some(route) = Route::parse(path) else {
        return not_found(site, req);
    };
    if route.page == PageKind::NotFound {
        return not_found(site, req);
    }
    if !path.ends_with('/') {
        let query = req
            .uri()
            .query()
            .map(|q| format!("?{q}"))
            .unwrap_or_default();
        return Redirect::permanent(format!("{}/{query}", link(path))).into_response();
    }
    // The preloader plays on the first home page of a visit.
    let intro =
        route.page == PageKind::Home && (!site.live || req.cookie("paquets-intro").is_none());
    let ctx = pages::Ctx {
        site,
        route,
        origin: site.origin(req),
        intro,
    };
    let response = pages::render(&ctx)
        .into_response()
        .header("cache-control", "no-cache");
    if intro && site.live {
        response.header("set-cookie", "paquets-intro=1; Path=/; SameSite=Lax")
    } else {
        response
    }
}

/// The "not found" page, in the language of the URL or of the browser.
fn not_found(site: &Site, req: &Request) -> Response {
    let lang = Route::parse(req.path()).map_or_else(
        || pick_lang(req.header("accept-language").unwrap_or("")),
        |r| r.lang,
    );
    let ctx = pages::Ctx {
        site,
        route: Route::new(lang, PageKind::NotFound),
        origin: site.origin(req),
        intro: false,
    };
    pages::not_found(&ctx).into_response()
}

/// `/search/{lang}.txt`: the search index of a language, one entry per line (see
/// [`crate::search::Entry`]). The browser downloads it once and searches locally.
fn search_index(site: &Site, req: &Request) -> Response {
    let file = req.param("file").unwrap_or("");
    let Some(lang) = file.strip_suffix(".txt").and_then(Lang::parse) else {
        return not_found(site, req);
    };
    let body: String = site
        .docs
        .search_index(lang)
        .iter()
        .map(|e| e.line())
        .collect();
    Response::new()
        .header("content-type", "text/plain; charset=utf-8")
        .header("cache-control", "public, max-age=300")
        .send(body)
}

fn sitemap(site: &Site, req: &Request) -> Response {
    let origin = site.origin(req);
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for route in Route::all() {
        xml.push_str(&format!("<url><loc>{origin}{}</loc>", route.path()));
        for lang in Lang::ALL {
            xml.push_str(&format!(
                r#"<xhtml:link rel="alternate" hreflang="{}" href="{origin}{}"/>"#,
                lang.code(),
                route.with_lang(lang).path()
            ));
        }
        xml.push_str(&format!(
            r#"<xhtml:link rel="alternate" hreflang="x-default" href="{origin}{}"/></url>"#,
            route.with_lang(Lang::En).path()
        ));
        xml.push('\n');
    }
    xml.push_str("</urlset>\n");
    Response::new()
        .header("content-type", "application/xml; charset=utf-8")
        .send(xml)
}

fn robots(site: &Site, req: &Request) -> Response {
    let origin = site.origin(req);
    Response::new()
        .header("content-type", "text/plain; charset=utf-8")
        .send(format!(
            "User-agent: *\nAllow: /\nDisallow: /demo\n\nSitemap: {origin}/sitemap.xml\n"
        ))
}

/// Routes a handler that only needs the site and the request.
fn route(app: &mut App, path: &str, site: &Arc<Site>, handler: fn(&Site, &Request) -> Response) {
    let site = site.clone();
    app.get(path, move |req: Request| {
        let site = site.clone();
        async move { handler(&site, &req) }
    });
}

/// The application, ready to run or to test.
pub fn app(site: Arc<Site>) -> App {
    let mut app = App::new();
    app.get("/", |req: Request| async move { root(&req) });
    route(&mut app, "/sitemap.xml", &site, sitemap);
    route(&mut app, "/robots.txt", &site, robots);
    route(&mut app, "/search/:file", &site, search_index);
    for path in ["/:lang", "/:lang/docs", "/:lang/docs/:slug"] {
        route(&mut app, path, &site, page);
    }
    demo::mount(&mut app);
    let fallback = site.clone();
    app.fallback(move |req: Request| {
        let site = fallback.clone();
        async move { not_found(&site, &req) }
    });
    app
}

/// `cargo run`: the server, on `PORT` (Heroku) or 3000.
pub fn run() -> io::Result<()> {
    routes::set_base(&std::env::var("BASE_PATH").unwrap_or_default());
    // Paquets serves `public/` and `pkg/` from the current directory.
    let dir = site_dir();
    if dir != Path::new(".") {
        std::env::set_current_dir(&dir)?;
    }
    let site = Arc::new(Site::load(true, std::env::var("SITE_URL").ok()));
    for lang in Lang::ALL {
        println!(
            "   docs: {}/{} pages in {}",
            site.docs.count(lang),
            crate::catalog::PAGES.len(),
            lang.code()
        );
    }
    app(site).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_change_with_the_content() {
        assert_eq!(version(b"a").len(), 10);
        assert_ne!(version(b"a"), version(b"b"));
    }
}
