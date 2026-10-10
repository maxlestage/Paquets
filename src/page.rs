//! Web pages: `active` views rendered on the server, with SEO tags, a default theme
//! and a Content-Security-Policy that allows exactly what the page contains.

use std::sync::OnceLock;

use active::{Document, Node, Seo};
use securite::headers::ContentSecurityPolicy;
use vitesse::{IntoResponse, Response, StatusCode, header};

/// The default theme: readable typography, forms, buttons, tables, cards, light and
/// dark modes. Remove it with [`Page::plain`].
pub const THEME: &str = include_str!("theme.css");

/// The browser bundle (`pkg/<name>.js` and `pkg/<name>_bg.wasm`), found by the
/// app at startup. Pages that contain islands load it.
static BUNDLE: OnceLock<(String, String)> = OnceLock::new();

pub(crate) fn set_bundle(js: String, wasm: String) {
    let _ = BUNDLE.set((js, wasm));
}

/// A complete HTML page, ready to be returned by a handler.
///
/// ```
/// use paquets::prelude::*;
///
/// async fn home(_req: Request) -> Page {
///     Page::new("Home")
///         .description("My site, in Rust.")
///         .body(main().child(h1().text("Hello 👋")).child(p().text("Built with Paquets.")))
/// }
///
/// let mut app = App::new();
/// app.get("/", home);
/// ```
///
/// The server sends finished HTML with every SEO tag (title, description, Open
/// Graph, Twitter, canonical, JSON-LD…: see [`Seo`]). Islands
/// ([`island`](active::island)) are made interactive by the browser bundle built
/// with `paquets build`.
///
/// Build the page at the end of the handler, after the last `.await`: views stay
/// on the thread that renders them.
#[must_use]
pub struct Page {
    title: String,
    seo: Seo,
    lang: String,
    body: Vec<Node>,
    styles: Vec<String>,
    stylesheets: Vec<String>,
    head: Vec<String>,
    theme: bool,
    status: StatusCode,
}

impl Page {
    /// A page titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Page {
            title: title.into(),
            seo: Seo::new(),
            lang: "en".into(),
            body: Vec::new(),
            styles: Vec::new(),
            stylesheets: Vec::new(),
            head: Vec::new(),
            theme: true,
            status: StatusCode::OK,
        }
    }

    /// `<meta name="description">`, also used for Open Graph and Twitter cards.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.seo = self.seo.description(description);
        self
    }

    /// The language of the page (`<html lang="…">`, `en` by default).
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Adds content to the `<body>`.
    pub fn body(mut self, content: impl Into<Node>) -> Self {
        self.body.push(content.into());
        self
    }

    /// More SEO tags (canonical URL, image, `noindex`, alternate languages,
    /// JSON-LD…), merged with the title and description.
    pub fn seo(mut self, seo: Seo) -> Self {
        self.seo = self.seo.merged(&seo);
        self
    }

    /// CSS written in the page (`<style>`), allowed by the page's CSP.
    pub fn style(mut self, css: impl Into<String>) -> Self {
        self.styles.push(css.into());
        self
    }

    /// A stylesheet (`<link rel="stylesheet">`), e.g. a file of `public/`.
    pub fn stylesheet(mut self, href: impl Into<String>) -> Self {
        self.stylesheets.push(href.into());
        self
    }

    /// Trusted HTML added to `<head>` (an icon, a font preload…).
    pub fn head(mut self, html: impl Into<String>) -> Self {
        self.head.push(html.into());
        self
    }

    /// Without the default theme: the page only has your own styles.
    pub fn plain(mut self) -> Self {
        self.theme = false;
        self
    }

    /// The HTTP status (`200` by default), e.g. `404` for a "not found" page.
    pub fn status(mut self, status: u16) -> Self {
        self.status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
        self
    }

    /// The HTML of the page.
    pub fn render(self) -> String {
        let seo = Seo::new().title(self.title).merged(&self.seo);
        let mut document = Document::new(self.body).lang(self.lang).seo(seo);
        if self.theme {
            document = document.head(format!("<style>{THEME}</style>"));
        }
        for css in self.styles {
            document = document.head(format!("<style>{}</style>", css.replace("</", "<\\/")));
        }
        for href in self.stylesheets {
            document = document.stylesheet(href);
        }
        for html in self.head {
            document = document.head(html);
        }
        let mut html = document.render();
        if let Some((js, wasm)) = BUNDLE.get() {
            if html.contains("<active-island") {
                if let Some(at) = html.find("</head>") {
                    html.insert_str(at, &loader(js, wasm));
                }
            }
        }
        html
    }
}

/// The tags `active` writes to start the browser bundle: the module, the binary
/// preloaded in parallel, and the call that starts it.
fn loader(js: &str, wasm: &str) -> String {
    format!(
        r#"<link rel="modulepreload" href="{js}"/><link rel="preload" href="{wasm}" as="fetch" type="application/wasm" crossorigin=""/><script type="module">import init from "{js}";init({{module_or_path:"{wasm}"}});</script>"#
    )
}

/// `default-src 'self'` and friends, plus a hash for each inline `<style>` and
/// `<script>` of this page: nothing else can run, even if some HTML slipped in.
pub(crate) fn content_security_policy(html: &str) -> String {
    let mut scripts = vec!["'self'".to_owned(), "'wasm-unsafe-eval'".to_owned()];
    scripts.extend(ContentSecurityPolicy::hashes_for_inline_scripts(html));
    let mut styles = vec!["'self'".to_owned()];
    styles.extend(ContentSecurityPolicy::hashes_for_inline_styles(html));
    ContentSecurityPolicy::strict()
        .script_src(scripts)
        .style_src(styles)
        .style_src_attr(["'unsafe-inline'"])
        .img_src(["'self'", "data:", "https:"])
        .remove_directive("upgrade-insecure-requests")
        .to_string()
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let status = self.status;
        let html = self.render();
        let csp = content_security_policy(&html);
        Response::new()
            .status(status)
            .header(header::CONTENT_SECURITY_POLICY, csp)
            .html(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use active::html::{h1, main};

    #[test]
    fn renders_a_themed_page_with_its_seo() {
        let html = Page::new("Home")
            .description("Hello")
            .lang("fr")
            .body(main().child(h1().text("Bonjour")))
            .render();
        assert!(html.starts_with(r#"<!DOCTYPE html><html lang="fr">"#));
        assert!(html.contains("<title>Home</title>"));
        assert!(html.contains(r#"content="Hello""#));
        assert!(html.contains("<main><h1>Bonjour</h1></main>"));
        assert!(html.contains(":root"));
    }

    #[test]
    fn the_policy_allows_the_inline_styles_of_the_page() {
        let html = Page::new("x").style("p{color:red}").render();
        let csp = content_security_policy(&html);
        let hashes = ContentSecurityPolicy::hashes_for_inline_styles(&html);
        assert_eq!(hashes.len(), 3, "active's style, the theme and ours");
        for hash in hashes {
            assert!(csp.contains(&hash));
        }
        assert!(csp.contains("script-src 'self' 'wasm-unsafe-eval'"));
        assert!(!csp.contains("unsafe-inline';") || csp.contains("style-src-attr 'unsafe-inline'"));
    }

    #[test]
    fn plain_pages_have_no_theme_and_custom_status() {
        let res = Page::new("Missing").plain().status(404).into_response();
        assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
        assert!(res.get_header("content-security-policy").is_some());
    }
}
