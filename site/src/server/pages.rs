//! Every page of the site, written with active (through Paquets' prelude).

use paquets::prelude::*;

use super::Site;
use super::docs::Doc;
use crate::catalog::{self, CATEGORIES, PAGES, pages_of};
use crate::data::{
    AUTHOR, CRATES_IO, DOCS_RS, ECOSYSTEM, FEATURES, INSTALL, JOURNEY, LICENSE_PATH, MARQUEE_CODE,
    NUMBERS, PACKAGES,
};
use crate::i18n::{Lang, Texts, number};
use crate::icons::{icon, logo};
use crate::routes::{Page as PageKind, Route, base, doc_href, link};

const FONTS: &str = "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&family=Syne:wght@600;700;800&display=swap";

/// What a request needs to render a page.
pub struct Ctx<'a> {
    pub site: &'a Site,
    pub route: Route,
    /// Absolute origin of the site (with the base path), for canonical and Open
    /// Graph URLs, e.g. `https://maxlestage.github.io/Paquets`.
    pub origin: String,
    /// Play the preloader (the home page, once per visit).
    pub intro: bool,
}

impl Ctx<'_> {
    fn t(&self) -> &'static Texts {
        self.route.lang.texts()
    }

    fn lang(&self) -> Lang {
        self.route.lang
    }

    /// A section of the home page of the current language.
    fn home(&self, anchor: &str) -> String {
        let home = Route::new(self.lang(), PageKind::Home).href();
        if anchor.is_empty() {
            home
        } else {
            format!("{home}#{anchor}")
        }
    }
}

// ----- Small pieces -------------------------------------------------------------------

/// A link attracted by the cursor (`data-magnetic`, moved by the browser bundle).
fn magnetic(href: &str, class: &str, external: bool) -> Element {
    let link = a()
        .class("magnetic btn")
        .class(class.to_owned())
        .href(href.to_owned())
        .attr("data-magnetic", "");
    if external {
        link.attr("target", "_blank").attr("rel", "noopener")
    } else {
        link
    }
}

fn inner(children: impl IntoIterator<Item = Node>) -> Element {
    span().class("magnetic__inner").children(children)
}

/// A text whose hover rolls a copy of itself up (`data-text`, CSS).
fn roll(text: &str) -> Element {
    span()
        .class("roll")
        .attr("data-text", text.to_owned())
        .child(span().text(text.to_owned()))
}

/// Each letter rises in turn (CSS); `offset` chains several lines.
fn split(text: &str, offset: usize) -> Element {
    let words: Vec<&str> = text.split(' ').collect();
    let mut root = span()
        .class("split")
        .attr("aria-label", text.to_owned())
        .attr("role", "text");
    let mut start = offset;
    for (w, word) in words.iter().enumerate() {
        let letters = (start..).zip(word.chars()).map(|(i, c)| {
            span()
                .class("ch")
                .attr("aria-hidden", "true")
                .style(format!("--i:{i}"))
                .text(c.to_string())
        });
        root = root.child(
            span()
                .class("word")
                .attr("aria-hidden", "true")
                .children(letters),
        );
        // One more for the space.
        start += word.chars().count() + 1;
        if w + 1 < words.len() {
            root = root.text(" ");
        }
    }
    root
}

/// A cube with six faces (CSS 3D). `size` 0 leaves the size to the CSS.
fn cube(size: u32, color: Option<&str>, glyph: Option<&str>, spin: bool, wire: bool) -> Element {
    let mut style = String::new();
    if size > 0 {
        style.push_str(&format!("--s:{size}px;"));
    }
    if let Some(color) = color {
        style.push_str(&format!("--c:{color};"));
    }
    let mut root = div().class("cube").attr("aria-hidden", "true");
    if color.is_none() {
        root = root.class("cube--brand");
    }
    if wire {
        root = root.class("cube--wire");
    }
    if !style.is_empty() {
        root = root.style(style);
    }
    let body = div().class(if spin {
        "cube__body cube__body--spin"
    } else {
        "cube__body"
    });
    root.child(body.children(
        ["front", "back", "right", "left", "top", "bottom"].map(|face| {
            let face = span().class(format!("cube__face cube__face--{face}"));
            match glyph {
                Some(g) => face.child(b().text(g.to_owned())),
                None => face,
            }
        }),
    ))
}

/// The Paquets box: five glass faces and a lid hinged on its back edge; `content`
/// shares its 3D space.
fn pbox(class: &str, content: Vec<Node>) -> Element {
    div()
        .class("pbox")
        .class(class.to_owned())
        .attr("aria-hidden", "true")
        .child(
            span()
                .class("pbox__face pbox__face--front")
                .child(span().class("pbox__tape"))
                .child(span().class("pbox__mark").child(logo())),
        )
        .child(span().class("pbox__face pbox__face--back"))
        .child(
            span()
                .class("pbox__face pbox__face--right")
                .child(span().class("pbox__tape")),
        )
        .child(span().class("pbox__face pbox__face--left"))
        .child(span().class("pbox__face pbox__face--bottom"))
        .child(
            span()
                .class("pbox__lid")
                .child(span().class("pbox__tape pbox__tape--lid")),
        )
        .children(content)
}

/// The heading of a section: number, label, title and lead.
fn head(num: &str, label: &str, title: &str, lead: Option<&str>) -> Element {
    let mut root = div()
        .class("head")
        .child(
            p().class("eyebrow")
                .attr("data-reveal", "")
                .child(span().class("eyebrow__num").text(num.to_owned()))
                .text(label.to_owned()),
        )
        .child(
            h2().class("head__title")
                .attr("data-reveal", "")
                .style("--d:80ms")
                .text(title.to_owned()),
        );
    if let Some(lead) = lead {
        root = root.child(
            p().class("head__lead")
                .attr("data-reveal", "")
                .style("--d:160ms")
                .text(lead.to_owned()),
        );
    }
    root
}

/// `$ cargo install paquets` with a copy button.
fn install(ctx: &Ctx<'_>, class: &str, delay: &str) -> Element {
    let t = ctx.t();
    div()
        .class("install")
        .class(class.to_owned())
        .style(format!("--d:{delay}"))
        .child(span().class("install__prompt").text("$"))
        .child(code().text(INSTALL))
        .child(
            button()
                .class("copy")
                .attr("type", "button")
                .attr("data-copy", INSTALL)
                .attr("data-copied", t.copied)
                .attr("aria-label", t.copy_aria)
                .attr("data-cursor", t.copy)
                .child(icon("copy"))
                .child(span().text(t.copy)),
        )
}

/// The language selector: links to the same page in each language.
fn lang_switch(ctx: &Ctx<'_>) -> Element {
    let index = Lang::ALL.iter().position(|&l| l == ctx.lang()).unwrap_or(0);
    div()
        .class("lang")
        .attr("role", "group")
        .attr("aria-label", ctx.t().lang_aria)
        .style(format!("--idx:{index}"))
        .child(icon("globe"))
        .child(span().class("lang__pill").attr("aria-hidden", "true"))
        .children(Lang::ALL.map(|l| {
            let route = if ctx.route.page == PageKind::NotFound {
                Route::new(l, PageKind::Home)
            } else {
                ctx.route.with_lang(l)
            };
            let link = a()
                .class("lang__btn")
                .href(route.href())
                .attr("hreflang", l.code())
                .attr("lang", l.code())
                .attr("title", l.name())
                .text(l.code().to_uppercase());
            if l == ctx.lang() {
                link.class("is-active").attr("aria-current", "true")
            } else {
                link
            }
        }))
}

// ----- Layout ---------------------------------------------------------------------------

/// The page around `content`: head tags, navigation, footer, cursor, preloader.
fn layout(ctx: &Ctx<'_>, title: &str, description: &str, content: Element) -> Page {
    let lang = ctx.lang();
    let route = ctx.route;
    let mut seo = Seo::new()
        .site_name("Paquets")
        .base_url(ctx.origin.clone())
        .image("/assets/og-image.png")
        .og_type("website")
        .locale(lang.locale())
        .meta_name("author", AUTHOR);
    if route.page == PageKind::NotFound {
        seo = seo.noindex(true);
    } else {
        seo = seo.canonical(route.path());
        for l in Lang::ALL {
            seo = seo.alternate(l.code(), route.with_lang(l).path());
        }
        seo = seo.alternate("x-default", route.with_lang(Lang::En).path());
    }
    let icon_href = link("/assets/logo.svg");
    let head = format!(
        r##"<meta name="color-scheme" content="dark"/><link rel="icon" href="{icon_href}" type="image/svg+xml"/><link rel="preconnect" href="https://fonts.googleapis.com"/><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>"##
    );
    let mut root = div()
        .class("site")
        .attr("data-base", base())
        .child(a().class("skip").href("#main").text(ctx.t().skip))
        .child(div().class("progress").attr("aria-hidden", "true"));
    if ctx.intro {
        root = root.class("site--intro").child(loader(ctx));
    }
    root = root
        .child(cursor())
        .child(navbar(ctx))
        .child(menu(ctx))
        .child(content.id("main"))
        .child(site_footer(ctx))
        .child(div().class("noise").attr("aria-hidden", "true"));
    Page::new(title)
        .lang(lang.code())
        .description(description)
        .seo(seo)
        .plain()
        .stylesheet(FONTS)
        .stylesheet(format!("{}?v={}", link("/site.css"), ctx.site.css_version))
        .head(head)
        .body(root)
}

fn loader(ctx: &Ctx<'_>) -> Element {
    div()
        .class("loader")
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("loader__inner")
                .child(
                    div()
                        .class("loader__stage")
                        .child(div().class("loader__rig").child(pbox(
                            "pbox--loader",
                            vec![span().class("loader__beam").into()],
                        )))
                        .child(div().class("loader__shadow")),
                )
                .child(
                    div().class("loader__word").children(
                        "PAQUETS"
                            .chars()
                            .enumerate()
                            .map(|(i, c)| span().style(format!("--i:{i}")).text(c.to_string())),
                    ),
                )
                .child(div().class("loader__bar").child(span()))
                .child(
                    div().class("loader__label").text(ctx.t().loader).child(
                        span()
                            .class("loader__dots")
                            .child(i().text("."))
                            .child(i().text("."))
                            .child(i().text(".")),
                    ),
                ),
        )
        .child(div().class("loader__percent"))
        .child(div().class("loader__curtain"))
}

fn cursor() -> Element {
    div()
        .class("cursor")
        .attr("aria-hidden", "true")
        .child(div().class("cursor__dot"))
        .child(div().class("cursor__ring").child(span()))
}

const SECTIONS: [&str; 4] = ["packages", "code", "features", "deploy"];

fn navbar(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let docs = matches!(ctx.route.page, PageKind::Docs | PageKind::Doc(_));
    let mut header =
        header()
            .class("nav")
            .id("nav")
            .child(
                a().class("nav__brand")
                    .href(ctx.home(""))
                    .attr("aria-label", t.brand_aria)
                    .child(logo())
                    .child(span().text("Paquets")),
            )
            .child(
                nav()
                    .class("nav__links")
                    .attr("aria-label", t.nav_aria)
                    .children(SECTIONS.iter().zip(t.nav_links).map(|(id, text)| {
                        a().class("nav__link").href(ctx.home(id)).child(roll(text))
                    }))
                    .child({
                        let link = a()
                            .class("nav__link nav__link--docs")
                            .href(Route::new(ctx.lang(), PageKind::Docs).href())
                            .child(icon("book"))
                            .child(roll(t.nav_docs));
                        if docs { link.class("is-active") } else { link }
                    }),
            )
            .child(island("search", ctx.lang().code(), crate::islands::search))
            .child(lang_switch(ctx))
            .child(
                button()
                    .class("nav__burger")
                    .attr("type", "button")
                    .attr("data-toggle-menu", "")
                    .attr("aria-label", t.menu)
                    .attr("aria-expanded", "false")
                    .child(span())
                    .child(span()),
            );
    if docs {
        header = header.class("nav--docs");
    }
    header
}

fn menu(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let mut links: Vec<(String, &str, bool)> = SECTIONS
        .iter()
        .zip(t.nav_links)
        .map(|(id, text)| (ctx.home(id), text, false))
        .collect();
    links.push((
        Route::new(ctx.lang(), PageKind::Docs).href(),
        t.docs_title,
        false,
    ));
    div()
        .class("menu")
        .child(div().class("menu__glow").attr("aria-hidden", "true"))
        .child(
            nav()
                .attr("aria-label", t.nav_aria)
                .children(
                    links
                        .into_iter()
                        .enumerate()
                        .map(|(i, (href, text, external))| {
                            let link = a()
                                .href(href)
                                .style(format!("--i:{i}"))
                                .child(span().class("menu__num").text(format!("0{}", i + 1)))
                                .text(text);
                            if external {
                                link.attr("target", "_blank").attr("rel", "noopener")
                            } else {
                                link
                            }
                        }),
                ),
        )
        .child(
            div()
                .class("menu__foot")
                .child(p().text(t.footer_tagline))
                .child(lang_switch(ctx)),
        )
}

/// The signature footer, shared in spirit by every site of the ecosystem: the call to
/// action and the install command, the link columns, the sister projects, the author's
/// signature, the giant wordmark and the legal line.
fn site_footer(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let lang = ctx.lang();
    let external = |link: Element| link.attr("target", "_blank").attr("rel", "noopener");
    // (text, link, external)
    let product: Vec<(&str, String, bool)> = ["packages", "code", "features", "numbers", "deploy"]
        .iter()
        .zip(t.footer_product)
        .map(|(id, text)| (text, ctx.home(id), false))
        .collect();
    let docs: Vec<(&str, String, bool)> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(c, cat)| {
            let first = pages_of(c).next().map_or("introduction", |p| p.slug);
            (lang.pick(cat.title), doc_href(lang, first, None), false)
        })
        .collect();
    let packages: Vec<(&str, String, bool)> = PACKAGES
        .iter()
        .map(|p| (p.name, p.site.to_owned(), true))
        .collect();
    let mut resources: Vec<(&str, String, bool)> = vec![
        (t.footer_resources[0], CRATES_IO.to_owned(), true),
        (t.footer_resources[1], DOCS_RS.to_owned(), true),
        (
            t.footer_resources[2],
            doc_href(lang, "deploy-heroku-mobile", None),
            false,
        ),
        (t.footer_license, link(LICENSE_PATH), true),
    ];
    if ctx.site.live {
        resources.push((t.demo, "/demo".to_owned(), false));
    }
    let columns = [
        (t.footer_cols[0], product),
        (t.footer_cols[1], docs),
        (t.footer_cols[2], packages),
        (t.footer_cols[3], resources),
    ];
    let totop = svg()
        .class("totop__text")
        .attr("viewBox", "0 0 100 100")
        .attr("aria-hidden", "true")
        .child(
            defs().child(
                path()
                    .id("totop-circle")
                    .attr("d", "M50 50 m-38 0 a38 38 0 1 1 76 0 a38 38 0 1 1 -76 0"),
            ),
        )
        .child(
            text_svg().child(
                el("textPath")
                    .attr("href", "#totop-circle")
                    .attr("textLength", "236")
                    .attr("lengthAdjust", "spacing")
                    .text(format!("{} ", t.totop)),
            ),
        );
    let cta = div()
        .class("footer__cta")
        .attr("data-reveal", "")
        .child(
            a().class("footer__cta-link")
                .href(doc_href(lang, "quick-start", None))
                .child(span().text(t.footer_start))
                .child(Node::from(" "))
                .child(em().text("Paquets"))
                .child(icon("arrow")),
        )
        .child(
            install(ctx, "footer__install", "0ms")
                .attr("aria-label", format!("{}: {INSTALL}", t.footer_install)),
        );
    let brand = div()
        .class("footer__brand")
        .attr("data-reveal", "")
        .child(
            a().class("footer__logo")
                .href(ctx.home(""))
                .attr("aria-label", t.brand_aria)
                .child(logo())
                .child(span().text("Paquets")),
        )
        .child(p().class("footer__tagline").text(t.footer_tagline))
        .child(
            ul().class("footer__badges")
                .attr("aria-label", "Paquets")
                .child(li().class("hot").text(format!("v{}", paquets::VERSION)))
                .child(li().text(t.footer_rust))
                .child(li().child(external(a().href(CRATES_IO).text("crates.io")))),
        )
        .child(
            div()
                .class("footer__lang")
                .child(span().text(t.footer_lang))
                .child(lang_switch(ctx)),
        );
    let cols = div()
        .class("footer__cols")
        .children(columns.into_iter().enumerate().map(|(c, (title, links))| {
            div()
                .class("footer__col")
                .attr("data-reveal", "")
                .style(format!("--d:{}ms", c * 80))
                .child(h2().text(title))
                .child(
                    ul().children(links.into_iter().map(|(text, href, opens_tab)| {
                        let link = a()
                            .class("footer__link")
                            .href(href)
                            .child(span().text(text))
                            .child(icon("arrow"));
                        li().child(if opens_tab { external(link) } else { link })
                    })),
                )
        }));
    let ecosystem = nav()
        .class("footer__eco")
        .attr("aria-label", t.footer_ecosystem)
        .attr("data-reveal", "")
        .child(
            div()
                .class("footer__eco-head")
                .child(h2().text(t.footer_ecosystem))
                .child(p().text(t.footer_ecosystem_text)),
        )
        .child(
            ul().class("footer__eco-grid")
                .children(
                    ECOSYSTEM
                        .iter()
                        .zip(t.footer_projects)
                        .map(|((name, url), text)| {
                            let here = *name == "Paquets";
                            let mut title = span().class("eco__name").text(*name);
                            if here {
                                title = title.child(span().class("eco__here").text(t.footer_here));
                            }
                            let link = a()
                                .class(if here { "eco eco--here" } else { "eco" })
                                .href(*url)
                                .child(title)
                                .child(span().class("eco__text").text(text))
                                .child(icon("arrow"));
                            li().child(if here {
                                link.attr("aria-current", "page")
                            } else {
                                external(link)
                            })
                        }),
                ),
        );
    let credits = div()
        .class("footer__credits")
        .attr("data-reveal", "")
        .child(p().class("footer__by").text(t.footer_by))
        .child(
            p().class("footer__name").children(
                AUTHOR
                    .split(' ')
                    .enumerate()
                    .map(|(i, word)| span().style(format!("--i:{i}")).text(word.to_owned())),
            ),
        )
        .child(p().class("footer__crafted").text(t.footer_crafted));
    footer()
        .class("footer")
        .child(div().class("footer__glow").attr("aria-hidden", "true"))
        .child(cta)
        .child(div().class("footer__top").child(brand).child(cols))
        .child(ecosystem)
        .child(credits)
        .child(
            div()
                .class("footer__word")
                .attr("aria-hidden", "true")
                .attr("data-reveal", "")
                .children(
                    "PAQUETS"
                        .chars()
                        .enumerate()
                        .map(|(i, c)| span().style(format!("--i:{i}")).text(c.to_string())),
                ),
        )
        .child(
            div()
                .class("footer__bottom")
                .child(p().class("footer__copy").text(t.footer_rights))
                .child(external(
                    a().class("footer__license")
                        .href(link(LICENSE_PATH))
                        .text(t.footer_proprietary),
                ))
                .child(
                    p().class("footer__made")
                        .child(span().class("footer__heart").text("📦"))
                        .text(t.footer_made),
                )
                .child(
                    a().class("totop")
                        .href("#main")
                        .attr("data-cursor", t.totop_cursor)
                        .attr("aria-label", t.totop_aria)
                        .child(totop)
                        .child(icon("up")),
                ),
        )
}

// ----- Home ----------------------------------------------------------------------------

pub fn home(ctx: &Ctx<'_>) -> Page {
    let t = ctx.t();
    let content = main()
        .class("home")
        .child(hero(ctx))
        .child(marquee(ctx))
        .child(packages(ctx))
        .child(showcase(ctx))
        .child(features(ctx))
        .child(numbers(ctx))
        .child(journey(ctx))
        .child(deploy(ctx))
        .child(docs_preview(ctx))
        .child(cta(ctx));
    layout(ctx, t.meta_title, t.hero_lead, content)
}

fn hero(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let mut offset = 0;
    let lines = t.hero_title.iter().enumerate().map(|(i, line)| {
        let words = split(line, offset);
        offset += line.chars().count() + 1;
        let row = span().class("hero__line");
        if i == t.hero_accent {
            row.class("hero__line--accent")
                .child(span().class("grad-text").child(words))
        } else {
            row.child(words)
        }
    });
    let blobs = ["a", "b", "c", "d"]
        .iter()
        .zip(["0.14", "-0.08", "0.06", "0.2"])
        .map(|(name, speed)| {
            span()
                .class(format!("aurora__blob aurora__blob--{name}"))
                .attr("data-speed", speed)
        });
    // (left %, top %, size px, parallax speed, wireframe)
    const FLOATERS: [(u32, u32, u32, &str, bool); 7] = [
        (6, 22, 34, "0.18", true),
        (88, 16, 26, "-0.12", false),
        (14, 78, 22, "0.08", false),
        (78, 82, 40, "0.22", true),
        (46, 8, 18, "-0.2", true),
        (94, 52, 16, "0.3", false),
        (36, 92, 28, "-0.1", true),
    ];
    let floaters = FLOATERS
        .iter()
        .enumerate()
        .map(|(i, (x, y, size, speed, wire))| {
            div()
                .class("floater")
                .attr("data-speed", *speed)
                .style(format!("left:{x}%;top:{y}%;--i:{i}"))
                .child(cube(*size, None, None, true, *wire))
        });
    section()
        .class("hero")
        .id("top")
        .child(
            div()
                .class("hero__bg")
                .attr("aria-hidden", "true")
                .child(div().class("aurora").children(blobs))
                .child(div().class("hero__grid"))
                .child(div().class("floaters").children(floaters))
                .child(div().class("hero__fade")),
        )
        .child(
            div()
                .class("hero__inner")
                .child(
                    div()
                        .class("hero__content")
                        .child(
                            a().class("badge intro")
                                .style("--d:0ms")
                                .href("#packages")
                                .child(span().class("badge__dot"))
                                .child(span().text(t.hero_badge))
                                .child(icon("arrow")),
                        )
                        .child(h1().class("hero__title").children(lines))
                        .child(
                            p().class("hero__lead intro")
                                .style("--d:1000ms")
                                .text(t.hero_lead),
                        )
                        .child(
                            div().class("hero__ctas intro").style("--d:1150ms").child(
                                magnetic(
                                    &doc_href(ctx.lang(), "quick-start", None),
                                    "btn--primary btn--lg",
                                    false,
                                )
                                .attr("data-cursor", "Go")
                                .child(inner([
                                    span().text(t.hero_start).into(),
                                    icon("arrow").into(),
                                ])),
                            ),
                        )
                        .child(install(ctx, "intro", "1300ms")),
                )
                .child(
                    div()
                        .class("hero__visual intro")
                        .style("--d:500ms")
                        .attr("role", "img")
                        .attr("aria-label", t.hero_scene)
                        .child(scene()),
                ),
        )
        .child(
            a().class("scroll-hint intro")
                .style("--d:1700ms")
                .href("#packages")
                .attr("aria-label", t.scroll_aria)
                .child(span().class("scroll-hint__mouse").child(span()))
                .child(span().text(t.scroll)),
        )
}

/// The hero scene: the four packages orbit, dive into the box one after the other,
/// the lid closes and the box lights up, then it starts again (CSS).
fn scene() -> Element {
    let pkgs: Vec<Node> = PACKAGES
        .iter()
        .enumerate()
        .map(|(i, p)| {
            div()
                .class(format!("pkg pkg--{i}"))
                .style(format!("--i:{i}"))
                .child(cube(0, Some(p.color), Some(p.glyph), false, false))
                .into()
        })
        .collect();
    div()
        .class("scene")
        .attr("aria-hidden", "true")
        .child(div().class("scene__halo"))
        .child(
            div()
                .class("scene__tilt")
                .child(div().class("scene__rig").child(pbox("", pkgs))),
        )
        .child(div().class("scene__floor"))
        .child(
            ul().class("scene__labels")
                .children(PACKAGES.iter().enumerate().map(|(i, p)| {
                    li().style(format!("--i:{i};--c:{}", p.color))
                        .child(i_tag())
                        .text(p.name)
                })),
        )
}

fn i_tag() -> Element {
    i()
}

fn marquee(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let row = |items: &[&str], class: &str| {
        let group = |hidden: bool| {
            let g = div()
                .class("marquee__group")
                .children(items.iter().flat_map(|w| {
                    [
                        Node::from(span().class("marquee__item").text((*w).to_owned())),
                        Node::from(span().class("marquee__sep").child(logo())),
                    ]
                }));
            if hidden {
                g.attr("aria-hidden", "true")
            } else {
                g
            }
        };
        div().class("marquee__row").class(class.to_owned()).child(
            div()
                .class("marquee__track")
                .child(group(false))
                .child(group(true)),
        )
    };
    section()
        .class("marquee")
        .attr("aria-label", t.marquee_aria)
        .child(row(&t.marquee, "marquee__row--a"))
        .child(row(&MARQUEE_CODE, "marquee__row--b"))
}

fn packages(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let cards = PACKAGES
        .iter()
        .zip(&t.packages)
        .enumerate()
        .map(|(i, (pkg, text))| {
            let (value, unit) = pkg.figure.split_once(' ').unwrap_or((pkg.figure, ""));
            div()
                .class("pkgs__item")
                .attr("data-reveal", "")
                .style(format!("--d:{}ms", i * 100))
                .child(
                    div()
                        .class("tilt pkg-card card")
                        .attr("data-tilt", "10")
                        .style(format!("--c:{}", pkg.color))
                        .child(span().class("tilt__spot").attr("aria-hidden", "true"))
                        .child(
                            div()
                                .class("pkg-card__top")
                                .child(cube(46, Some(pkg.color), Some(pkg.glyph), true, false))
                                .child(span().class("pkg-card__index").text(format!("0{}", i + 1))),
                        )
                        .child(h3().class("pkg-card__name").text(pkg.name))
                        .child(
                            p().class("pkg-card__role")
                                .child(icon(pkg.icon))
                                .text(text.role),
                        )
                        .child(p().class("pkg-card__text").text(text.text))
                        .child(
                            div()
                                .class("pkg-card__figure")
                                .child(strong().text(value).child(small().text(unit)))
                                .child(span().text(text.figure)),
                        )
                        .child(
                            a().class("pkg-card__link")
                                .href(pkg.site)
                                .attr("target", "_blank")
                                .attr("rel", "noopener")
                                .child(icon("globe"))
                                .child(span().text(t.pkg_site))
                                .child(icon("external")),
                        ),
                )
        });
    let mut terms: Vec<Node> = vec![
        span()
            .class("equation__term equation__term--brand")
            .style("--i:0")
            .text("paquets")
            .into(),
        span().class("equation__op").style("--i:1").text("=").into(),
    ];
    for (i, p) in PACKAGES.iter().enumerate() {
        if i > 0 {
            terms.push(
                span()
                    .class("equation__op")
                    .style(format!("--i:{}", i * 2 + 1))
                    .text("+")
                    .into(),
            );
        }
        terms.push(
            span()
                .class("equation__term")
                .style(format!("--i:{};--c:{}", i * 2 + 2, p.color))
                .text(p.name)
                .into(),
        );
    }
    section()
        .class("section")
        .id("packages")
        .child(head("01", t.pkg_label, t.pkg_title, Some(t.pkg_lead)))
        .child(div().class("pkgs").children(cards))
        .child(
            div()
                .class("equation")
                .attr("data-reveal", "")
                .child(code().children(terms))
                .child(p().text(t.pkg_alone)),
        )
}

fn showcase(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    section()
        .class("section showcase")
        .id("code")
        .child(head("02", t.code_label, t.code_title, Some(t.code_lead)))
        .child(island(
            "showcase",
            ctx.lang().code(),
            crate::islands::showcase,
        ))
}

fn features(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    section()
        .class("section")
        .id("features")
        .child(head(
            "03",
            t.features_label,
            t.features_title,
            Some(t.features_lead),
        ))
        .child(
            div()
                .class("features")
                .children(FEATURES.iter().zip(&t.features).enumerate().map(
                    |(i, ((name, tag), f))| {
                        div()
                            .class("features__item")
                            .attr("data-reveal", "")
                            .style(format!("--d:{}ms", (i % 4) * 90))
                            .child(
                                div()
                                    .class("tilt feature card")
                                    .attr("data-tilt", "10")
                                    .child(span().class("tilt__spot").attr("aria-hidden", "true"))
                                    .child(span().class("feature__icon").child(icon(name)))
                                    .child(h3().text(f.title))
                                    .child(p().text(f.text))
                                    .child(code().class("feature__tag").text(*tag)),
                            )
                    },
                )),
        )
}

fn numbers(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let lang = ctx.lang();
    section()
        .class("section numbers")
        .id("numbers")
        .child(div().class("numbers__bg").attr("aria-hidden", "true"))
        .child(head("04", t.numbers_label, t.numbers_title, None))
        .child(
            div()
                .class("stats")
                .children(NUMBERS.iter().zip(&t.numbers).enumerate().map(
                    |(i, ((value, decimals, prefix, suffix), item))| {
                        article()
                            .class("stat")
                            .attr("data-reveal", "")
                            .style(format!("--d:{}ms", (i % 3) * 110))
                            .child(
                                div()
                                    .class("stat__value")
                                    .child(span().class("stat__affix").text(prefix.trim()))
                                    .child(
                                        span()
                                            .class("counter")
                                            .attr("data-count", value.to_string())
                                            .attr("data-decimals", decimals.to_string())
                                            .text(number(lang, *value, *decimals)),
                                    )
                                    .child(span().class("stat__unit").text(suffix.trim())),
                            )
                            .child(h3().class("stat__label").text(item.title))
                            .child(p().class("stat__detail").text(item.text))
                            .child(span().class("stat__line"))
                    },
                )),
        )
}

fn journey(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let icons = ["box", "terminal", "git", "cloud"];
    let steps = t
        .journey
        .iter()
        .zip(JOURNEY)
        .enumerate()
        .map(|(i, (item, command))| {
            // The reveal is on an inner element: the browser toggles the classes of the
            // step itself while the terminal types.
            li().class("timeline__step").child(
                div()
                    .class("timeline__inner")
                    .attr("data-reveal", "")
                    .style(format!("--d:{}ms", i * 90))
                    .child(span().class("timeline__dot").child(icon(icons[i])))
                    .child(
                        div()
                            .class("timeline__body")
                            .child(span().class("timeline__num").text(format!("0{}", i + 1)))
                            .child(h3().text(item.title))
                            .child(p().text(item.text))
                            .child(code().class("timeline__cmd").text(command)),
                    ),
            )
        });
    section()
        .class("section journey")
        .id("start")
        .child(head(
            "05",
            t.journey_label,
            t.journey_title,
            Some(t.journey_lead),
        ))
        .child(
            div()
                .class("journey__grid")
                .child(
                    ol().class("timeline")
                        .id("journey")
                        .child(
                            span()
                                .class("timeline__rail")
                                .attr("aria-hidden", "true")
                                .child(span().class("timeline__fill")),
                        )
                        .child(
                            span()
                                .class("timeline__traveler")
                                .attr("aria-hidden", "true")
                                .child(cube(26, None, None, true, false)),
                        )
                        .children(steps),
                )
                .child(
                    div()
                        .class("journey__terminal")
                        .attr("data-reveal", "")
                        .style("--d:150ms")
                        .child(island(
                            "terminal",
                            ctx.lang().code(),
                            crate::islands::terminal,
                        )),
                ),
        )
}

fn phone(t: &Texts) -> Element {
    let log = [
        "Step 1/6 : FROM rust:slim",
        "Compiling paquets v0.1.0",
        "Compiling my-app v0.1.0",
        "Finished release",
        "Launching web.1",
    ];
    div()
        .class("phone")
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("phone__frame")
                .child(div().class("phone__notch"))
                .child(
                    div()
                        .class("phone__screen")
                        .child(
                            div()
                                .class("phone__status")
                                .child(span().text("9:41"))
                                .child(
                                    span().class("phone__bars").child(i()).child(i()).child(i()),
                                ),
                        )
                        .child(
                            div()
                                .class("phone__view phone__view--1")
                                .child(
                                    div()
                                        .class("phone__app")
                                        .child(span().class("phone__heroku").text("H"))
                                        .child(b().text(t.phone[0])),
                                )
                                .child(
                                    div()
                                        .class("phone__field")
                                        .child(small().text(t.phone[5]))
                                        .child(span().class("phone__typed").text(t.phone[1])),
                                )
                                .child(
                                    div()
                                        .class("phone__field")
                                        .child(small().text(t.phone[6]))
                                        .child(span().text(t.phone[7])),
                                )
                                .child(
                                    div()
                                        .class("phone__btn")
                                        .child(span().class("phone__tap"))
                                        .text(t.phone[0]),
                                ),
                        )
                        .child(
                            div()
                                .class("phone__view phone__view--2")
                                .child(p().class("phone__title").text(t.phone[2]))
                                .child(
                                    div()
                                        .class("phone__log")
                                        .children(log.map(|l| span().text(l))),
                                )
                                .child(div().class("phone__progress").child(span())),
                        )
                        .child(
                            div()
                                .class("phone__view phone__view--3")
                                .child(div().class("phone__check").child(icon("check")))
                                .child(p().class("phone__title").text(t.phone[3]))
                                .child(
                                    p().class("phone__url")
                                        .text(format!("{}.herokuapp.com", t.phone[1])),
                                )
                                .child(
                                    div().class("phone__btn phone__btn--ghost").text(t.phone[4]),
                                ),
                        ),
                ),
        )
        .child(div().class("phone__glow"))
}

fn deploy(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    section()
        .class("section deploy")
        .id("deploy")
        .child(head(
            "06",
            t.deploy_label,
            t.deploy_title,
            Some(t.deploy_lead),
        ))
        .child(
            div()
                .class("deploy__grid")
                .child(
                    div()
                        .class("deploy__visual")
                        .attr("data-reveal", "")
                        .child(phone(t)),
                )
                .child(
                    div()
                        .class("deploy__content")
                        .child(ol().class("deploy__steps").children(
                            t.deploy_steps.iter().enumerate().map(|(i, step)| {
                                li().attr("data-reveal", "")
                                    .style(format!("--d:{}ms", i * 110))
                                    .child(span().class("deploy__num").text((i + 1).to_string()))
                                    .child(
                                        div()
                                            .child(h3().text(step.title))
                                            .child(p().text(step.text)),
                                    )
                            }),
                        ))
                        .child(
                            div()
                                .class("deploy__secrets")
                                .attr("data-reveal", "")
                                .style("--d:200ms")
                                .child(code().text("HEROKU_API_KEY"))
                                .child(code().text("HEROKU_APP_NAME"))
                                .child(code().text("git push → master")),
                        )
                        .child(
                            div()
                                .class("deploy__actions")
                                .attr("data-reveal", "")
                                .style("--d:260ms")
                                .child(
                                    magnetic(
                                        &doc_href(ctx.lang(), "deploy-heroku-mobile", None),
                                        "btn--primary btn--lg",
                                        false,
                                    )
                                    .child(inner([
                                        span().text(t.deploy_guide).into(),
                                        icon("arrow").into(),
                                    ])),
                                ),
                        )
                        .child(
                            p().class("deploy__note")
                                .attr("data-reveal", "")
                                .style("--d:300ms")
                                .text(t.deploy_note),
                        ),
                ),
        )
}

fn docs_preview(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let lang = ctx.lang();
    section()
        .class("section")
        .id("docs")
        .child(head(
            "07",
            t.docs_label,
            t.docs_home_title,
            Some(t.docs_home_lead),
        ))
        .child(
            div()
                .class("doc-preview")
                .children(CATEGORIES.iter().enumerate().map(|(c, category)| {
                    div()
                        .class("doc-preview__item")
                        .attr("data-reveal", "")
                        .style(format!("--d:{}ms", c * 80))
                        .child(
                            div()
                                .class("tilt doc-preview__card card")
                                .attr("data-tilt", "6")
                                .child(span().class("tilt__spot").attr("aria-hidden", "true"))
                                .child(span().class("doc-preview__num").text(format!("0{}", c + 1)))
                                .child(span().class("doc-preview__icon").child(icon(category.icon)))
                                .child(h3().text(lang.pick(category.title)))
                                .child(ul().children(pages_of(c).map(|page| {
                                    li().child(
                                        a().href(doc_href(lang, page.slug, None))
                                            .text(page.title(lang)),
                                    )
                                }))),
                        )
                })),
        )
        .child(
            div().class("center").attr("data-reveal", "").child(
                magnetic(
                    &Route::new(lang, PageKind::Docs).href(),
                    "btn--ghost btn--lg",
                    false,
                )
                .child(inner([
                    icon("book").into(),
                    span().text(t.docs_browse).into(),
                ])),
            ),
        )
}

fn cta(ctx: &Ctx<'_>) -> Element {
    let t = ctx.t();
    let mut actions = div()
        .class("cta__actions")
        .attr("data-reveal", "")
        .style("--d:200ms")
        .child(
            magnetic(
                &doc_href(ctx.lang(), "quick-start", None),
                "btn--primary btn--lg",
                false,
            )
            .attr("data-cursor", "Go")
            .child(inner([
                span().text(t.cta_start).into(),
                icon("arrow").into(),
            ])),
        );
    if ctx.site.live {
        actions = actions.child(
            magnetic("/demo", "btn--ghost btn--lg", false)
                .child(inner([icon("spark").into(), span().text(t.demo).into()])),
        );
    }
    section()
        .class("cta")
        .child(div().class("cta__bg").attr("aria-hidden", "true"))
        .child(
            div()
                .class("cta__cube")
                .attr("aria-hidden", "true")
                .attr("data-speed", "-0.06")
                .child(cube(120, None, None, true, false)),
        )
        .child(
            h2().class("cta__title")
                .attr("data-reveal", "")
                .child(span().text(t.cta_title[0]))
                .child(
                    span()
                        .class("cta__outline")
                        .attr("data-text", t.cta_title[1])
                        .text(t.cta_title[1]),
                ),
        )
        .child(install(ctx, "install--center", "120ms").attr("data-reveal", ""))
        .child(actions)
}

// ----- Documentation ----------------------------------------------------------------------

/// The big search field of the docs hub: it opens the search of the navigation.
fn search_field(t: &Texts) -> Element {
    button()
        .class("doc-search")
        .attr("type", "button")
        .attr("data-search-open", "")
        .child(icon("search"))
        .child(span().text(t.docs_search))
        .child(kbd().text("/"))
}

pub fn docs_home(ctx: &Ctx<'_>) -> Page {
    let t = ctx.t();
    let lang = ctx.lang();
    let quick = [
        ("quick-start", "rocket"),
        ("deploy-heroku-mobile", "phone"),
        ("cheatsheet", "page"),
    ];
    let content =
        main()
            .class("docs-home")
            .child(
                div()
                    .class("docs-home__hero")
                    .child(div().class("docs-home__glow").attr("aria-hidden", "true"))
                    .child(
                        div()
                            .class("docs-home__cube")
                            .attr("aria-hidden", "true")
                            .child(cube(64, None, None, true, false)),
                    )
                    .child(
                        p().class("eyebrow")
                            .child(span().class("eyebrow__num").text("Paquets"))
                            .text(lang.name()),
                    )
                    .child(h1().class("docs-home__title").text(t.docs_title))
                    .child(p().class("docs-home__lead").text(t.docs_lead))
                    .child(search_field(t))
                    .child(p().class("docs-home__hint").text(t.docs_search_hint)),
            )
            .child(
                div()
                    .class("docs-home__quick")
                    .child(h2().class("docs-home__h2").text(t.docs_quick))
                    .child(div().class("docs-home__quick-grid").children(
                        quick.iter().enumerate().filter_map(|(i, (slug, name))| {
                            let (_, page) = catalog::find(slug)?;
                            Some(
                                a().class("doc-quick card")
                                    .href(doc_href(lang, page.slug, None))
                                    .attr("data-reveal", "")
                                    .style(format!("--d:{}ms", i * 80))
                                    .child(span().class("doc-quick__icon").child(icon(name)))
                                    .child(span().class("doc-quick__title").text(page.title(lang)))
                                    .child(span().class("doc-quick__text").text(page.summary(lang)))
                                    .child(span().class("doc-quick__arrow").child(icon("arrow"))),
                            )
                        }),
                    )),
            )
            .child(
                div()
                    .class("docs-home__cats")
                    .children(CATEGORIES.iter().enumerate().map(|(c, category)| {
                        let pages: Vec<&catalog::Page> = pages_of(c).collect();
                        article()
                            .class("doc-cat card")
                            .attr("data-reveal", "")
                            .style(format!("--d:{}ms", (c % 3) * 90))
                            .child(
                                div()
                                    .class("doc-cat__head")
                                    .child(span().class("doc-cat__icon").child(icon(category.icon)))
                                    .child(h2().text(lang.pick(category.title)))
                                    .child(span().class("doc-cat__count").text(
                                        t.docs_pages.replace("{}", &pages.len().to_string()),
                                    )),
                            )
                            .child(ul().children(pages.iter().map(|page| {
                                li().child(
                                    a().href(doc_href(lang, page.slug, None))
                                        .child(strong().text(page.title(lang)))
                                        .child(span().text(page.summary(lang))),
                                )
                            })))
                    })),
            );
    layout(
        ctx,
        &format!("{} — Paquets", t.docs_title),
        t.docs_lead,
        content,
    )
}

fn sidebar(ctx: &Ctx<'_>, current: &str) -> Element {
    let t = ctx.t();
    let lang = ctx.lang();
    aside()
        .class("doc-side")
        .id("doc-side")
        .child(
            div()
                .class("doc-side__head")
                .child(
                    a().class("doc-side__home")
                        .href(Route::new(lang, PageKind::Docs).href())
                        .child(logo())
                        .child(span().text(t.docs_title)),
                )
                .child(
                    button()
                        .class("doc-side__close")
                        .attr("type", "button")
                        .attr("data-toggle-side", "")
                        .attr("aria-label", t.docs_menu)
                        .child(icon("close")),
                ),
        )
        .child(
            nav()
                .class("doc-side__nav")
                .attr("aria-label", t.docs_menu)
                .children(CATEGORIES.iter().enumerate().map(|(c, category)| {
                    div()
                        .class("doc-side__group")
                        .child(
                            p().class("doc-side__cat")
                                .child(icon(category.icon))
                                .text(lang.pick(category.title)),
                        )
                        .child(ul().children(pages_of(c).map(|page| {
                            let link = a()
                                .href(doc_href(lang, page.slug, None))
                                .text(page.title(lang));
                            li().child(if page.slug == current {
                                link.class("is-active").attr("aria-current", "page")
                            } else {
                                link
                            })
                        })))
                })),
        )
        .child(
            a().class("doc-side__api")
                .href(DOCS_RS)
                .attr("target", "_blank")
                .attr("rel", "noopener")
                .child(icon("crate"))
                .child(span().text(t.docs_api)),
        )
}

pub fn doc(ctx: &Ctx<'_>, slug: &'static str) -> Page {
    let t = ctx.t();
    let lang = ctx.lang();
    let (index, page) = catalog::find(slug).expect("a page of the catalog");
    let doc: Option<&Doc> = ctx.site.docs.get(lang, slug);
    let title = doc.map_or(page.title(lang), |d| d.title.as_str());
    let description = doc
        .map(|d| d.description.as_str())
        .filter(|d| !d.is_empty())
        .unwrap_or(page.summary(lang));
    let prev = index.checked_sub(1).map(|i| &PAGES[i]);
    let next = PAGES.get(index + 1);
    let category = page.category;
    let pager = |page: &catalog::Page, label: &str, class: &str| {
        a().class(format!("doc-pager__link {class}"))
            .href(doc_href(lang, page.slug, None))
            .child(span().class("doc-pager__label").text(label.to_owned()))
            .child(span().class("doc-pager__title").text(page.title(lang)))
    };
    let mut article = article().class("docs__main").child(
        p().class("docs__crumb")
            .child(
                a().href(Route::new(lang, PageKind::Docs).href())
                    .text(t.docs_title),
            )
            .child(span().class("docs__crumb-sep").text("/"))
            .child(
                a().href(doc_href(
                    lang,
                    pages_of(category).next().map_or(slug, |p| p.slug),
                    None,
                ))
                .text(lang.pick(CATEGORIES[category].title)),
            ),
    );
    article = match doc {
        Some(doc) => {
            if doc.fallback {
                article = article.child(
                    div()
                        .class("callout callout--note doc-fallback")
                        .child(p().text(t.docs_fallback)),
                );
            }
            article.child(div().class("doc-body").html(doc.html.clone()))
        }
        None => article.child(
            div()
                .class("doc-body")
                .child(h1().class("doc-title").text(title.to_owned()))
                .child(
                    div()
                        .class("doc-missing card")
                        .child(p().text(t.docs_missing)),
                ),
        ),
    };
    let mut pager_nav = nav().class("doc-pager");
    if let Some(prev) = prev {
        pager_nav = pager_nav.child(pager(prev, t.docs_prev, "doc-pager__link--prev"));
    }
    if let Some(next) = next {
        pager_nav = pager_nav.child(pager(next, t.docs_next, "doc-pager__link--next"));
    }
    article = article.child(pager_nav);
    let toc = doc
        .map(|d| &d.toc)
        .filter(|toc| !toc.is_empty())
        .map(|toc| {
            nav()
                .class("doc-toc")
                .attr("aria-label", t.docs_toc)
                .child(p().class("doc-toc__title").text(t.docs_toc))
                .child(ul().children(toc.iter().map(|e| {
                    let item = li().child(a().href(format!("#{}", e.id)).text(e.text.clone()));
                    if e.level == 3 {
                        item.class("is-sub")
                    } else {
                        item
                    }
                })))
        });
    let content = main()
        .class("docs")
        .child(
            div()
                .class("docs__bar")
                .child(
                    button()
                        .class("docs__menu")
                        .attr("type", "button")
                        .attr("data-toggle-side", "")
                        .attr("aria-controls", "doc-side")
                        .attr("aria-expanded", "false")
                        .child(icon("menu"))
                        .child(span().text(t.docs_menu)),
                )
                .child(span().class("docs__bar-title").text(page.title(lang))),
        )
        .child(sidebar(ctx, slug))
        .child(div().class("docs__scrim").attr("aria-hidden", "true"))
        .child(article)
        .child(aside().class("docs__toc").children(toc));
    layout(
        ctx,
        &format!("{title} — {}", t.docs_meta),
        description,
        content,
    )
}

// ----- Not found ----------------------------------------------------------------------------

pub fn not_found(ctx: &Ctx<'_>) -> Page {
    let t = ctx.t();
    let lang = ctx.lang();
    let content = main()
        .class("nf")
        .child(div().class("nf__glow").attr("aria-hidden", "true"))
        .child(
            div().class("nf__stage").attr("aria-hidden", "true").child(
                div()
                    .class("nf__rig")
                    .child(pbox("pbox--empty", Vec::new())),
            ),
        )
        .child(
            p().class("eyebrow")
                .child(span().class("eyebrow__num").text("404"))
                .text("Paquets"),
        )
        .child(h1().class("nf__title").text(t.nf_title))
        .child(p().class("nf__text").text(t.nf_text))
        .child(
            div()
                .class("nf__actions")
                .child(
                    magnetic(&ctx.home(""), "btn--primary btn--lg", false)
                        .child(inner([span().text(t.nf_home).into(), icon("arrow").into()])),
                )
                .child(
                    magnetic(
                        &Route::new(lang, PageKind::Docs).href(),
                        "btn--ghost btn--lg",
                        false,
                    )
                    .child(inner([
                        icon("book").into(),
                        span().text(t.docs_title).into(),
                    ])),
                ),
        );
    layout(ctx, &format!("404 — {}", t.nf_title), t.nf_text, content).status(404)
}

/// A JavaScript string literal that cannot close the `<script>` element.
fn js_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The script of the pages a static host serves without knowing the visitor's language:
/// goes to `/{lang}/{target}` for the language of the address (`/fr/…` on an unknown
/// page), else the browser's, else English. Without JavaScript, a `<meta>` sends to
/// English.
fn language_redirect(target: &str) -> String {
    let codes: Vec<String> = Lang::ALL.iter().map(|l| js_string(l.code())).collect();
    format!(
        concat!(
            "<script>(function(){{var b={base},L=[{codes}],c=null;",
            "var s=location.pathname.slice(b.length).split('/').filter(Boolean)[0];",
            "if(L.indexOf(s)>=0)c=s;",
            "var l=navigator.languages||[navigator.language||''];",
            "for(var i=0;i<l.length&&!c;i++){{var x=String(l[i]).slice(0,2).toLowerCase();",
            "if(L.indexOf(x)>=0)c=x}}",
            "location.replace(b+'/'+(c||'en')+'/'+{target}+location.search+location.hash)}})()</script>",
            r#"<noscript><meta http-equiv="refresh" content="0; url={fallback}"/></noscript>"#,
        ),
        base = js_string(base()),
        codes = codes.join(","),
        target = js_string(target),
        fallback = link(&format!("/en/{target}")),
    )
}

/// The page a static host serves at the root (`target` empty) and for an unknown address
/// (`target` = `404/`): it goes to the visitor's language by itself and links every
/// language for the others.
pub fn language_picker(site: &Site, origin: &str, target: &str) -> Page {
    let ctx = Ctx {
        site,
        route: Route::new(Lang::En, PageKind::NotFound),
        origin: origin.to_owned(),
        intro: false,
    };
    let t = ctx.t();
    let content = main()
        .class("nf")
        .child(div().class("nf__glow").attr("aria-hidden", "true"))
        .child(
            p().class("eyebrow")
                .child(span().class("eyebrow__num").text("📦"))
                .text("Paquets"),
        )
        .child(h1().class("nf__title").text(t.footer_lang))
        .child(p().class("nf__text").text(t.meta_title))
        .child(div().class("nf__actions").children(Lang::ALL.map(|l| {
            magnetic(
                &link(&format!("/{}/{target}", l.code())),
                "btn--ghost btn--lg",
                false,
            )
            .attr("hreflang", l.code())
            .attr("lang", l.code())
            .child(inner([span().text(l.name()).into(), icon("arrow").into()]))
        })));
    layout(&ctx, "Paquets", t.meta_title, content).head(language_redirect(target))
}

/// The page of a route.
pub fn render(ctx: &Ctx<'_>) -> Page {
    match ctx.route.page {
        PageKind::Home => home(ctx),
        PageKind::Docs => docs_home(ctx),
        PageKind::Doc(slug) => doc(ctx, slug),
        PageKind::NotFound => not_found(ctx),
    }
}
