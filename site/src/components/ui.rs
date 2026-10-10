//! Small animated components used everywhere.

use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::callback::{Interval, Timeout};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::dom::{self, RafLoop};
use crate::highlight::{self, Kind};
use crate::i18n::{self, use_lang};

// ----- Magnetic button -------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct MagneticProps {
    pub href: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or(0.35)]
    pub strength: f64,
    #[prop_or_default]
    pub external: bool,
    #[prop_or_default]
    pub cursor: Option<AttrValue>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

fn current(e: &MouseEvent) -> Option<HtmlElement> {
    e.current_target()
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
}

/// A link attracted by the cursor.
#[component]
pub fn Magnetic(props: &MagneticProps) -> Html {
    let strength = props.strength;
    let onmousemove = Callback::from(move |e: MouseEvent| {
        let Some(el) = current(&e) else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        let x = f64::from(e.client_x()) - rect.left() - rect.width() / 2.0;
        let y = f64::from(e.client_y()) - rect.top() - rect.height() / 2.0;
        dom::set_var(&el, "--mx", &format!("{:.1}px", x * strength));
        dom::set_var(&el, "--my", &format!("{:.1}px", y * strength));
    });
    let onmouseleave = Callback::from(|e: MouseEvent| {
        let Some(el) = current(&e) else {
            return;
        };
        dom::set_var(&el, "--mx", "0px");
        dom::set_var(&el, "--my", "0px");
    });
    let (target, rel) = if props.external {
        (Some("_blank"), Some("noopener"))
    } else {
        (None, None)
    };
    html! {
        <a class={classes!("magnetic", props.class.clone())} href={props.href.clone()}
           target={target} rel={rel} data-cursor={props.cursor.clone()} aria-label={props.label.clone()}
           {onmousemove} {onmouseleave}>
            <span class="magnetic__inner">{ props.children.clone() }</span>
        </a>
    }
}

// ----- 3D tilt card ----------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct TiltProps {
    #[prop_or_default]
    pub class: Classes,
    /// Maximum angle, in degrees.
    #[prop_or(10.0)]
    pub max: f64,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A card that tilts in 3D towards the cursor, with a light that follows it.
#[component]
pub fn Tilt(props: &TiltProps) -> Html {
    let max = props.max;
    let onmousemove = Callback::from(move |e: MouseEvent| {
        let Some(el) = current(&e) else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        let x = (f64::from(e.client_x()) - rect.left()) / rect.width();
        let y = (f64::from(e.client_y()) - rect.top()) / rect.height();
        dom::set_var(&el, "--px", &format!("{:.1}%", x * 100.0));
        dom::set_var(&el, "--py", &format!("{:.1}%", y * 100.0));
        dom::set_var(&el, "--ry", &format!("{:.2}deg", (x - 0.5) * max));
        dom::set_var(&el, "--rx", &format!("{:.2}deg", (0.5 - y) * max));
    });
    let onmouseleave = Callback::from(|e: MouseEvent| {
        let Some(el) = current(&e) else {
            return;
        };
        dom::set_var(&el, "--rx", "0deg");
        dom::set_var(&el, "--ry", "0deg");
    });
    html! {
        <div class={classes!("tilt", props.class.clone())} style={props.style.clone()} {onmousemove} {onmouseleave}>
            <span class="tilt__spot" aria-hidden="true"></span>
            { props.children.clone() }
        </div>
    }
}

// ----- Scrambled text on hover -------------------------------------------------

const GLYPHS: &[char] = &[
    '#', '%', '&', '*', '+', '=', '/', '<', '>', '?', '0', '1', '▣', '_', '◆',
];

#[derive(Properties, PartialEq)]
pub struct ScrambleProps {
    pub text: AttrValue,
}

/// A text whose letters shuffle, then fall back into place, on hover.
#[component]
pub fn Scramble(props: &ScrambleProps) -> Html {
    let shown = use_state(|| props.text.to_string());
    let timer: Rc<RefCell<Option<Interval>>> = use_mut_ref(|| None);
    {
        // The text changes with the language.
        let (shown, timer) = (shown.clone(), timer.clone());
        use_effect_with(props.text.clone(), move |text| {
            timer.borrow_mut().take();
            shown.set(text.to_string());
        });
    }
    let onmouseenter = {
        let shown = shown.clone();
        let timer = timer.clone();
        let text: Vec<char> = props.text.chars().collect();
        Callback::from(move |_: MouseEvent| {
            if dom::reduced_motion() || timer.borrow().is_some() {
                return;
            }
            let shown = shown.clone();
            let text = text.clone();
            let mut frame = 0usize;
            let total = text.len() * 2 + 4;
            let interval = Interval::new(28, move || {
                frame += 1;
                let revealed = frame.saturating_sub(4) / 2;
                let s: String = text
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        if i < revealed || c == ' ' || frame >= total {
                            c
                        } else {
                            GLYPHS[(dom::random() * GLYPHS.len() as f64) as usize % GLYPHS.len()]
                        }
                    })
                    .collect();
                shown.set(s);
            });
            let stop = timer.clone();
            *timer.borrow_mut() = Some(interval);
            Timeout::new((total as u32 + 1) * 28, move || {
                stop.borrow_mut().take();
            })
            .forget();
        })
    };
    html! {
        <span class="scramble" aria-label={props.text.clone()} {onmouseenter}>
            <span aria-hidden="true">{ (*shown).clone() }</span>
        </span>
    }
}

// ----- Animated counter --------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct CounterProps {
    pub value: f64,
    #[prop_or_default]
    pub decimals: usize,
    #[prop_or_default]
    pub prefix: AttrValue,
    #[prop_or_default]
    pub suffix: AttrValue,
    #[prop_or(1800.0)]
    pub duration: f64,
}

/// A number that rolls up to its value when it enters the screen.
#[component]
pub fn Counter(props: &CounterProps) -> Html {
    let lang = use_lang().lang;
    let node = use_node_ref();
    let anim: Rc<RefCell<Option<RafLoop>>> = use_mut_ref(|| None);
    {
        let node = node.clone();
        let (value, decimals, duration) = (props.value, props.decimals, props.duration);
        let (prefix, suffix) = (props.prefix.to_string(), props.suffix.to_string());
        use_effect_with((value.to_bits(), lang), move |_| {
            let mut observer = None;
            if let Some(el) = node.cast::<HtmlElement>() {
                let render = move |el: &HtmlElement, v: f64| {
                    el.set_text_content(Some(&format!(
                        "{prefix}{}{suffix}",
                        i18n::number(lang, v, decimals)
                    )));
                };
                if dom::reduced_motion() {
                    render(&el, value);
                } else {
                    render(&el, 0.0);
                    let target = el.clone();
                    observer = Some(dom::on_first_visible(&el, 0.4, move || {
                        let mut start = None;
                        *anim.borrow_mut() = Some(dom::raf_loop(move |t| {
                            let t0 = *start.get_or_insert(t);
                            let p = ((t - t0) / duration).min(1.0);
                            // Exponential ease-out.
                            let eased = if p >= 1.0 {
                                1.0
                            } else {
                                1.0 - 2f64.powf(-10.0 * p)
                            };
                            render(&target, value * eased);
                            p < 1.0
                        }));
                    }));
                }
            }
            move || {
                if let Some(o) = observer {
                    o.disconnect();
                }
            }
        });
    }
    html! { <span class="counter" ref={node}>{ format!("{}{}{}", props.prefix, i18n::number(lang, props.value, props.decimals), props.suffix) }</span> }
}

// ----- Copy button ---------------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct CopyProps {
    pub text: AttrValue,
}

#[component]
pub fn CopyButton(props: &CopyProps) -> Html {
    let t = use_lang().lang.texts();
    let copied = use_state(|| false);
    let onclick = {
        let copied = copied.clone();
        let text = props.text.clone();
        Callback::from(move |_: MouseEvent| {
            let _ = dom::window().navigator().clipboard().write_text(&text);
            copied.set(true);
            let copied = copied.clone();
            Timeout::new(1600, move || copied.set(false)).forget();
        })
    };
    html! {
        <button class={classes!("copy", copied.then_some("copy--done"))} {onclick}
                data-cursor={t.copy} aria-label={t.copy_aria}>
            <Icon name={if *copied { "check" } else { "copy" }} />
            <span>{ if *copied { t.copied } else { t.copy } }</span>
        </button>
    }
}

// ----- Text split into letters ----------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct SplitProps {
    pub text: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    /// Offset (in letters) to chain several lines.
    #[prop_or_default]
    pub offset: usize,
}

/// Each letter rises in turn (the animation is in CSS).
#[component]
pub fn Split(props: &SplitProps) -> Html {
    let words: Vec<&str> = props.text.split(' ').collect();
    let mut index = props.offset;
    html! {
        <span class={classes!("split", props.class.clone())} aria-label={props.text.clone()} role="text">
            { for words.iter().enumerate().map(|(w, word)| {
                let chars: Vec<Html> = word.chars().map(|c| {
                    let i = index;
                    index += 1;
                    html! { <span class="ch" aria-hidden="true" style={format!("--i:{i}")}>{ c }</span> }
                }).collect();
                index += 1;
                html! {
                    <>
                        <span class="word" aria-hidden="true">{ for chars }</span>
                        if w + 1 < words.len() { { " " } }
                    </>
                }
            }) }
        </span>
    }
}

// ----- Icons ------------------------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct IconProps {
    pub name: AttrValue,
}

/// The stroke paths of an icon (24×24).
pub fn icon_paths(name: &str) -> &'static [&'static str] {
    match name {
        "bolt" => &["M13 2 4 14h7l-1 8 9-12h-7z"],
        "shield" => &[
            "M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6z",
            "m9 12 2 2 4-4",
        ],
        "db" => &[
            "M4 6c0-1.7 3.6-3 8-3s8 1.3 8 3-3.6 3-8 3-8-1.3-8-3z",
            "M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6",
            "M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
        ],
        "spark" => &[
            "M12 3c.8 4.5 2.5 6.2 7 7-4.5.8-6.2 2.5-7 7-.8-4.5-2.5-6.2-7-7 4.5-.8 6.2-2.5 7-7z",
            "M19 15v5",
            "M16.5 17.5h5",
        ],
        "box" => &[
            "M12 2.5 3.5 7v10l8.5 4.5 8.5-4.5V7z",
            "m3.5 7 8.5 4.5L20.5 7",
            "M12 11.5v10",
            "m7.8 4.7 8.5 4.6",
        ],
        "page" => &[
            "M14 3H6v18h12V7z",
            "M14 3v4h4",
            "M9 12h6",
            "M9 16h6",
        ],
        "phone" => &[
            "M8 2h8a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z",
            "M11 18h2",
        ],
        "flask" => &["M9 3h6", "M10 3v6L4 20h16L14 9V3", "M7 15h10"],
        "crate" => &["M3 4h18v4H3z", "M5 8v12h14V8", "M10 12h4"],
        "rocket" => &[
            "M5 16c-1.5 1.2-2 5-2 5s3.8-.5 5-2c.7-.8.7-2.1-.1-2.9A2.2 2.2 0 0 0 5 16z",
            "m12 15-3-3a22 22 0 0 1 2-4A13 13 0 0 1 22 2c0 2.7-.8 7.5-6 11a22 22 0 0 1-4 2z",
            "M9 12H4s.6-3 2-4c1.6-1.1 5 0 5 0",
            "M12 15v5s3-.6 4-2c1.1-1.6 0-5 0-5",
        ],
        "layers" => &["m12 3 9 5-9 5-9-5z", "m3 13 9 5 9-5"],
        "cloud" => &["M7 19h10a4.5 4.5 0 0 0 .6-9A6.5 6.5 0 0 0 5.2 10.4 4.3 4.3 0 0 0 7 19z"],
        "book" => &[
            "M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z",
            "M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5",
            "M8 7h8",
        ],
        "github" => &[
            "M9 19c-5 1.5-5-2.5-7-3m14 6v-3.9a3.4 3.4 0 0 0-.9-2.6c3-.3 6.1-1.5 6.1-6.6a5.1 5.1 0 0 0-1.4-3.6 4.8 4.8 0 0 0-.1-3.5s-1.1-.3-3.6 1.4a12.3 12.3 0 0 0-6.4 0C6.2 1.6 5.1 1.9 5.1 1.9a4.8 4.8 0 0 0-.1 3.5A5.1 5.1 0 0 0 3.6 9c0 5.1 3.1 6.3 6.1 6.6a3.4 3.4 0 0 0-.9 2.6V22",
        ],
        "git" => &[
            "M6 3v12",
            "M18 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            "M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
            "M18 9a9 9 0 0 1-9 9",
        ],
        "terminal" => &["m4 17 6-6-6-6", "M12 19h8"],
        "server" => &[
            "M4 4h16v6H4z",
            "M4 14h16v6H4z",
            "M8 7h.01",
            "M8 17h.01",
        ],
        "external" => &["M14 4h6v6", "M20 4 10 14", "M18 14v6H4V6h6"],
        "arrow" => &["M5 12h14", "m13 6 6 6-6 6"],
        "up" => &["M12 19V5", "m6 11 6-6 6 6"],
        "search" => &["M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z", "m20 20-4.2-4.2"],
        "edit" => &["M4 20h4L19 9l-4-4L4 16z", "m13.5 6.5 4 4"],
        "menu" => &["M4 7h16", "M4 12h16", "M4 17h10"],
        "copy" => &["M9 9h11v11H9z", "M5 15H4V4h11v1"],
        "check" => &["m5 12 5 5L20 7"],
        "replay" => &["M3 12a9 9 0 1 0 3-6.7", "M3 4v5h5"],
        "globe" => &[
            "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z",
            "M3 12h18",
            "M12 3c2.5 2.5 3.8 5.5 3.8 9s-1.3 6.5-3.8 9c-2.5-2.5-3.8-5.5-3.8-9S9.5 5.5 12 3z",
        ],
        _ => &[],
    }
}

#[component]
pub fn Icon(props: &IconProps) -> Html {
    html! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"
             stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            { for icon_paths(&props.name).iter().map(|d| html! { <path d={*d} /> }) }
        </svg>
    }
}

/// The logo: the Paquets box (the same drawing as `assets/logo.svg`).
#[component]
pub fn Logo() -> Html {
    html! {
        <svg class="logo-mark" viewBox="0 0 64 64" aria-hidden="true">
            <defs>
                <linearGradient id="lg-top" x1="8" y1="18" x2="56" y2="18" gradientUnits="userSpaceOnUse">
                    <stop offset="0" stop-color="#8f74ff" /><stop offset="1" stop-color="#22d3ee" />
                </linearGradient>
                <linearGradient id="lg-left" x1="8" y1="18" x2="32" y2="60" gradientUnits="userSpaceOnUse">
                    <stop offset="0" stop-color="#6d4aff" /><stop offset="1" stop-color="#3a1fb8" />
                </linearGradient>
                <linearGradient id="lg-right" x1="32" y1="32" x2="56" y2="56" gradientUnits="userSpaceOnUse">
                    <stop offset="0" stop-color="#ff4fa3" /><stop offset="1" stop-color="#b72a7c" />
                </linearGradient>
            </defs>
            <g stroke-linejoin="round" stroke-width="2">
                <path class="logo-mark__top" d="M32 4 56 18 32 32 8 18Z" fill="url(#lg-top)" stroke="url(#lg-top)" />
                <path d="M8 18 32 32 32 60 8 46Z" fill="url(#lg-left)" stroke="url(#lg-left)" />
                <path d="M32 32 56 18 56 46 32 60Z" fill="url(#lg-right)" stroke="url(#lg-right)" />
            </g>
            <path d="M17.4 12.5 22.6 9.5 46.6 23.5 41.4 26.5Z" fill="#fff" fill-opacity=".9" />
            <path d="M41.4 26.5 46.6 23.5 46.6 51.5 41.4 54.5Z" fill="#fff" fill-opacity=".55" />
        </svg>
    }
}

// ----- Code block -----------------------------------------------------------------------

#[derive(Properties, PartialEq)]
pub struct CodeProps {
    pub code: AttrValue,
    #[prop_or(AttrValue::Static("rust"))]
    pub language: AttrValue,
    /// Number of characters shown (typewriter effect).
    #[prop_or(usize::MAX)]
    pub limit: usize,
    #[prop_or_default]
    pub caret: bool,
}

#[component]
pub fn Code(props: &CodeProps) -> Html {
    let tokens = use_memo((props.code.clone(), props.language.clone()), |(code, lang)| {
        highlight::highlight(code, lang).unwrap_or_else(|| vec![(Kind::Plain, code.to_string())])
    });
    let shown = if props.limit == usize::MAX {
        (*tokens).clone()
    } else {
        highlight::truncate(&tokens, props.limit)
    };
    html! {
        <pre class="code"><code>
            { for shown.into_iter().map(|(kind, text)| {
                if kind == Kind::Plain { html! { { text } } }
                else { html! { <span class={kind.class()}>{ text }</span> } }
            }) }
            if props.caret { <span class="caret" aria-hidden="true"></span> }
        </code></pre>
    }
}
