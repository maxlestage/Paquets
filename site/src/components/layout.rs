//! Preloader, cursor, navigation, language switch and footer.

use std::cell::Cell;
use std::rc::Rc;

use gloo_events::EventListener;
use gloo_timers::callback::Interval;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use super::cube::PaquetsBox;
use super::ui::{Icon, Logo, Magnetic, Scramble};
use crate::data::{CRATES_IO, DOCS_RS, GITHUB, HEROKU_DEPLOY, PACKAGES};
use crate::docs::index::{CATEGORIES, pages_of};
use crate::docs::render::page_href;
use crate::dom;
use crate::i18n::{Lang, use_lang};

// ----- Preloader ---------------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct LoaderProps {
    pub done: bool,
}

/// A box spins in, its lid opens and light comes out while a counter
/// reaches 100; then a gradient curtain lifts.
#[component]
pub fn Loader(props: &LoaderProps) -> Html {
    let t = use_lang().lang.texts();
    let percent = use_state(|| 0u32);
    {
        let percent = percent.clone();
        use_effect_with((), move |_| {
            let mut p = 0u32;
            let interval = Interval::new(16, move || {
                if p < 100 {
                    // Fast, then slower, like a real download.
                    p = (p + 1 + (100 - p) / 16).min(100);
                    percent.set(p);
                }
            });
            move || drop(interval)
        });
    }
    html! {
        <div class={classes!("loader", props.done.then_some("loader--done"))} aria-hidden="true">
            <div class="loader__inner">
                <div class="loader__stage">
                    <div class="loader__rig">
                        <PaquetsBox class="pbox--loader">
                            <span class="loader__beam"></span>
                        </PaquetsBox>
                    </div>
                    <div class="loader__shadow"></div>
                </div>
                <div class="loader__word">
                    { for "PAQUETS".chars().enumerate().map(|(i, c)| html! {
                        <span style={format!("--i:{i}")}>{ c }</span>
                    }) }
                </div>
                <div class="loader__bar"><span style={format!("transform:scaleX({})", f64::from(*percent) / 100.0)}></span></div>
                <div class="loader__label">{ t.loader }<span class="loader__dots"><i>{"."}</i><i>{"."}</i><i>{"."}</i></span></div>
            </div>
            <div class="loader__percent">{ format!("{:03}", *percent) }</div>
            <div class="loader__curtain"></div>
        </div>
    }
}

// ----- Cursor -------------------------------------------------------------------

/// A dot that follows the mouse, and a ring that catches up with a delay;
/// the ring grows on clickable elements and shows `data-cursor` labels.
#[component]
pub fn Cursor() -> Html {
    let dot = use_node_ref();
    let ring = use_node_ref();
    let label = use_state(String::new);
    {
        let (dot, ring, label) = (dot.clone(), ring.clone(), label.clone());
        use_effect_with((), move |_| {
            let mut keep: Vec<EventListener> = Vec::new();
            let mut anim = None;
            if dom::fine_pointer() && !dom::reduced_motion() {
                if let Some(body) = dom::document().body() {
                    let _ = body.class_list().add_1("has-cursor");
                }
                let target = Rc::new(Cell::new((-100.0f64, -100.0f64)));
                let win = dom::window();
                {
                    let target = target.clone();
                    let dot = dot.clone();
                    keep.push(EventListener::new(&win, "mousemove", move |e| {
                        let Some(e) = e.dyn_ref::<MouseEvent>() else {
                            return;
                        };
                        let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
                        target.set((x, y));
                        if let Some(dot) = dot.cast::<HtmlElement>() {
                            dom::set_var(&dot, "transform", &format!("translate3d({x}px,{y}px,0)"));
                        }
                    }));
                }
                {
                    let ring = ring.clone();
                    let label = label.clone();
                    keep.push(EventListener::new(&win, "mouseover", move |e| {
                        let hovered = e
                            .target()
                            .and_then(|t| t.dyn_into::<Element>().ok())
                            .and_then(|el| {
                                el.closest("a, button, input, [data-cursor]").ok().flatten()
                            });
                        if let Some(ring) = ring.cast::<HtmlElement>() {
                            let _ = ring
                                .class_list()
                                .toggle_with_force("cursor__ring--hover", hovered.is_some());
                        }
                        label.set(
                            hovered
                                .and_then(|el| el.get_attribute("data-cursor"))
                                .unwrap_or_default(),
                        );
                    }));
                }
                for (event, down) in [("mousedown", true), ("mouseup", false)] {
                    let ring = ring.clone();
                    keep.push(EventListener::new(&win, event, move |_| {
                        if let Some(ring) = ring.cast::<HtmlElement>() {
                            let _ = ring
                                .class_list()
                                .toggle_with_force("cursor__ring--down", down);
                        }
                    }));
                }
                let mut pos = (-100.0f64, -100.0f64);
                anim = Some(dom::raf_loop(move |_| {
                    let (tx, ty) = target.get();
                    pos.0 += (tx - pos.0) * 0.17;
                    pos.1 += (ty - pos.1) * 0.17;
                    if let Some(ring) = ring.cast::<HtmlElement>() {
                        dom::set_var(
                            &ring,
                            "transform",
                            &format!("translate3d({:.1}px,{:.1}px,0)", pos.0, pos.1),
                        );
                    }
                    true
                }));
            }
            move || {
                drop(keep);
                drop(anim);
            }
        });
    }
    html! {
        <div class="cursor" aria-hidden="true">
            <div class="cursor__dot" ref={dot}></div>
            <div class={classes!("cursor__ring", (!label.is_empty()).then_some("cursor__ring--label"))} ref={ring}>
                <span>{ (*label).clone() }</span>
            </div>
        </div>
    }
}

// ----- Navigation ------------------------------------------------------------------

const SECTIONS: [&str; 4] = ["#packages", "#code", "#features", "#deploy"];

#[derive(Properties, PartialEq)]
pub struct LangSwitchProps {
    #[prop_or_default]
    pub class: Classes,
}

/// The language selector: `EN FR ES`.
#[component]
pub fn LangSwitch(props: &LangSwitchProps) -> Html {
    let ctx = use_lang();
    let t = ctx.lang.texts();
    let index = Lang::ALL.iter().position(|&l| l == ctx.lang).unwrap_or(0);
    html! {
        <div class={classes!("lang", props.class.clone())} role="group" aria-label={t.lang_aria} style={format!("--idx:{index}")}>
            <Icon name="globe" />
            <span class="lang__pill" aria-hidden="true"></span>
            { for Lang::ALL.iter().map(|&l| {
                let set = ctx.set.clone();
                html! {
                    <button class={classes!("lang__btn", (l == ctx.lang).then_some("is-active"))}
                            lang={l.code()} title={l.name()} aria-pressed={(l == ctx.lang).to_string()}
                            onclick={Callback::from(move |_: MouseEvent| set.emit(l))}>
                        { l.code().to_uppercase() }
                    </button>
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavProps {
    /// True on the documentation pages.
    pub docs: bool,
}

#[component]
pub fn Nav(props: &NavProps) -> Html {
    let t = use_lang().lang.texts();
    let open = use_state(|| false);
    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    let close = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(false))
    };
    use_effect_with(*open, |open| {
        if let Some(body) = dom::document().body() {
            let _ = body.class_list().toggle_with_force("menu-open", *open);
        }
    });
    html! {
        <>
            <div class="progress" id="progress" aria-hidden="true"></div>
            <header class={classes!("nav", open.then_some("nav--open"), props.docs.then_some("nav--docs"))} id="nav">
                <a class="nav__brand" href="#top" aria-label={t.brand_aria} onclick={close.clone()}>
                    <Logo />
                    <span>{ "Paquets" }</span>
                </a>
                <nav class="nav__links" aria-label={t.nav_aria}>
                    { for SECTIONS.iter().zip(t.nav_links.iter()).map(|(href, text)| html! {
                        <a href={*href} class="nav__link"><Scramble text={*text} /></a>
                    }) }
                    <a href="#/docs" class={classes!("nav__link", "nav__link--docs", props.docs.then_some("is-active"))}>
                        <Icon name="book" /><Scramble text={t.nav_docs} />
                    </a>
                </nav>
                <LangSwitch />
                <Magnetic href={GITHUB} external=true class={classes!("btn", "btn--ghost", "nav__cta")} strength={0.25} label={Some(AttrValue::from("GitHub"))}>
                    <Icon name="github" />
                    <span>{ "GitHub" }</span>
                </Magnetic>
                <button class="nav__burger" onclick={toggle} aria-label={t.menu} aria-expanded={(*open).to_string()}>
                    <span></span><span></span>
                </button>
            </header>
            <div class={classes!("menu", open.then_some("menu--open"))} aria-hidden={(!*open).to_string()}>
                <div class="menu__glow" aria-hidden="true"></div>
                <nav>
                    { for SECTIONS.iter().zip(t.nav_links.iter()).enumerate().map(|(i, (href, text))| html! {
                        <a href={*href} style={format!("--i:{i}")} onclick={close.clone()} tabindex={if *open { "0" } else { "-1" }}>
                            <span class="menu__num">{ format!("0{}", i + 1) }</span>{ *text }
                        </a>
                    }) }
                    <a href="#/docs" style="--i:4" onclick={close.clone()} tabindex={if *open { "0" } else { "-1" }}>
                        <span class="menu__num">{ "05" }</span>{ t.docs_title }
                    </a>
                    <a href={GITHUB} target="_blank" rel="noopener" style="--i:5" onclick={close.clone()} tabindex={if *open { "0" } else { "-1" }}>
                        <span class="menu__num">{ "06" }</span>{ "GitHub" }
                    </a>
                </nav>
                <p class="menu__foot">{ t.footer_tagline }</p>
            </div>
        </>
    }
}

// ----- Footer -----------------------------------------------------------------------

#[component]
pub fn Footer() -> Html {
    let t = use_lang().lang.texts();
    let lang = use_lang().lang;
    let to_top = Callback::from(|e: MouseEvent| {
        e.prevent_default();
        dom::scroll_top(true);
    });
    // (text, link, external)
    let product: Vec<(&str, String, bool)> = ["#packages", "#code", "#features", "#numbers", "#deploy"]
        .iter()
        .zip(t.footer_product)
        .map(|(href, text)| (text, (*href).to_owned(), false))
        .collect();
    let docs: Vec<(&str, String, bool)> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(c, cat)| {
            let first = pages_of(c).next().map(|p| p.slug).unwrap_or_default();
            (lang.pick(cat.title), page_href(first, None), false)
        })
        .collect();
    let packages: Vec<(&str, String, bool)> = PACKAGES
        .iter()
        .map(|p| (p.name, p.repo.to_owned(), true))
        .collect();
    let resources: Vec<(&str, String, bool)> = [GITHUB, CRATES_IO, DOCS_RS, HEROKU_DEPLOY]
        .iter()
        .zip(t.footer_resources)
        .map(|(href, text)| (text, (*href).to_owned(), true))
        .collect();
    let columns = [
        (t.footer_cols[0], product),
        (t.footer_cols[1], docs),
        (t.footer_cols[2], packages),
        (t.footer_cols[3], resources),
    ];
    html! {
        <footer class="footer">
            <div class="footer__glow" aria-hidden="true"></div>
            <div class="footer__top">
                <div class="footer__brand" data-reveal="">
                    <a class="footer__logo" href="#top" aria-label={t.brand_aria}>
                        <Logo /><span>{ "Paquets" }</span>
                    </a>
                    <p class="footer__tagline">{ t.footer_tagline }</p>
                    <div class="footer__lang">
                        <span>{ t.footer_lang }</span>
                        <LangSwitch />
                    </div>
                </div>
                <div class="footer__cols">
                    { for columns.iter().enumerate().map(|(c, (title, links))| html! {
                        <div class="footer__col" data-reveal="" style={format!("--d:{}ms", c * 80)}>
                            <h2>{ *title }</h2>
                            <ul>
                                { for links.iter().map(|(text, href, external)| html! {
                                    <li><a href={href.clone()} class="footer__link"
                                           target={external.then_some("_blank")} rel={external.then_some("noopener")}>
                                        <span>{ *text }</span><Icon name="arrow" />
                                    </a></li>
                                }) }
                            </ul>
                        </div>
                    }) }
                </div>
            </div>
            <div class="footer__word" aria-hidden="true" data-reveal="">
                { for "PAQUETS".chars().enumerate().map(|(i, c)| html! {
                    <span style={format!("--i:{i}")}>{ c }</span>
                }) }
            </div>
            <div class="footer__bottom">
                <p class="footer__made"><span class="footer__heart">{ "📦" }</span>{ t.footer_made }</p>
                <p class="footer__copy">
                    { "© 2026 " }<a href={crate::data::AUTHOR} target="_blank" rel="noopener">{ "Maxime Lestage" }</a>{ " — MIT OR Apache-2.0" }
                </p>
                <a href="#top" class="totop" onclick={to_top} data-cursor={t.totop_cursor} aria-label={t.totop_aria}>
                    <svg viewBox="0 0 100 100" class="totop__text" aria-hidden="true">
                        <defs><path id="totop-circle" d="M50 50 m-38 0 a38 38 0 1 1 76 0 a38 38 0 1 1 -76 0" /></defs>
                        // The text fits the circle exactly, whatever the language.
                        <text><textPath href="#totop-circle" textLength="236" lengthAdjust="spacing">{ format!("{} ", t.totop) }</textPath></text>
                    </svg>
                    <Icon name="up" />
                </a>
            </div>
        </footer>
    }
}
