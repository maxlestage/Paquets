//! Turns a Markdown page of the documentation into HTML: headings with
//! anchors (GitHub style), links rewritten for the site, highlighted code
//! with a "copy" button, `> [!NOTE]` callouts and tables that scroll on
//! phones.

use std::collections::HashMap;

use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html,
};

use crate::data::GITHUB;
use crate::highlight::{self, Kind, Tokens};
use crate::i18n::Lang;

use super::index;

#[derive(Clone, PartialEq, Debug)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub text: String,
}

pub struct Rendered {
    pub html: String,
    pub title: String,
    pub toc: Vec<TocEntry>,
}

/// Heading ids, like GitHub: lowercase, punctuation removed, spaces
/// replaced by dashes, `-1`, `-2`… suffixes for duplicates. Anchors work the
/// same on the site and on GitHub.
#[derive(Default)]
pub struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let base: String = text
            .trim()
            .to_lowercase()
            .chars()
            .filter_map(|c| match c {
                ' ' => Some('-'),
                c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c),
                _ => None,
            })
            .collect();
        let count = self.seen.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slug
    }
}

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// The link of a page of the site, with an optional anchor.
pub fn page_href(slug: &str, anchor: Option<&str>) -> String {
    match anchor {
        Some(anchor) if !anchor.is_empty() => format!("#/docs/{slug}/{anchor}"),
        _ => format!("#/docs/{slug}"),
    }
}

/// Rewrites the destination of a link. Also returns `true` when the link
/// leaves the site (it then opens in a new tab).
pub fn rewrite_link(dest: &str, lang: Lang, slug: &str) -> (String, bool) {
    if dest.starts_with("http://") || dest.starts_with("https://") {
        return (dest.to_owned(), true);
    }
    if dest.starts_with("mailto:") {
        return (dest.to_owned(), false);
    }
    if let Some(anchor) = dest.strip_prefix('#') {
        return (page_href(slug, Some(anchor)), false);
    }
    let (path, anchor) = dest
        .split_once('#')
        .map_or((dest, None), |(p, a)| (p, Some(a)));
    // `routing.md`, `./routing.md` or `../fr/routing.md`: a page of the docs.
    let file = path.rsplit('/').next().unwrap_or(path);
    if let Some(page) = file.strip_suffix(".md") {
        let same_folder = !path.trim_start_matches("./").contains('/');
        let other_language = path.starts_with("../")
            && path.matches('/').count() == 2
            && Lang::from_code(&path[3..]).is_some();
        if same_folder || other_language {
            if page == "README" {
                return ("#/docs".to_owned(), false);
            }
            if index::find(page).is_some() {
                return (page_href(page, anchor), false);
            }
        }
    }
    // Another file of the repository: opened on GitHub.
    (
        format!("{GITHUB}/blob/master/docs/{}/{dest}", lang.code()),
        true,
    )
}

fn spans(tokens: Tokens) -> String {
    let mut out = String::new();
    for (kind, text) in tokens {
        if kind == Kind::Plain {
            out.push_str(&escape(&text));
        } else {
            out.push_str(&format!(
                "<span class=\"{}\">{}</span>",
                kind.class(),
                escape(&text)
            ));
        }
    }
    out
}

fn code_block(code: &str, language: &str, copy: &str, copied: &str) -> String {
    let code = code.trim_end_matches('\n');
    let language = language.to_ascii_lowercase();
    let body = match highlight::highlight(code, &language) {
        Some(tokens) => spans(tokens),
        None => escape(code),
    };
    let label = if language.is_empty() {
        "text".to_owned()
    } else {
        escape(&language)
    };
    format!(
        "<div class=\"doc-code\"><div class=\"doc-code__bar\"><span class=\"doc-code__lang\">{label}</span>\
         <button type=\"button\" class=\"doc-code__copy\" data-copied=\"{}\">{}</button></div>\
         <pre class=\"code\"><code>{body}</code></pre></div>",
        escape(copied),
        escape(copy),
    )
}

pub fn render(markdown: &str, lang: Lang, slug: &str) -> Rendered {
    let texts = lang.texts();
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM;
    let mut events = Parser::new_ext(markdown, options);
    let mut out: Vec<Event> = Vec::new();
    let mut slugger = Slugger::default();
    let mut toc = Vec::new();
    let mut title = String::new();
    // For each open quote: is it a callout?
    let mut quotes: Vec<bool> = Vec::new();
    let html_event = |s: String| Event::Html(CowStr::from(s));

    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Heading { level: l, .. }) => {
                let n = level(l);
                let mut inner = Vec::new();
                let mut text = String::new();
                for e in events.by_ref() {
                    match &e {
                        Event::End(TagEnd::Heading(_)) => break,
                        Event::Text(t) | Event::Code(t) => text.push_str(t),
                        _ => {}
                    }
                    inner.push(e);
                }
                if n == 1 {
                    if title.is_empty() {
                        title = text;
                    }
                    out.push(html_event("<h1 class=\"doc-title\">".into()));
                    out.extend(inner);
                    out.push(html_event("</h1>".into()));
                    continue;
                }
                let id = slugger.slug(&text);
                if n <= 3 {
                    toc.push(TocEntry {
                        level: n,
                        id: id.clone(),
                        text,
                    });
                }
                out.push(html_event(format!("<h{n} id=\"{}\">", escape(&id))));
                out.extend(inner);
                out.push(html_event(format!(
                    "<a class=\"doc-anchor\" href=\"{}\" aria-hidden=\"true\" tabindex=\"-1\">#</a></h{n}>",
                    escape(&page_href(slug, Some(&id)))
                )));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split([',', ' ']).next().unwrap_or_default().to_owned()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                let mut code = String::new();
                for e in events.by_ref() {
                    match e {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(t) => code.push_str(&t),
                        _ => {}
                    }
                }
                out.push(html_event(code_block(
                    &code,
                    &language,
                    texts.copy,
                    texts.copied,
                )));
            }
            Event::Start(Tag::Link {
                dest_url, title: t, ..
            }) => {
                let (href, external) = rewrite_link(&dest_url, lang, slug);
                let mut tag = format!("<a href=\"{}\"", escape(&href));
                if !t.is_empty() {
                    tag.push_str(&format!(" title=\"{}\"", escape(&t)));
                }
                if external {
                    tag.push_str(" target=\"_blank\" rel=\"noopener\"");
                }
                tag.push('>');
                out.push(Event::InlineHtml(CowStr::from(tag)));
            }
            Event::End(TagEnd::Link) => out.push(Event::InlineHtml(CowStr::from("</a>"))),
            Event::Start(Tag::Image {
                dest_url, title: t, ..
            }) => {
                // The alt text is the text inside the image tag.
                let mut alt = String::new();
                for e in events.by_ref() {
                    match e {
                        Event::End(TagEnd::Image) => break,
                        Event::Text(s) | Event::Code(s) => alt.push_str(&s),
                        _ => {}
                    }
                }
                let src = if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
                    dest_url.to_string()
                } else {
                    format!(
                        "{GITHUB}/raw/master/docs/{}/{}",
                        lang.code(),
                        dest_url.trim_start_matches("./")
                    )
                };
                out.push(Event::InlineHtml(CowStr::from(format!(
                    "<img src=\"{}\" alt=\"{}\" title=\"{}\" loading=\"lazy\">",
                    escape(&src),
                    escape(&alt),
                    escape(&t)
                ))));
            }
            Event::Start(Tag::BlockQuote(Some(kind))) => {
                let (class, i) = match kind {
                    BlockQuoteKind::Note => ("note", 0),
                    BlockQuoteKind::Tip => ("tip", 1),
                    BlockQuoteKind::Important => ("important", 2),
                    BlockQuoteKind::Warning => ("warning", 3),
                    BlockQuoteKind::Caution => ("caution", 4),
                };
                quotes.push(true);
                out.push(html_event(format!(
                    "<div class=\"callout callout--{class}\"><p class=\"callout__title\">{}</p>",
                    texts.callouts[i]
                )));
            }
            Event::Start(Tag::BlockQuote(None)) => {
                quotes.push(false);
                out.push(event);
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if quotes.pop() == Some(true) {
                    out.push(html_event("</div>".into()));
                } else {
                    out.push(event);
                }
            }
            Event::Start(Tag::Table(_)) => {
                out.push(html_event("<div class=\"table-wrap\">".into()));
                out.push(event);
            }
            Event::End(TagEnd::Table) => {
                out.push(event);
                out.push(html_event("</div>".into()));
            }
            // No raw HTML in the docs: it is shown as text.
            Event::Html(raw) | Event::InlineHtml(raw) => out.push(Event::Text(raw)),
            other => out.push(other),
        }
    }

    let mut html = String::new();
    html::push_html(&mut html, out.into_iter());
    Rendered { html, title, toc }
}

/// The plain text of a line of Markdown, for the search.
pub fn plain_line(line: &str) -> String {
    let mut s = line.trim();
    s = s.trim_start_matches('#').trim_start();
    s = s.trim_start_matches('>').trim_start();
    for marker in [
        "[!NOTE]",
        "[!TIP]",
        "[!IMPORTANT]",
        "[!WARNING]",
        "[!CAUTION]",
    ] {
        s = s.trim_start_matches(marker);
    }
    if let Some(rest) = s.strip_prefix("- ").or_else(|| s.strip_prefix("* ")) {
        s = rest;
    }
    // [text](link) → text; `, ** and _ are removed.
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' | '*' => {}
            ']' if chars.peek() == Some(&'(') => {
                for c in chars.by_ref() {
                    if c == ')' {
                        break;
                    }
                }
            }
            '[' => {}
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_like_github() {
        let mut s = Slugger::default();
        assert_eq!(s.slug("Paramètres de route"), "paramètres-de-route");
        assert_eq!(s.slug("L'API REST en une ligne"), "lapi-rest-en-une-ligne");
        assert_eq!(s.slug("`app.get()` et `app.post()`"), "appget-et-apppost");
        assert_eq!(s.slug("Paramètres de route"), "paramètres-de-route-1");
        assert_eq!(s.slug("¿Qué es Paquets?"), "qué-es-paquets");
        assert_eq!(s.slug("Deploy — step 1"), "deploy--step-1");
    }

    #[test]
    fn rewrites_links() {
        let l = Lang::Fr;
        assert_eq!(rewrite_link("routing.md", l, "intro").0, "#/docs/routing");
        assert_eq!(
            rewrite_link("routing.md#x", l, "intro").0,
            "#/docs/routing/x"
        );
        assert_eq!(
            rewrite_link("./database.md#transactions", l, "intro").0,
            "#/docs/database/transactions"
        );
        assert_eq!(
            rewrite_link("#plus-loin", l, "intro").0,
            "#/docs/intro/plus-loin"
        );
        assert_eq!(rewrite_link("../en/routing.md", l, "x").0, "#/docs/routing");
        assert_eq!(rewrite_link("README.md", l, "x").0, "#/docs");
        let (href, external) = rewrite_link("../../examples/todo.rs", l, "x");
        assert!(external);
        assert_eq!(
            href,
            "https://github.com/maxlestage/Paquets/blob/master/docs/fr/../../examples/todo.rs"
        );
        // An unknown page stays a link to GitHub rather than a dead link.
        assert!(rewrite_link("nope.md", l, "x").1);
        assert_eq!(
            rewrite_link("https://docs.rs/paquets", l, "x"),
            ("https://docs.rs/paquets".to_owned(), true)
        );
    }

    #[test]
    fn renders_headings_code_and_callouts() {
        let md = "# Titre\n\nIntro avec [un lien](routing.md#x).\n\n## Une section\n\n```rust\nlet x = 1;\n```\n\n```toml\n[dependencies]\npaquets = \"0.1\"\n```\n\n```bash\n$ paquets dev\n```\n\n> [!TIP]\n> Astuce !\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
        let r = render(md, Lang::Fr, "intro");
        assert_eq!(r.title, "Titre");
        assert_eq!(r.toc.len(), 1);
        assert_eq!(r.toc[0].id, "une-section");
        assert!(r.html.contains("<h2 id=\"une-section\">"));
        assert!(r.html.contains("href=\"#/docs/routing/x\""));
        assert!(r.html.contains("<span class=\"t-kw\">let</span>"));
        assert!(
            r.html
                .contains("<span class=\"t-type\">[dependencies]</span>")
        );
        assert!(r.html.contains("<span class=\"t-fn\">paquets</span>"));
        assert!(r.html.contains("data-copied=\"Copié !\">Copier</button>"));
        assert!(r.html.contains("callout--tip"));
        assert!(r.html.contains("Astuce"));
        assert!(r.html.contains("<div class=\"table-wrap\"><table>"));
    }

    #[test]
    fn keeps_escaped_pipes_in_tables() {
        let r = render(
            "| a | b |\n|---|---|\n| `\\|req\\| async {}` | x |\n",
            Lang::En,
            "x",
        );
        assert!(r.html.contains("<code>|req| async {}</code>"), "{}", r.html);
    }

    #[test]
    fn escapes_raw_html() {
        let r = render("Hello <script>alert(1)</script>", Lang::En, "x");
        assert!(!r.html.contains("<script>"));
        let r = render("```html\n<b>x</b>\n```", Lang::En, "x");
        assert!(r.html.contains("&lt;b&gt;"));
    }

    #[test]
    fn renders_images_from_github() {
        let r = render("![The button](https://www.herokucdn.com/deploy/button.svg)", Lang::En, "x");
        assert!(r.html.contains("<img src=\"https://www.herokucdn.com/deploy/button.svg\" alt=\"The button\""));
        let r = render("![Logo](../../site/assets/logo.svg)", Lang::Es, "x");
        assert!(r.html.contains("src=\"https://github.com/maxlestage/Paquets/raw/master/docs/es/../../site/assets/logo.svg\""));
    }

    #[test]
    fn plain_lines_for_search() {
        assert_eq!(
            plain_line("> [!NOTE] See [routing](routing.md) and `app.get`"),
            "See routing and app.get"
        );
        assert_eq!(plain_line("## A **title**"), "A title");
    }
}
