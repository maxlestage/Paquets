//! The documentation: the Markdown pages of `docs/{lang}/{slug}.md`, rendered once at
//! startup, and the search index of each language.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::markdown::{TocEntry, render};
use crate::catalog::{self, CATEGORIES, PAGES};
use crate::i18n::Lang;
use crate::routes::doc_href;
use crate::search::{Entry, norm};

/// A rendered page.
pub struct Doc {
    pub title: String,
    pub description: String,
    pub html: String,
    pub toc: Vec<TocEntry>,
    /// The page does not exist in this language: this is the English one.
    pub fallback: bool,
}

pub struct Docs {
    pages: HashMap<(Lang, &'static str), Doc>,
}

/// Where the Markdown pages are: `DOCS_DIR`, else `../docs` next to this crate (a
/// clone of the repository), else `docs/` in the current directory (Docker).
pub fn docs_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("DOCS_DIR") {
        return PathBuf::from(dir);
    }
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs");
    if repo.is_dir() {
        return repo;
    }
    PathBuf::from("docs")
}

impl Docs {
    /// Reads `dir/{lang}/{slug}.md` for every page of the catalog. A page missing in a
    /// language is shown in English; a page missing everywhere is left out (its URL
    /// still answers, with a notice).
    pub fn load(dir: &Path) -> Docs {
        let read = |lang: Lang, slug: &str| {
            std::fs::read_to_string(dir.join(lang.code()).join(format!("{slug}.md"))).ok()
        };
        let mut pages = HashMap::new();
        for page in &PAGES {
            let english = read(Lang::En, page.slug);
            for lang in Lang::ALL {
                let (source, fallback) = match read(lang, page.slug) {
                    Some(source) => (Some(source), false),
                    None => (english.clone(), true),
                };
                let Some(source) = source else {
                    continue;
                };
                let r = render(&source, lang);
                pages.insert(
                    (lang, page.slug),
                    Doc {
                        title: if r.title.is_empty() {
                            page.title(lang).to_owned()
                        } else {
                            r.title
                        },
                        description: r.description,
                        html: r.html,
                        toc: r.toc,
                        fallback: fallback && lang != Lang::En,
                    },
                );
            }
        }
        Docs { pages }
    }

    pub fn get(&self, lang: Lang, slug: &str) -> Option<&Doc> {
        let (_, page) = catalog::find(slug)?;
        self.pages.get(&(lang, page.slug))
    }

    /// The number of pages written in `lang` (not counting English fallbacks).
    pub fn count(&self, lang: Lang) -> usize {
        self.pages
            .iter()
            .filter(|((l, _), doc)| *l == lang && !doc.fallback)
            .count()
    }

    /// The search index of `lang`: every page, then its headings, in reading order.
    pub fn search_index(&self, lang: Lang) -> Vec<Entry> {
        let mut entries = Vec::new();
        for page in &PAGES {
            let category = lang.pick(CATEGORIES[page.category].title);
            let href = doc_href(lang, page.slug, None);
            let (title, description) = match self.get(lang, page.slug) {
                Some(doc) => (doc.title.clone(), doc.description.clone()),
                None => (page.title(lang).to_owned(), String::new()),
            };
            entries.push(Entry {
                page: true,
                href: href.clone(),
                title: title.clone(),
                context: category.to_owned(),
                words: norm(&format!(
                    "{title} {} {} {description} {category}",
                    page.title(lang),
                    page.summary(lang)
                )),
                title_words: norm(&title),
            });
            for h in self.get(lang, page.slug).map_or(&[][..], |d| &d.toc[..]) {
                entries.push(Entry {
                    page: false,
                    href: format!("{href}#{}", h.id),
                    title: h.text.clone(),
                    context: title.clone(),
                    words: norm(&format!("{} {title}", h.text)),
                    title_words: norm(&h.text),
                });
            }
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_english_and_indexes_headings() {
        let dir = std::env::temp_dir().join(format!("paquets-site-docs-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("en")).unwrap();
        std::fs::write(
            dir.join("en/routing.md"),
            "# Routing\n\nRoutes and parameters.\n\n## Parameters\n\nText.\n",
        )
        .unwrap();
        let docs = Docs::load(&dir);
        let en = docs.get(Lang::En, "routing").unwrap();
        assert_eq!(en.title, "Routing");
        assert!(!en.fallback);
        let fr = docs.get(Lang::Fr, "routing").unwrap();
        assert!(fr.fallback, "French falls back to English");
        assert!(docs.get(Lang::En, "faq").is_none());
        assert_eq!(docs.count(Lang::En), 1);
        assert_eq!(docs.count(Lang::Fr), 0);

        let index = docs.search_index(Lang::Fr);
        assert_eq!(index.iter().filter(|e| e.page).count(), PAGES.len());
        let heading = index.iter().find(|e| !e.page).unwrap();
        assert_eq!(heading.href, "/fr/docs/routing/#parameters");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
