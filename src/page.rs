//! Web pages: `active` views rendered on the server, with SEO tags, a default theme
//! and a Content-Security-Policy that allows exactly what the page contains.

use std::sync::OnceLock;

use active::{Document, Node, Prefetch, Seo};
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
    speed: Speed,
}

/// How the page makes the next one faster (`view_transitions`, `prefetch`, `prerender`).
#[derive(Clone, Copy, Default)]
struct Speed {
    transitions: bool,
    prefetch: Option<Prefetch>,
    prerender: Option<Prefetch>,
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
            speed: Speed::default(),
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

    /// Animates the change to the next page of the site, like an app: the browser fades
    /// from one page to the other, and elements with the same `view-transition-name` move
    /// to their new place (the View Transitions API, as in Astro and Next). Nothing
    /// happens in browsers without it, nor for visitors who prefer reduced motion.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// let html = Page::new("Home").view_transitions().render();
    /// assert!(html.contains("@view-transition{navigation:auto}"));
    /// ```
    pub fn view_transitions(mut self) -> Self {
        self.speed.transitions = true;
        self
    }

    /// Loads the pages the visitor is about to open before they click, so that they show
    /// at once (`Prefetch::Hover`: when the pointer rests on a link). Same-site links
    /// only; links with `rel="nofollow"` or `download`, or inside `data-no-prefetch`, are
    /// left alone.
    ///
    /// ```
    /// use paquets::prelude::*;
    ///
    /// let html = Page::new("Home").prefetch(Prefetch::Hover).render();
    /// assert!(html.contains(r#"<script type="speculationrules">"#));
    /// ```
    pub fn prefetch(mut self, when: Prefetch) -> Self {
        self.speed.prefetch = Some(when);
        self
    }

    /// Like [`prefetch`](Self::prefetch), with the next page also run and painted in the
    /// background where the browser can (Chrome, Edge): it appears instantly. Keep it for
    /// pages without side effects when they load.
    pub fn prerender(mut self, when: Prefetch) -> Self {
        self.speed.prerender = Some(when);
        self
    }

    /// The HTML of the page.
    pub fn render(self) -> String {
        self.render_with_policy().0
    }

    /// The HTML and its Content-Security-Policy: `default-src 'self'` and friends,
    /// plus the hash of each inline `<style>` and `<script>` the page writes itself
    /// (the theme, [`style`](Self::style), [`head`](Self::head), the islands loader).
    /// Nothing else inline can run, even if some HTML slipped into the body.
    fn render_with_policy(self) -> (String, String) {
        let seo = Seo::new().title(self.title).merged(&self.seo);
        let lang = self.lang;
        let mut document = Document::new(self.body).lang(lang.clone()).seo(seo);
        // `active` always writes this style for islands.
        let mut styles = vec![cached(&ACTIVE_STYLE, || style_hash(ACTIVE_STYLE_CSS))];
        let mut scripts = Vec::new();
        let mut fonts = Vec::new();
        if self.theme {
            styles.push(cached(&THEME_HASH, || style_hash(THEME)));
            document = document.head(format!("<style>{THEME}</style>"));
        }
        for css in self.styles {
            let css = css.replace("</", "<\\/");
            styles.push(style_hash(&css));
            document = document.head(format!("<style>{css}</style>"));
        }
        for href in self.stylesheets {
            if let Some(origin) = crate::shield::origin_of(&href) {
                if origin == "https://fonts.googleapis.com" {
                    fonts.push("https://fonts.gstatic.com".to_owned());
                }
                styles.push(origin);
            }
            document = document.stylesheet(href);
        }
        for html in self.head {
            styles.extend(ContentSecurityPolicy::hashes_for_inline_styles(&html));
            scripts.extend(ContentSecurityPolicy::hashes_for_inline_scripts(&html));
            document = document.head(html);
        }
        let speed = self.speed;
        if speed.transitions {
            document = document.view_transitions();
            allow(
                &mut styles,
                &mut scripts,
                "transitions".into(),
                Document::view_transitions,
            );
        }
        if let Some(when) = speed.prefetch {
            document = document.prefetch(when);
            allow(
                &mut styles,
                &mut scripts,
                format!("prefetch {when:?}"),
                |d| d.prefetch(when),
            );
        }
        if let Some(when) = speed.prerender {
            document = document.prerender(when);
            allow(
                &mut styles,
                &mut scripts,
                format!("prerender {when:?}"),
                |d| d.prerender(when),
            );
        }
        if let Some(manifest) = crate::pwa::manifest() {
            document = document
                .manifest(&crate::pwa::manifest_url(&lang), manifest)
                .service_worker(crate::pwa::WORKER_URL);
            scripts.push(cached(&REGISTER_HASH, || {
                let script = active::pwa::register_script(crate::pwa::WORKER_URL);
                ContentSecurityPolicy::hashes_for_inline_scripts(&format!(
                    "<script>{script}</script>"
                ))
                .pop()
                .unwrap_or_default()
            }));
        }
        if crate::ops::live_reload() {
            let (script, hash) = crate::ops::live_reload_script();
            document = document.head(format!("<script>{script}</script>"));
            scripts.push(hash.clone());
        }
        let mut html = document.render();
        if let Some((tags, hash)) = loader()
            && html.contains("<active-island")
            && let Some(at) = html.find("</head>")
        {
            html.insert_str(at, tags);
            scripts.push(hash.clone());
        }
        let policy = format!(
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'{}; style-src 'self'{}; \
             style-src-attr 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'{}; \
             connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; \
             frame-ancestors 'none'",
            sources(&scripts),
            sources(&styles),
            sources(&fonts),
        );
        (html, policy)
    }
}

const ACTIVE_STYLE_CSS: &str = "active-island{display:contents}";
static ACTIVE_STYLE: OnceLock<String> = OnceLock::new();
static THEME_HASH: OnceLock<String> = OnceLock::new();
static REGISTER_HASH: OnceLock<String> = OnceLock::new();
static LOADER: OnceLock<(String, String)> = OnceLock::new();

/// The hashes of the inline styles and scripts `active` writes for an option of
/// [`Document`], by option: found once, by rendering an empty page with it.
type Hashes = (Vec<String>, Vec<String>);
static OPTION_HASHES: std::sync::Mutex<Vec<(String, Hashes)>> = std::sync::Mutex::new(Vec::new());

/// Adds to the page's policy what `option` writes inline, so that it runs and nothing
/// else does.
fn allow(
    styles: &mut Vec<String>,
    scripts: &mut Vec<String>,
    key: String,
    option: impl FnOnce(Document) -> Document,
) {
    let (style_hashes, script_hashes) = {
        let mut known = OPTION_HASHES.lock().unwrap_or_else(|e| e.into_inner());
        match known.iter().find(|(k, _)| *k == key) {
            Some((_, hashes)) => hashes.clone(),
            None => {
                let html = option(Document::new(Node::from(""))).render();
                let hashes = (
                    ContentSecurityPolicy::hashes_for_inline_styles(&html),
                    ContentSecurityPolicy::hashes_for_inline_scripts(&html),
                );
                known.push((key, hashes.clone()));
                hashes
            }
        }
    };
    for (list, hashes) in [(styles, style_hashes), (scripts, script_hashes)] {
        for hash in hashes {
            if !list.contains(&hash) {
                list.push(hash);
            }
        }
    }
}

fn cached(cell: &'static OnceLock<String>, compute: impl FnOnce() -> String) -> String {
    cell.get_or_init(compute).clone()
}

/// `'sha256-…'` of a `<style>` content.
fn style_hash(css: &str) -> String {
    ContentSecurityPolicy::hashes_for_inline_styles(&format!("<style>{css}</style>"))
        .pop()
        .unwrap_or_default()
}

fn sources(list: &[String]) -> String {
    list.iter().fold(String::new(), |mut out, source| {
        out.push(' ');
        out.push_str(source);
        out
    })
}

/// The tags `active` writes to start the browser bundle (the module, the binary
/// preloaded in parallel, the call that starts it), and the hash of that call.
fn loader() -> Option<&'static (String, String)> {
    let (js, wasm) = BUNDLE.get()?;
    Some(LOADER.get_or_init(|| {
        let script = format!(r#"import init from "{js}";init({{module_or_path:"{wasm}"}});"#);
        let tags = format!(
            r#"<link rel="modulepreload" href="{js}"/><link rel="preload" href="{wasm}" as="fetch" type="application/wasm" crossorigin=""/><script type="module">{script}</script>"#
        );
        let hash =
            ContentSecurityPolicy::hashes_for_inline_scripts(&format!("<script>{script}</script>"))
                .pop()
                .unwrap_or_default();
        (tags, hash)
    }))
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let status = self.status;
        let (html, policy) = self.render_with_policy();
        Response::new()
            .status(status)
            .header(header::CONTENT_SECURITY_POLICY, policy)
            .html(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use active::html::{h1, main, p};

    #[test]
    fn faster_navigations_are_allowed_by_the_policy_and_nothing_else() {
        let (html, policy) = Page::new("Home")
            .view_transitions()
            .prefetch(Prefetch::Hover)
            .render_with_policy();
        assert!(html.contains("@view-transition{navigation:auto}"), "{html}");
        assert!(
            html.contains(r#"<script type="speculationrules">"#),
            "{html}"
        );
        // Each inline style and script the page writes has its hash in the policy.
        for hash in ContentSecurityPolicy::hashes_for_inline_styles(&html)
            .into_iter()
            .chain(ContentSecurityPolicy::hashes_for_inline_scripts(&html))
        {
            assert!(policy.contains(&hash), "{hash} missing from {policy}");
        }
        // Without them, the page has none of it.
        let (plain, plain_policy) = Page::new("Home").render_with_policy();
        assert!(!plain.contains("speculationrules") && !plain.contains("view-transition"));
        assert!(plain_policy.len() < policy.len());
    }

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
    fn the_policy_allows_the_inline_code_of_the_page_and_nothing_else() {
        let (html, csp) = Page::new("x")
            .style("p{color:red}")
            .head("<script>console.log(1)</script>")
            .stylesheet("https://fonts.googleapis.com/css2?family=Inter")
            .body(p().html("<script>alert('injected')</script>"))
            .render_with_policy();
        let styles = ContentSecurityPolicy::hashes_for_inline_styles(&html);
        assert_eq!(styles.len(), 3, "active's style, the theme and ours");
        for hash in styles {
            assert!(csp.contains(&hash), "{hash} missing in {csp}");
        }
        let scripts = ContentSecurityPolicy::hashes_for_inline_scripts(&html);
        assert_eq!(scripts.len(), 2);
        assert!(csp.contains(&scripts[0]), "the head script is allowed");
        assert!(!csp.contains(&scripts[1]), "the injected script is not");
        assert!(csp.contains("script-src 'self' 'wasm-unsafe-eval' 'sha256-"));
        assert!(csp.contains("https://fonts.googleapis.com"));
        assert!(csp.contains("font-src 'self' https://fonts.gstatic.com"));
        assert!(csp.contains("style-src-attr 'unsafe-inline'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn plain_pages_have_no_theme_and_custom_status() {
        let res = Page::new("Missing").plain().status(404).into_response();
        assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
        assert!(res.get_header("content-security-policy").is_some());
    }
}
