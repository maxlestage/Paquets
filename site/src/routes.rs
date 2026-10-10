//! The URLs of the site: `/{lang}/`, `/{lang}/docs/`, `/{lang}/docs/{slug}/`, behind an
//! optional base path (`/Paquets` when the site is published on GitHub Pages).

use std::sync::OnceLock;

use crate::catalog;
pub use crate::i18n::Lang;

static BASE: OnceLock<String> = OnceLock::new();

/// Sets the base path once, before anything is rendered: `""` (the default) or
/// `/Paquets`. The server takes it from `BASE_PATH`, the browser from `data-base`.
pub fn set_base(base: &str) {
    let _ = BASE.set(normalize_base(base));
}

/// `""`, or `/something` without a trailing slash.
pub fn normalize_base(base: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    match base {
        "" => String::new(),
        b if b.starts_with('/') => b.to_owned(),
        b => format!("/{b}"),
    }
}

/// The base path, without trailing slash (empty by default).
pub fn base() -> &'static str {
    BASE.get().map_or("", String::as_str)
}

/// A link to `path` (which starts with `/`), behind the base path.
pub fn link(path: &str) -> String {
    format!("{}{path}", base())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Docs,
    Doc(&'static str),
    NotFound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    pub lang: Lang,
    pub page: Page,
}

impl Route {
    pub fn new(lang: Lang, page: Page) -> Route {
        Route { lang, page }
    }

    /// The route of a path (without the base path); `None` when it does not start
    /// with a language.
    pub fn parse(path: &str) -> Option<Route> {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let lang = Lang::parse(parts.first()?)?;
        let page = match parts[1..] {
            [] => Page::Home,
            ["docs"] => Page::Docs,
            ["docs", slug] => match catalog::find(slug) {
                Some((_, page)) => Page::Doc(page.slug),
                None => Page::NotFound,
            },
            _ => Page::NotFound,
        };
        Some(Route { lang, page })
    }

    /// The path of the page, without the base path.
    pub fn path(&self) -> String {
        let l = self.lang.code();
        match self.page {
            Page::Home => format!("/{l}/"),
            Page::Docs => format!("/{l}/docs/"),
            Page::Doc(slug) => format!("/{l}/docs/{slug}/"),
            Page::NotFound => format!("/{l}/404/"),
        }
    }

    /// A link to the page, behind the base path.
    pub fn href(&self) -> String {
        link(&self.path())
    }

    /// The same page in another language.
    pub fn with_lang(&self, lang: Lang) -> Route {
        Route { lang, ..*self }
    }

    /// Every page of the site, in every language (what the static export writes).
    pub fn all() -> Vec<Route> {
        let mut routes = Vec::new();
        for lang in Lang::ALL {
            routes.push(Route::new(lang, Page::Home));
            routes.push(Route::new(lang, Page::Docs));
            routes.extend(
                catalog::PAGES
                    .iter()
                    .map(|p| Route::new(lang, Page::Doc(p.slug))),
            );
        }
        routes
    }
}

/// A link to a docs page of `lang`, with an optional anchor.
pub fn doc_href(lang: Lang, slug: &str, anchor: Option<&str>) -> String {
    let path = link(&format!("/{}/docs/{slug}/", lang.code()));
    match anchor {
        Some(anchor) if !anchor.is_empty() => format!("{path}#{anchor}"),
        _ => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_builds_urls() {
        let doc = Route::parse("/fr/docs/routing/").unwrap();
        assert_eq!(doc, Route::new(Lang::Fr, Page::Doc("routing")));
        assert_eq!(doc.path(), "/fr/docs/routing/");
        assert_eq!(doc.with_lang(Lang::Es).path(), "/es/docs/routing/");
        assert_eq!(Route::parse("/en").unwrap().page, Page::Home);
        assert_eq!(Route::parse("/en/").unwrap().page, Page::Home);
        assert_eq!(Route::parse("/es/docs").unwrap().page, Page::Docs);
        assert_eq!(
            Route::parse("/en/docs/nope/").unwrap().page,
            Page::NotFound
        );
        assert_eq!(Route::parse("/en/a/b/c").unwrap().page, Page::NotFound);
        assert_eq!(Route::parse("/de/"), None);
        assert_eq!(Route::parse("/"), None);
    }

    #[test]
    fn lists_every_page_once() {
        let all = Route::all();
        assert_eq!(all.len(), 3 * (2 + catalog::PAGES.len()));
        let mut paths: Vec<String> = all.iter().map(Route::path).collect();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), all.len());
    }

    #[test]
    fn normalizes_the_base_path() {
        assert_eq!(normalize_base(""), "");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base("Paquets/"), "/Paquets");
        assert_eq!(normalize_base(" /Paquets "), "/Paquets");
        // The base is not set in unit tests.
        assert_eq!(link("/en/"), "/en/");
        assert_eq!(
            doc_href(Lang::Es, "faq", Some("x")),
            "/es/docs/faq/#x"
        );
    }
}
