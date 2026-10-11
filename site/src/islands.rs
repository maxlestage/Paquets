//! The interactive parts of the site. Each island renders on the server like any view
//! and, compiled to WebAssembly, hydrates in the browser from the same code.

use std::rc::Rc;

use paquets::prelude::*;

use crate::data::SAMPLES;
use crate::i18n::Lang;
use crate::icons::icon;
use crate::routes::doc_href;

fn lang(props: &str) -> Lang {
    Lang::parse(props).unwrap_or(Lang::En)
}

/// Every island, registered under the name used by [`island`].
pub const ISLANDS: [(&str, paquets::active::IslandFn); 3] = [
    ("search", search),
    ("showcase", showcase),
    ("terminal", terminal),
];

// ----- Docs search -------------------------------------------------------------------

/// A button that opens a dialog; results come from the search index of the language,
/// downloaded on first use and filtered locally on every key press.
pub fn search(props: &str) -> Node {
    let lang = lang(props);
    let t = lang.texts();
    let results = use_state(Vec::<(String, String, String)>::new());
    let active = use_state(0usize);
    let loaded = use_state(false);
    let typed = use_state(String::new());

    let go = move || {
        if let Some(href) = results.with(|r| r.get(active.get()).map(|r| r.0.clone())) {
            #[cfg(target_arch = "wasm32")]
            crate::client::search::navigate(&href);
            let _ = href;
        }
    };

    fragment([
        button()
            .class("search-btn")
            .attr("type", "button")
            .attr("aria-label", t.docs_search)
            .attr("data-search-button", "")
            .on_click(move |_e| {
                #[cfg(target_arch = "wasm32")]
                crate::client::search::open(lang, results, loaded);
            })
            .on_mount(move |_button| {
                #[cfg(target_arch = "wasm32")]
                crate::client::search::shortcut(_button);
            })
            .child(icon("search"))
            .child(span().class("search-btn__label").text(t.search))
            .child(kbd().text("/")),
        dialog()
            .class("search")
            .attr("aria-label", t.docs_search)
            .on_click(|_e| {
                #[cfg(target_arch = "wasm32")]
                crate::client::search::close_on_backdrop(&_e);
            })
            .child(
                div()
                    .class("search-box")
                    .child(
                        label()
                            .class("search-field")
                            .child(icon("search"))
                            .child(
                                input()
                                    .attr("type", "search")
                                    .attr("placeholder", t.docs_search)
                                    .attr("aria-label", t.docs_search)
                                    .attr("autocomplete", "off")
                                    .attr("spellcheck", "false")
                                    .on_input(move |e| {
                                        active.set(0);
                                        typed.set(e.value());
                                        #[cfg(target_arch = "wasm32")]
                                        crate::client::search::query(
                                            lang,
                                            e.value(),
                                            results,
                                            loaded,
                                        );
                                    })
                                    .on("keydown", move |e| match e.key().as_str() {
                                        "ArrowDown" => {
                                            e.prevent_default();
                                            let n = results.with(Vec::len);
                                            active
                                                .update(|a| *a = (*a + 1).min(n.saturating_sub(1)));
                                        }
                                        "ArrowUp" => {
                                            e.prevent_default();
                                            active.update(|a| *a = a.saturating_sub(1));
                                        }
                                        "Enter" => {
                                            e.prevent_default();
                                            go();
                                        }
                                        _ => {}
                                    }),
                            )
                            .child(kbd().text("Esc")),
                    )
                    .child(ul().attr("role", "listbox").children_dyn(move || {
                        let list = results.get();
                        if list.is_empty() && loaded.get() {
                            let text = t.docs_no_results.replace("{}", typed.get().trim());
                            return vec![li().class("empty").text(text).into()];
                        }
                        list.into_iter()
                            .enumerate()
                            .map(|(i, (href, title, sub))| {
                                li().attr("role", "option")
                                    .attr_dyn("aria-selected", move || {
                                        (active.get() == i).to_string()
                                    })
                                    .child(
                                        a().href(href)
                                            .on("mouseenter", move |_| active.set(i))
                                            .child(strong().text(title))
                                            .child(span().text(sub)),
                                    )
                                    .into()
                            })
                            .collect()
                    }))
                    .child(
                        p().class("search-keys")
                            .child(kbd().text("↑"))
                            .child(kbd().text("↓"))
                            .child(kbd().text("↵"))
                            .child(span().text(t.docs_search_hint)),
                    ),
            ),
    ])
}

// ----- Code showcase -----------------------------------------------------------------

/// The docs page that goes further than each sample.
pub const SAMPLE_DOCS: [&str; 5] = ["quick-start", "rest-api", "pages", "islands", "testing"];

/// The highlighted code of a sample, one `<span class="ln">` per line. The server
/// renders it; in the browser the panels keep the server HTML (raw HTML is never
/// touched by hydration).
fn sample_html(index: usize, lang: Lang) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let code = crate::data::localize(SAMPLES[index].code, lang.texts().samples[index].comments);
        crate::highlight::lines_html(&crate::highlight::tokenize(&code))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (index, lang);
        String::new()
    }
}

/// Code samples with tabs; each panel types itself line by line when it appears.
pub fn showcase(props: &str) -> Node {
    let lang = lang(props);
    let t = lang.texts();
    let tab = use_state(0usize);
    let count = SAMPLES.len();
    div()
        .class("showcase__wrap")
        .child(
            div()
                .class("tabs")
                .attr("role", "tablist")
                .attr("data-reveal", "")
                .attr_dyn("style", move || {
                    format!("--tab:{};--tabs:{count}", tab.get())
                })
                .child(span().class("tabs__indicator").attr("aria-hidden", "true"))
                .children(t.samples.iter().enumerate().map(|(i, sample)| {
                    button()
                        .class("tabs__tab")
                        .class_if("is-active", move || tab.get() == i)
                        .attr("type", "button")
                        .attr("role", "tab")
                        .attr_dyn("aria-selected", move || (tab.get() == i).to_string())
                        .on_click(move |_| tab.set(i))
                        .text(sample.label)
                })),
        )
        .child(
            div()
                .class("showcase__grid")
                .attr("data-reveal", "")
                .style("--d:120ms")
                .child(
                    div()
                        .class("showcase__asides")
                        .children(t.samples.iter().enumerate().map(|(i, sample)| {
                            div()
                                .class("showcase__aside")
                                .bool_attr("hidden", move || tab.get() != i)
                                .child(
                                    span()
                                        .class("showcase__num")
                                        .text(format!("0{}", i + 1))
                                        .child(small().text(format!(" / 0{count}"))),
                                )
                                .child(h3().text(sample.label))
                                .child(p().text(sample.caption))
                                .child(
                                    a().class("link-arrow")
                                        .href(doc_href(lang, SAMPLE_DOCS[i], None))
                                        .child(span().text(t.docs_browse))
                                        .child(icon("arrow")),
                                )
                        })),
                )
                .child(
                    div()
                        .class("window window--glow")
                        .attr("data-copy-root", "")
                        .child(
                            div()
                                .class("window__bar")
                                .child(span())
                                .child(span())
                                .child(span())
                                .child(em().text_dyn(move || SAMPLES[tab.get()].file.to_string()))
                                .child(
                                    button()
                                        .class("copy")
                                        .attr("type", "button")
                                        .attr("aria-label", t.copy_aria)
                                        .attr("data-copied", t.copied)
                                        .child(icon("copy"))
                                        .child(span().text(t.copy)),
                                ),
                        )
                        .children((0..count).map(|i| {
                            div()
                                .class("panel")
                                .attr("role", "tabpanel")
                                .bool_attr("hidden", move || tab.get() != i)
                                .child(pre().class("code code--lines").child(
                                    code().attr("data-lang", "rust").html(sample_html(i, lang)),
                                ))
                        })),
                ),
        )
        .into()
}

// ----- Typing terminal --------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineKind {
    /// Typed character by character after the prompt.
    Command,
    /// Compiler output, dimmed.
    Dim,
    /// A result, highlighted.
    Done,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub text: String,
    /// The step of the journey it belongs to (`None` before the first one).
    pub step: Option<usize>,
}

/// The whole journey, from `cargo install --git https://github.com/maxlestage/Paquets paquets` to production.
pub fn script(out: &[&str; 4]) -> Vec<Line> {
    let line = |kind, text: &str, step| Line {
        kind,
        text: text.to_owned(),
        step,
    };
    vec![
        line(
            LineKind::Command,
            "cargo install --git https://github.com/maxlestage/Paquets paquets",
            None,
        ),
        line(LineKind::Dim, "   Compiling paquets v0.1.0", None),
        line(LineKind::Done, &format!("✓ {}", out[0]), None),
        line(LineKind::Command, "paquets new my-app", Some(0)),
        line(LineKind::Done, &format!("📦 {}", out[1]), Some(0)),
        line(LineKind::Command, "cd my-app && paquets dev", Some(1)),
        line(LineKind::Dim, "   Compiling my-app v0.1.0", Some(1)),
        line(LineKind::Done, &format!("➜ {}", out[2]), Some(1)),
        line(LineKind::Command, "git push origin my-feature", Some(2)),
        line(LineKind::Dim, "   CI: fmt · clippy · test · build", Some(2)),
        line(LineKind::Done, &format!("✓ {}", out[3]), Some(3)),
    ]
}

/// Ticks (of about 34 ms) spent on a line before the next one starts.
pub fn duration(line: &Line) -> usize {
    match line.kind {
        LineKind::Command => line.text.chars().count() + 14,
        LineKind::Dim => 9,
        LineKind::Done => 16,
    }
}

pub fn total(lines: &[Line]) -> usize {
    lines.iter().map(duration).sum()
}

/// What is visible at `tick`: the number of complete lines, and the number of
/// characters typed on the line after them.
pub fn visible(lines: &[Line], tick: usize) -> (usize, usize) {
    let mut left = tick;
    for (i, line) in lines.iter().enumerate() {
        let d = duration(line);
        if left < d {
            let typed = match line.kind {
                LineKind::Command => left.min(line.text.chars().count()),
                _ => 0,
            };
            return (i, typed);
        }
        left -= d;
    }
    (lines.len(), 0)
}

/// The step of the journey the terminal is at.
pub fn step_at(lines: &[Line], tick: usize) -> Option<usize> {
    let (done, _) = visible(lines, tick);
    lines.get(done.min(lines.len().saturating_sub(1)))?.step
}

fn prompt() -> Element {
    span()
        .class("term__prompt")
        .text("➜ ")
        .child(span().class("term__dir").text("my-app"))
        .text(" $ ")
}

/// A terminal that types the journey when it scrolls into view. The server renders
/// the finished session (readable without JavaScript); the browser replays it.
pub fn terminal(props: &str) -> Node {
    let t = lang(props).texts();
    let lines = Rc::new(script(&t.term_out));
    let total = total(&lines);
    let count = lines.len();
    let tick = use_state(total);
    let seen: Rc<dyn Fn() -> (usize, usize)> = {
        let lines = lines.clone();
        Rc::new(move || visible(&lines, tick.get()))
    };
    let label = lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let rows = lines.iter().enumerate().map(|(i, line)| {
        let shown = seen.clone();
        let row = span()
            .class("term__line")
            .bool_attr("hidden", move || shown().0 < i);
        match line.kind {
            LineKind::Command => {
                let (typing, caret) = (seen.clone(), seen.clone());
                let text = line.text.clone();
                row.child(prompt())
                    .child(span().class("term__cmd").text_dyn(move || {
                        let (done, typed) = typing();
                        if done > i {
                            text.clone()
                        } else {
                            text.chars().take(typed).collect()
                        }
                    }))
                    .child(
                        span()
                            .class("caret")
                            .attr("aria-hidden", "true")
                            .bool_attr("hidden", move || caret().0 != i),
                    )
            }
            LineKind::Dim => row.class("term__line--dim").text(line.text.clone()),
            LineKind::Done => {
                let waiting = seen.clone();
                row.class("term__line--done")
                    .class_if("is-waiting", move || waiting().0 == i)
                    .text(line.text.clone())
            }
        }
    });
    let last = seen.clone();
    let steps = lines.clone();
    div()
        .class("window terminal")
        .on_mount(move |_el| {
            #[cfg(target_arch = "wasm32")]
            crate::client::terminal::start(_el, tick, steps.clone());
            let _ = &steps;
        })
        .child(
            div()
                .class("window__bar")
                .child(span())
                .child(span())
                .child(span())
                .child(em().child(icon("terminal")).text("zsh — my-app"))
                .child(
                    button()
                        .class("term__replay")
                        .attr("type", "button")
                        .class_if("is-visible", move || tick.get() >= total)
                        .on_click(move |_| {
                            #[cfg(target_arch = "wasm32")]
                            crate::client::terminal::replay(tick, total);
                        })
                        .child(icon("replay"))
                        .child(span().text(t.term_replay)),
                ),
        )
        .child(
            pre()
                .class("code term")
                .attr("aria-label", label)
                .children(rows)
                .child(
                    span()
                        .class("term__line")
                        .bool_attr("hidden", move || last().0 < count)
                        .child(prompt())
                        .child(span().class("caret").attr("aria-hidden", "true")),
                ),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_the_script_in_order() {
        let lines = script(&["a", "b", "c", "d"]);
        assert_eq!(visible(&lines, 0), (0, 0));
        assert_eq!(visible(&lines, 5), (0, 5));
        assert_eq!(visible(&lines, duration(&lines[0])), (1, 0));
        assert_eq!(visible(&lines, total(&lines)), (lines.len(), 0));
        // Every step of the journey appears, in order.
        let steps: Vec<usize> = lines.iter().filter_map(|l| l.step).collect();
        assert!(steps.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(step_at(&lines, 0), None);
        assert_eq!(step_at(&lines, total(&lines)), Some(3));
    }

    #[test]
    fn islands_render_on_the_server() {
        for (name, island_fn) in ISLANDS {
            let html = paquets::active::island(name, "fr", island_fn).render();
            assert!(
                html.starts_with(&format!(
                    r#"<active-island data-island="{name}" data-props="fr">"#
                )),
                "{name}"
            );
        }
        let html = paquets::active::island("showcase", "en", showcase).render();
        assert!(html.contains("<span class=\"t-kw\">fn</span>"), "{html}");
        assert_eq!(html.matches("role=\"tabpanel\"").count(), SAMPLES.len());
        assert_eq!(html.matches(" hidden").count(), 2 * (SAMPLES.len() - 1));
        let html = paquets::active::island("terminal", "es", terminal).render();
        // The finished session, readable without JavaScript.
        assert!(html.contains("paquets new my-app"));
        assert!(html.contains("Escuchando en http://localhost:3000"));
    }
}
