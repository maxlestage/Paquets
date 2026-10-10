//! The documentation: index, loading and rendering of the Markdown pages,
//! search, and the screens that show them.

pub mod fetch;
pub mod index;
pub mod render;
pub mod search;
mod view;

pub use view::{DocsHome, DocsPage};

#[cfg(test)]
mod tests {
    //! Checks the documentation of the repository (`../docs/<lang>/`): every
    //! page of the index exists in the three languages, and every internal
    //! link leads to a page and an anchor that exist.
    //!
    //! These tests read the real Markdown files, so they fail as long as a
    //! page is missing.

    use std::collections::{HashMap, HashSet};

    use super::index::PAGES;
    use super::render::{Rendered, render};
    use crate::i18n::Lang;

    fn ids(html: &str) -> HashSet<String> {
        html.split(" id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .map(|id| id.replace("&amp;", "&"))
            .collect()
    }

    fn path(lang: Lang, slug: &str) -> String {
        format!("{}/../docs/{}/{slug}.md", env!("CARGO_MANIFEST_DIR"), lang.code())
    }

    /// The rendered pages of a language that exist.
    fn pages(lang: Lang) -> HashMap<&'static str, Rendered> {
        PAGES
            .iter()
            .filter_map(|page| {
                let markdown = std::fs::read_to_string(path(lang, page.slug)).ok()?;
                Some((page.slug, render(&markdown, lang, page.slug)))
            })
            .collect()
    }

    #[test]
    fn every_page_exists_in_every_language() {
        let missing: Vec<String> = Lang::ALL
            .iter()
            .flat_map(|&lang| PAGES.iter().map(move |page| path(lang, page.slug)))
            .filter(|p| !std::path::Path::new(p).is_file())
            .collect();
        assert!(
            missing.is_empty(),
            "{} missing page(s):\n{}",
            missing.len(),
            missing.join("\n")
        );
    }

    #[test]
    fn every_page_has_a_title_and_sections() {
        let mut problems = Vec::new();
        for lang in Lang::ALL {
            for (slug, page) in pages(lang) {
                if page.title.is_empty() {
                    problems.push(format!("{}/{slug}: no `# Title`", lang.code()));
                }
                if page.toc.is_empty() {
                    problems.push(format!("{}/{slug}: no `##` section", lang.code()));
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    #[test]
    fn every_link_between_pages_resolves() {
        let mut problems = Vec::new();
        for lang in Lang::ALL {
            let pages = pages(lang);
            let anchors: HashMap<&str, HashSet<String>> = pages
                .iter()
                .map(|(slug, r)| (*slug, ids(&r.html)))
                .collect();
            for (slug, rendered) in &pages {
                let here = format!("{}/{slug}", lang.code());
                for link in rendered.html.split("href=\"#/docs").skip(1) {
                    let target = link.split('"').next().unwrap_or_default();
                    let mut parts = target.trim_start_matches('/').splitn(2, '/');
                    let target_slug = parts.next().unwrap_or_default();
                    if target_slug.is_empty() {
                        continue;
                    }
                    match anchors.get(target_slug) {
                        None => problems.push(format!("{here}: link to missing page «{target_slug}»")),
                        Some(ids) => {
                            if let Some(anchor) = parts.next()
                                && !ids.contains(anchor)
                            {
                                problems.push(format!(
                                    "{here}: anchor «{target_slug}#{anchor}» not found"
                                ));
                            }
                        }
                    }
                }
                // A link to `something.md` that is not a page of the index
                // would leave the site for GitHub: probably a typo.
                for link in rendered.html.split("href=\"https://github.com/maxlestage/Paquets/blob/master/docs/").skip(1) {
                    let target = link.split('"').next().unwrap_or_default();
                    let file = target.split('#').next().unwrap_or_default();
                    if file.ends_with(".md") && !file.contains("../") {
                        problems.push(format!("{here}: link to unknown page «{target}»"));
                    }
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }

    #[test]
    fn index_titles_match_the_pages() {
        let mut problems = Vec::new();
        for lang in Lang::ALL {
            for (slug, rendered) in pages(lang) {
                let Some((_, page)) = super::index::find(slug) else {
                    continue;
                };
                if rendered.title != page.title(lang) {
                    problems.push(format!(
                        "{}/{slug}: the page says «{}», the index «{}»",
                        lang.code(),
                        rendered.title,
                        page.title(lang)
                    ));
                }
            }
        }
        assert!(problems.is_empty(), "\n{}", problems.join("\n"));
    }
}
