//! The sections of the home page, from top to bottom.

use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::callback::Interval;
use web_sys::HtmlElement;
use yew::prelude::*;

use super::cube::{Assembly, Cube, FloatingCubes};
use super::terminal::Terminal;
use super::ui::{Code, CopyButton, Counter, Icon, Logo, Magnetic, Split, Tilt};
use crate::data::{
    FEATURES, GITHUB, HEROKU_BUTTON, HEROKU_DEPLOY, INSTALL, JOURNEY, MARQUEE_CODE, NUMBERS,
    PACKAGES, SAMPLES, localize,
};
use crate::docs::index::{CATEGORIES, pages_of};
use crate::docs::render::page_href;
use crate::dom;
use crate::i18n::use_lang;

// ----- Hero ------------------------------------------------------------------------

#[component]
pub fn Hero() -> Html {
    let t = use_lang().lang.texts();
    // Each line starts its animation after the previous one.
    let mut offset = 0;
    let lines: Vec<Html> = t
        .hero_title
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let split = html! { <Split text={*line} offset={offset} /> };
            offset += line.chars().count() + 1;
            html! {
                <span class={classes!("hero__line", (i == t.hero_accent).then_some("hero__line--accent"))}>
                    if i == t.hero_accent { <span class="grad-text">{ split }</span> } else { { split } }
                </span>
            }
        })
        .collect();
    html! {
        <section class="hero" id="top">
            <div class="hero__bg" aria-hidden="true">
                <div class="aurora">
                    <span class="aurora__blob aurora__blob--a" data-speed="0.14"></span>
                    <span class="aurora__blob aurora__blob--b" data-speed="-0.08"></span>
                    <span class="aurora__blob aurora__blob--c" data-speed="0.06"></span>
                    <span class="aurora__blob aurora__blob--d" data-speed="0.2"></span>
                </div>
                <div class="hero__grid"></div>
                <FloatingCubes />
                <div class="hero__fade"></div>
            </div>
            <div class="hero__inner">
                <div class="hero__content">
                    <a class="badge intro" style="--d:0ms" href="#packages">
                        <span class="badge__dot"></span>
                        <span>{ t.hero_badge }</span>
                        <Icon name="arrow" />
                    </a>
                    <h1 class="hero__title">{ for lines }</h1>
                    <p class="hero__lead intro" style="--d:1000ms">{ t.hero_lead }</p>
                    <div class="hero__ctas intro" style="--d:1150ms">
                        <Magnetic href={page_href("quick-start", None)} class={classes!("btn", "btn--primary", "btn--lg")} cursor={Some(AttrValue::from("Go"))}>
                            <span>{ t.hero_start }</span><Icon name="arrow" />
                        </Magnetic>
                        <Magnetic href={GITHUB} external=true class={classes!("btn", "btn--ghost", "btn--lg")}>
                            <Icon name="github" /><span>{ t.hero_github }</span>
                        </Magnetic>
                    </div>
                    <div class="install intro" style="--d:1300ms">
                        <span class="install__prompt">{ "$" }</span>
                        <code>{ INSTALL }</code>
                        <CopyButton text={INSTALL} />
                    </div>
                </div>
                <div class="hero__visual intro" style="--d:500ms" role="img" aria-label={t.hero_scene}>
                    <Assembly />
                </div>
            </div>
            <a class="scroll-hint intro" style="--d:1700ms" href="#packages" aria-label={t.scroll_aria}>
                <span class="scroll-hint__mouse"><span></span></span>
                <span>{ t.scroll }</span>
            </a>
        </section>
    }
}

// ----- Marquee -------------------------------------------------------------------------

#[component]
pub fn Marquee() -> Html {
    let t = use_lang().lang.texts();
    let row = |items: &[&str], class: &'static str| {
        let once = html! {
            <>{ for items.iter().map(|w| html! {
                <><span class="marquee__item">{ *w }</span><span class="marquee__sep"><Logo /></span></>
            }) }</>
        };
        html! {
            <div class={classes!("marquee__row", class)}>
                <div class="marquee__track">
                    <div class="marquee__group">{ once.clone() }</div>
                    <div class="marquee__group" aria-hidden="true">{ once }</div>
                </div>
            </div>
        }
    };
    html! {
        <section class="marquee" aria-label={t.marquee_aria}>
            { row(&t.marquee, "marquee__row--a") }
            { row(&MARQUEE_CODE, "marquee__row--b") }
        </section>
    }
}

// ----- Section heading ---------------------------------------------------------------

#[derive(Properties, PartialEq)]
struct HeadProps {
    num: &'static str,
    label: &'static str,
    title: &'static str,
    #[prop_or_default]
    lead: Option<&'static str>,
    #[prop_or_default]
    center: bool,
}

#[component]
fn Head(props: &HeadProps) -> Html {
    html! {
        <div class={classes!("head", props.center.then_some("head--center"))}>
            <p class="eyebrow" data-reveal=""><span class="eyebrow__num">{ props.num }</span>{ props.label }</p>
            <h2 class="head__title" data-reveal="" style="--d:80ms">{ props.title }</h2>
            if let Some(lead) = props.lead {
                <p class="head__lead" data-reveal="" style="--d:160ms">{ lead }</p>
            }
        </div>
    }
}

// ----- 4 packages, 1 framework ---------------------------------------------------------

#[component]
pub fn Packages() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <section class="section" id="packages">
            <Head num="01" label={t.pkg_label} title={t.pkg_title} lead={Some(t.pkg_lead)} />
            <div class="pkgs">
                { for PACKAGES.iter().zip(t.packages.iter()).enumerate().map(|(i, (p, text))| {
                    let (value, unit) = p.figure.split_once(' ').unwrap_or((p.figure, ""));
                    html! {
                        <div class="pkgs__item" data-reveal="" style={format!("--d:{}ms", i * 100)}>
                            <Tilt class={classes!("pkg-card", "card")} style={Some(AttrValue::from(format!("--c:{}", p.color)))}>
                                <div class="pkg-card__top">
                                    <Cube size={46} color={Some(AttrValue::from(p.color))} glyph={Some(AttrValue::from(p.glyph))} spin=true />
                                    <span class="pkg-card__index">{ format!("0{}", i + 1) }</span>
                                </div>
                                <h3 class="pkg-card__name">{ p.name }</h3>
                                <p class="pkg-card__role"><Icon name={p.icon} />{ text.role }</p>
                                <p class="pkg-card__text">{ text.text }</p>
                                <div class="pkg-card__figure">
                                    <strong>{ value }<small>{ unit }</small></strong>
                                    <span>{ text.figure }</span>
                                </div>
                                <a class="pkg-card__link" href={p.repo} target="_blank" rel="noopener">
                                    <Icon name="github" /><span>{ t.pkg_repo }</span><Icon name="external" />
                                </a>
                            </Tilt>
                        </div>
                    }
                }) }
            </div>
            <div class="equation" data-reveal="">
                <code>
                    <span class="equation__term equation__term--brand" style="--i:0">{ "paquets" }</span>
                    <span class="equation__op" style="--i:1">{ "=" }</span>
                    { for PACKAGES.iter().enumerate().map(|(i, p)| html! {
                        <>
                            if i > 0 { <span class="equation__op" style={format!("--i:{}", i * 2 + 1)}>{ "+" }</span> }
                            <span class="equation__term" style={format!("--i:{}; --c:{}", i * 2 + 2, p.color)}>{ p.name }</span>
                        </>
                    }) }
                </code>
                <p>{ t.pkg_alone }</p>
            </div>
        </section>
    }
}

// ----- Code showcase -----------------------------------------------------------------

/// The docs page that goes further than each sample.
const SAMPLE_DOCS: [&str; 5] = ["quick-start", "rest-api", "pages", "islands", "testing"];

#[component]
pub fn Showcase() -> Html {
    let lang = use_lang().lang;
    let t = lang.texts();
    let selected = use_state(|| 0usize);
    let typed = use_state(|| usize::MAX);
    let started = use_state(|| false);
    let window_ref = use_node_ref();
    let timer: Rc<RefCell<Option<Interval>>> = use_mut_ref(|| None);
    {
        let (started, window_ref) = (started.clone(), window_ref.clone());
        use_effect_with((), move |_| {
            let observer = window_ref
                .cast::<HtmlElement>()
                .map(|el| dom::on_first_visible(&el, 0.3, move || started.set(true)));
            move || {
                if let Some(o) = observer {
                    o.disconnect();
                }
            }
        });
    }
    let code = localize(SAMPLES[*selected].code, t.samples[*selected].comments);
    let total = code.chars().count();
    {
        let (typed, timer) = (typed.clone(), timer.clone());
        use_effect_with((*selected, *started, lang), move |&(_, started, _)| {
            if !started || dom::reduced_motion() {
                typed.set(usize::MAX);
            } else {
                let mut n = 0usize;
                typed.set(0);
                *timer.borrow_mut() = Some(Interval::new(16, move || {
                    if n < total {
                        n = (n + 7).min(total);
                        typed.set(n);
                    }
                }));
            }
            move || {
                timer.borrow_mut().take();
            }
        });
    }
    let sample = &SAMPLES[*selected];
    let text = &t.samples[*selected];
    let done = *typed >= total;
    let limit = if done { usize::MAX } else { *typed };
    html! {
        <section class="section showcase" id="code">
            <Head num="02" label={t.code_label} title={t.code_title} lead={Some(t.code_lead)} />
            <div class="tabs" role="tablist" data-reveal="" style={format!("--tab:{}; --tabs:{}", *selected, SAMPLES.len())}>
                <span class="tabs__indicator" aria-hidden="true"></span>
                { for t.samples.iter().enumerate().map(|(i, s)| {
                    let selected = selected.clone();
                    html! {
                        <button role="tab" aria-selected={(i == *selected).to_string()}
                                class={classes!("tabs__tab", (i == *selected).then_some("is-active"))}
                                onclick={Callback::from(move |_: MouseEvent| selected.set(i))}>
                            { s.label }
                        </button>
                    }
                }) }
            </div>
            <div class="showcase__grid" data-reveal="" style="--d:120ms">
                <aside class="showcase__aside" key={format!("{}-{}", *selected, lang.code())}>
                    <span class="showcase__num">{ format!("0{}", *selected + 1) }<small>{ format!(" / 0{}", SAMPLES.len()) }</small></span>
                    <h3>{ text.label }</h3>
                    <p>{ text.caption }</p>
                    <a class="link-arrow" href={page_href(SAMPLE_DOCS[*selected], None)}>
                        <span>{ t.docs_browse }</span><Icon name="arrow" />
                    </a>
                </aside>
                <div class="window window--glow" ref={window_ref}>
                    <div class="window__bar">
                        <span></span><span></span><span></span>
                        <em>{ sample.file }</em>
                        <CopyButton text={code.clone()} />
                    </div>
                    <Code code={code} limit={limit} caret={!done} />
                </div>
            </div>
        </section>
    }
}

// ----- Features -------------------------------------------------------------------------

#[component]
pub fn Features() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <section class="section" id="features">
            <Head num="03" label={t.features_label} title={t.features_title} lead={Some(t.features_lead)} />
            <div class="features">
                { for FEATURES.iter().zip(t.features.iter()).enumerate().map(|(i, ((icon, tag), f))| html! {
                    <div class="features__item" data-reveal="" style={format!("--d:{}ms", (i % 4) * 90)}>
                        <Tilt class={classes!("feature", "card")}>
                            <span class="feature__icon"><Icon name={*icon} /></span>
                            <h3>{ f.title }</h3>
                            <p>{ f.text }</p>
                            <code class="feature__tag">{ *tag }</code>
                        </Tilt>
                    </div>
                }) }
            </div>
        </section>
    }
}

// ----- Numbers ----------------------------------------------------------------------------

#[component]
pub fn Numbers() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <section class="section numbers" id="numbers">
            <div class="numbers__bg" aria-hidden="true"></div>
            <Head num="04" label={t.numbers_label} title={t.numbers_title} />
            <div class="stats">
                { for NUMBERS.iter().zip(t.numbers.iter()).enumerate().map(|(i, ((value, decimals, prefix, suffix), item))| html! {
                    <article class="stat" data-reveal="" style={format!("--d:{}ms", (i % 3) * 110)}>
                        <div class="stat__value">
                            <Counter value={*value} decimals={*decimals} prefix={*prefix} suffix={*suffix} />
                        </div>
                        <h3 class="stat__label">{ item.title }</h3>
                        <p class="stat__detail">{ item.text }</p>
                        <span class="stat__line"></span>
                    </article>
                }) }
            </div>
        </section>
    }
}

// ----- From zero to production ------------------------------------------------------------

#[component]
pub fn Journey() -> Html {
    let t = use_lang().lang.texts();
    let step = use_state(|| None::<usize>);
    let on_step = {
        let step = step.clone();
        Callback::from(move |s: Option<usize>| step.set(s))
    };
    let icons = ["box", "terminal", "git", "cloud"];
    html! {
        <section class="section journey" id="start">
            <Head num="05" label={t.journey_label} title={t.journey_title} lead={Some(t.journey_lead)} />
            <div class="journey__grid">
                <ol class="timeline" id="journey">
                    <span class="timeline__rail" aria-hidden="true"><span class="timeline__fill"></span></span>
                    <span class="timeline__traveler" aria-hidden="true"><Cube size={26} spin=true /></span>
                    { for t.journey.iter().zip(JOURNEY.iter()).enumerate().map(|(i, (item, command))| html! {
                        // The reveal is on an inner element: Yew rewrites the class
                        // of the step when it becomes active, which would remove `in`.
                        <li class={classes!("timeline__step", (*step == Some(i)).then_some("is-active"), step.is_some_and(|s| s > i).then_some("is-done"))}>
                            <div class="timeline__inner" data-reveal="" style={format!("--d:{}ms", i * 90)}>
                                <span class="timeline__dot"><Icon name={icons[i]} /></span>
                                <div class="timeline__body">
                                    <span class="timeline__num">{ format!("0{}", i + 1) }</span>
                                    <h3>{ item.title }</h3>
                                    <p>{ item.text }</p>
                                    <code class="timeline__cmd">{ *command }</code>
                                </div>
                            </div>
                        </li>
                    }) }
                </ol>
                <div class="journey__terminal" data-reveal="" style="--d:150ms">
                    <Terminal {on_step} />
                </div>
            </div>
        </section>
    }
}

// ----- Deploy from your phone ---------------------------------------------------------------

#[component]
fn Phone() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <div class="phone" aria-hidden="true">
            <div class="phone__frame">
                <div class="phone__notch"></div>
                <div class="phone__screen">
                    <div class="phone__status"><span>{ "9:41" }</span><span class="phone__bars"><i></i><i></i><i></i></span></div>
                    <div class="phone__view phone__view--1">
                        <div class="phone__app"><span class="phone__heroku">{ "H" }</span><b>{ t.phone[0] }</b></div>
                        <div class="phone__field"><small>{ t.phone[5] }</small><span class="phone__typed">{ t.phone[1] }</span></div>
                        <div class="phone__field"><small>{ t.phone[6] }</small><span>{ t.phone[7] }</span></div>
                        <div class="phone__btn"><span class="phone__tap"></span>{ t.phone[0] }</div>
                    </div>
                    <div class="phone__view phone__view--2">
                        <p class="phone__title">{ t.phone[2] }</p>
                        <div class="phone__log">
                            <span>{ "Step 1/6 : FROM rust:slim" }</span>
                            <span>{ "Compiling paquets v0.1.0" }</span>
                            <span>{ "Compiling my-app v0.1.0" }</span>
                            <span>{ "Finished release" }</span>
                            <span>{ "Launching web.1" }</span>
                        </div>
                        <div class="phone__progress"><span></span></div>
                    </div>
                    <div class="phone__view phone__view--3">
                        <div class="phone__check"><Icon name="check" /></div>
                        <p class="phone__title">{ t.phone[3] }</p>
                        <p class="phone__url">{ format!("{}.herokuapp.com", t.phone[1]) }</p>
                        <div class="phone__btn phone__btn--ghost">{ t.phone[4] }</div>
                    </div>
                </div>
            </div>
            <div class="phone__glow"></div>
        </div>
    }
}

#[component]
pub fn Deploy() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <section class="section deploy" id="deploy">
            <Head num="06" label={t.deploy_label} title={t.deploy_title} lead={Some(t.deploy_lead)} />
            <div class="deploy__grid">
                <div class="deploy__visual" data-reveal="">
                    <Phone />
                </div>
                <div class="deploy__content">
                    <ol class="deploy__steps">
                        { for t.deploy_steps.iter().enumerate().map(|(i, step)| html! {
                            <li data-reveal="" style={format!("--d:{}ms", i * 110)}>
                                <span class="deploy__num">{ i + 1 }</span>
                                <div>
                                    <h3>{ step.title }</h3>
                                    <p>{ step.text }</p>
                                </div>
                            </li>
                        }) }
                    </ol>
                    <div class="deploy__secrets" data-reveal="" style="--d:200ms">
                        <code>{ "HEROKU_API_KEY" }</code>
                        <code>{ "HEROKU_APP_NAME" }</code>
                        <code>{ "git push → master" }</code>
                    </div>
                    <div class="deploy__actions" data-reveal="" style="--d:260ms">
                        <a class="heroku-button" href={HEROKU_DEPLOY} target="_blank" rel="noopener">
                            <img src={HEROKU_BUTTON} alt={t.deploy_button_alt} width="147" height="32" loading="lazy" />
                        </a>
                        <a class="link-arrow" href={page_href("deploy-heroku-mobile", None)}>
                            <span>{ t.deploy_guide }</span><Icon name="arrow" />
                        </a>
                    </div>
                    <p class="deploy__note" data-reveal="" style="--d:300ms">{ t.deploy_note }</p>
                </div>
            </div>
        </section>
    }
}

// ----- Docs preview ------------------------------------------------------------------------------

#[component]
pub fn DocsPreview() -> Html {
    let lang = use_lang().lang;
    let t = lang.texts();
    html! {
        <section class="section" id="docs">
            <Head num="07" label={t.docs_label} title={t.docs_home_title} lead={Some(t.docs_home_lead)} />
            <div class="doc-preview">
                { for CATEGORIES.iter().enumerate().map(|(c, category)| html! {
                    <div class="doc-preview__item" data-reveal="" style={format!("--d:{}ms", c * 80)}>
                        <Tilt class={classes!("doc-preview__card", "card")} max={6.0}>
                            <span class="doc-preview__num">{ format!("0{}", c + 1) }</span>
                            <span class="doc-preview__icon"><Icon name={category.icon} /></span>
                            <h3>{ lang.pick(category.title) }</h3>
                            <ul>
                                { for pages_of(c).map(|page| html! {
                                    <li><a href={page_href(page.slug, None)}>{ page.title(lang) }</a></li>
                                }) }
                            </ul>
                        </Tilt>
                    </div>
                }) }
            </div>
            <div class="center" data-reveal="">
                <Magnetic href="#/docs" class={classes!("btn", "btn--ghost", "btn--lg")}>
                    <Icon name="book" /><span>{ t.docs_browse }</span>
                </Magnetic>
            </div>
        </section>
    }
}

// ----- Call to action ---------------------------------------------------------------------------

#[component]
pub fn Cta() -> Html {
    let t = use_lang().lang.texts();
    html! {
        <section class="cta">
            <div class="cta__bg" aria-hidden="true"></div>
            <div class="cta__cube" aria-hidden="true" data-speed="-0.06"><Cube size={120} spin=true /></div>
            <h2 class="cta__title" data-reveal="">
                <span>{ t.cta_title[0] }</span>
                <span class="cta__outline" data-text={t.cta_title[1]}>{ t.cta_title[1] }</span>
            </h2>
            <div class="install install--center" data-reveal="" style="--d:120ms">
                <span class="install__prompt">{ "$" }</span>
                <code>{ INSTALL }</code>
                <CopyButton text={INSTALL} />
            </div>
            <div class="cta__actions" data-reveal="" style="--d:200ms">
                <Magnetic href={page_href("quick-start", None)} class={classes!("btn", "btn--primary", "btn--lg")} cursor={Some(AttrValue::from("Go"))}>
                    <span>{ t.cta_start }</span><Icon name="arrow" />
                </Magnetic>
                <Magnetic href={GITHUB} external=true class={classes!("btn", "btn--ghost", "btn--lg")}>
                    <Icon name="github" /><span>{ t.cta_star }</span>
                </Magnetic>
            </div>
        </section>
    }
}
