//! Hash routing (`#/docs/<slug>/<anchor>`), which works on GitHub Pages
//! without any server configuration. Other hashes (`#packages`…) are
//! sections of the home page.

use crate::docs::index;
use crate::dom;

#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Home,
    DocsHome,
    Doc {
        slug: &'static str,
        anchor: Option<String>,
    },
}

impl Route {
    pub fn is_docs(&self) -> bool {
        !matches!(self, Route::Home)
    }
}

/// Decodes `%20`-style escapes (anchors with accents arrive encoded).
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(byte) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_owned())
}

pub fn parse(hash: &str) -> Route {
    let hash = hash.trim_start_matches('#');
    let Some(rest) = hash.strip_prefix("/docs") else {
        return Route::Home;
    };
    if !(rest.is_empty() || rest.starts_with('/')) {
        return Route::Home;
    }
    let mut parts = rest.trim_matches('/').splitn(2, '/');
    let slug = parts.next().unwrap_or_default();
    let anchor = parts
        .next()
        .map(|a| a.trim_matches('/'))
        .filter(|a| !a.is_empty())
        .map(decode);
    match index::find(slug) {
        Some((_, page)) => Route::Doc {
            slug: page.slug,
            anchor,
        },
        None => Route::DocsHome,
    }
}

pub fn current() -> Route {
    parse(&dom::window().location().hash().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(slug: &'static str, anchor: Option<&str>) -> Route {
        Route::Doc {
            slug,
            anchor: anchor.map(str::to_owned),
        }
    }

    #[test]
    fn parses_home_hashes() {
        assert_eq!(parse(""), Route::Home);
        assert_eq!(parse("#"), Route::Home);
        assert_eq!(parse("#top"), Route::Home);
        assert_eq!(parse("#packages"), Route::Home);
        assert_eq!(parse("#/docsx"), Route::Home);
    }

    #[test]
    fn parses_docs_hashes() {
        assert_eq!(parse("#/docs"), Route::DocsHome);
        assert_eq!(parse("#/docs/"), Route::DocsHome);
        assert_eq!(parse("#/docs/nope"), Route::DocsHome);
        assert_eq!(parse("#/docs/routing"), doc("routing", None));
        assert_eq!(parse("#/docs/routing/"), doc("routing", None));
        assert_eq!(parse("#/docs/quick-start/x"), doc("quick-start", Some("x")));
        assert_eq!(
            parse("#/docs/deploy-heroku-mobile/step-2/"),
            doc("deploy-heroku-mobile", Some("step-2"))
        );
        assert!(parse("#/docs/faq").is_docs());
        assert!(!parse("#features").is_docs());
    }

    #[test]
    fn decodes_anchors() {
        assert_eq!(
            parse("#/docs/routing/param%C3%A8tres-de-route"),
            doc("routing", Some("paramètres-de-route"))
        );
        assert_eq!(decode("100%"), "100%");
        assert_eq!(decode("a%2Fb"), "a/b");
    }
}
