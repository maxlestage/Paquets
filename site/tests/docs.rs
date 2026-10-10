//! The documentation of the repository (`../docs/{lang}/{slug}.md`): every page of the
//! site's index exists in the three languages, has a title and sections, and every
//! link between pages leads to a page and an anchor that exist.
//!
//! These tests read the real Markdown files: they fail as long as a page is missing.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use paquets_site::catalog::{self, PAGES};
use paquets_site::i18n::Lang;
use paquets_site::server::markdown::{Rendered, render};

fn path(lang: Lang, slug: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../docs")
        .join(lang.code())
        .join(format!("{slug}.md"))
}

/// The rendered pages of a language that exist.
fn pages(lang: Lang) -> HashMap<&'static str, Rendered> {
    PAGES
        .iter()
        .filter_map(|page| {
            let markdown = std::fs::read_to_string(path(lang, page.slug)).ok()?;
            Some((page.slug, render(&markdown, lang)))
        })
        .collect()
}

fn ids(html: &str) -> HashSet<String> {
    html.split(" id=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .map(|id| id.replace("&amp;", "&"))
        .collect()
}

#[test]
fn every_page_exists_in_every_language() {
    let missing: Vec<String> = Lang::ALL
        .iter()
        .flat_map(|&lang| PAGES.iter().map(move |page| path(lang, page.slug)))
        .filter(|p| !p.is_file())
        .map(|p| p.display().to_string())
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
        let anchors: HashMap<&str, HashSet<String>> =
            pages.iter().map(|(slug, r)| (*slug, ids(&r.html))).collect();
        let prefix = format!("href=\"/{}/docs/", lang.code());
        for (slug, rendered) in &pages {
            let here = format!("{}/{slug}", lang.code());
            for link in rendered.html.split(prefix.as_str()).skip(1) {
                let target = link.split('"').next().unwrap_or_default();
                let (page, anchor) = target.split_once('#').unwrap_or((target, ""));
                let page = page.trim_end_matches('/');
                if page.is_empty() {
                    continue;
                }
                match anchors.get(page) {
                    None if catalog::find(page).is_some() => {
                        problems.push(format!("{here}: link to «{page}», not written yet"))
                    }
                    None => problems.push(format!("{here}: link to unknown page «{page}»")),
                    Some(ids) if !anchor.is_empty() && !ids.contains(anchor) => {
                        problems.push(format!("{here}: anchor «{page}#{anchor}» not found"))
                    }
                    Some(_) => {}
                }
            }
            // Anchors inside the page.
            let own = &anchors[slug];
            for link in rendered.html.split("href=\"#").skip(1) {
                let anchor = link.split('"').next().unwrap_or_default();
                if !own.contains(anchor) {
                    problems.push(format!("{here}: anchor «#{anchor}» not found"));
                }
            }
            // A link to `something.md` outside the index would leave the site for
            // GitHub: probably a typo.
            for link in rendered
                .html
                .split("href=\"https://github.com/maxlestage/Paquets/blob/master/docs/")
                .skip(1)
            {
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
            let (_, page) = catalog::find(slug).unwrap();
            if rendered.title != page.title(lang) {
                problems.push(format!(
                    "{}/{slug}: the page says «{}», the site's index «{}»",
                    lang.code(),
                    rendered.title,
                    page.title(lang)
                ));
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
