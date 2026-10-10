//! Page motion that CSS cannot do alone: scroll reveals, progress bar, navigation that
//! hides, parallax, counters, the cursor, magnetic buttons, 3D tilt, menus, copy
//! buttons and the "on this page" scroll-spy. Everything degrades to a static page.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};

use super::{all, document, later, matches, on, on_passive, reduced_motion, request_frame, window};
use crate::i18n::{Lang, number};

pub fn init() {
    let root: HtmlElement = document().document_element().unwrap().unchecked_into();
    intro(&root);
    reveals();
    // Only now may the CSS hide what is not revealed yet: what is already on screen
    // was marked just above, so nothing blinks.
    let _ = root.class_list().add_1("hydrated");
    scroll_state();
    counters();
    copy_buttons();
    toggles(&root);
    toc();
    if matches("(hover: hover) and (pointer: fine)") && !reduced_motion() {
        let _ = root.class_list().add_1("has-cursor");
        cursor();
        magnetic();
        tilt();
        hero_mouse();
    }
}

fn observe(
    selector: &str,
    options: &IntersectionObserverInit,
    f: impl Fn(&IntersectionObserverEntry, &IntersectionObserver) + 'static,
) {
    let callback = Closure::<dyn Fn(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, io: IntersectionObserver| {
            for entry in entries.iter() {
                if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() {
                    f(&entry, &io);
                }
            }
        },
    );
    if let Ok(io) =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), options)
    {
        for el in all(selector) {
            io.observe(&el);
        }
    }
    callback.forget();
}

fn viewport_height() -> f64 {
    window()
        .inner_height()
        .ok()
        .and_then(|h| h.as_f64())
        .unwrap_or(800.0)
}

/// The preloader plays once per browser session: on the next home pages it gets
/// out of the way at once. A click also skips it.
fn intro(root: &HtmlElement) {
    if all(".loader").is_empty() {
        return;
    }
    let storage = window().session_storage().ok().flatten();
    let seen = storage
        .as_ref()
        .and_then(|s| s.get_item("paquets-intro").ok().flatten())
        .is_some();
    if seen || reduced_motion() {
        let _ = root.class_list().add_1("intro-seen");
    } else if let Some(storage) = storage {
        let _ = storage.set_item("paquets-intro", "1");
    }
    for loader in all(".loader") {
        let root = root.clone();
        on(&loader, "click", move |_: web_sys::Event| {
            let _ = root.class_list().add_1("intro-seen");
        });
    }
}

/// `[data-reveal]` elements fade in when they scroll into view.
fn reveals() {
    let height = viewport_height();
    for el in all("[data-reveal]") {
        let rect = el.get_bounding_client_rect();
        if rect.top() < height * 0.92 && rect.bottom() > 0.0 {
            let _ = el.class_list().add_1("in");
        }
    }
    let options = IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(0.0));
    options.set_root_margin("0px 0px -8% 0px");
    observe("[data-reveal]:not(.in)", &options, |entry, io| {
        if entry.is_intersecting() {
            let target = entry.target();
            let _ = target.class_list().add_1("in");
            io.unobserve(&target);
        }
    });
}

fn set(el: &HtmlElement, name: &str, value: &str) {
    let _ = el.style().set_property(name, value);
}

/// Progress bar, navigation style and visibility, parallax, and the progress of the
/// timeline and of the hero.
fn scroll_state() {
    let progress = all(".progress");
    let nav = all("#nav");
    let journey = all("#journey");
    let hero = all(".hero");
    let still = reduced_motion();
    let parallax: Vec<(HtmlElement, f64)> = if still {
        Vec::new()
    } else {
        all("[data-speed]")
            .into_iter()
            .map(|el| {
                let speed = el
                    .get_attribute("data-speed")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                (el, speed)
            })
            .collect()
    };
    let last = Rc::new(Cell::new(0.0f64));
    let update = move || {
        let y = window().scroll_y().unwrap_or(0.0);
        let height = viewport_height();
        let root = document().document_element().unwrap();
        let max = (f64::from(root.scroll_height()) - height).max(1.0);
        for bar in &progress {
            set(
                bar,
                "transform",
                &format!("scaleX({:.4})", (y / max).clamp(0.0, 1.0)),
            );
        }
        let menu_open = root.class_list().contains("menu-open");
        for nav in &nav {
            let list = nav.class_list();
            let _ = list.toggle_with_force("nav--scrolled", y > 30.0);
            if y > last.get() + 2.0 && y > 500.0 && !menu_open {
                let _ = list.add_1("nav--hidden");
            } else if y < last.get() - 2.0 {
                let _ = list.remove_1("nav--hidden");
            }
        }
        last.set(y);
        for (el, speed) in &parallax {
            set(
                el,
                "transform",
                &format!("translate3d(0,{:.1}px,0)", y * speed),
            );
        }
        for el in &journey {
            let rect = el.get_bounding_client_rect();
            let p = ((height * 0.6 - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
            set(el, "--p", &format!("{p:.4}"));
        }
        if !still {
            for el in &hero {
                set(
                    el,
                    "--scroll",
                    &format!("{:.4}", (y / height).clamp(0.0, 1.0)),
                );
            }
        }
    };
    update();
    let update = Rc::new(update);
    let ticking = Rc::new(Cell::new(false));
    for event in ["scroll", "resize"] {
        let (update, ticking) = (update.clone(), ticking.clone());
        on_passive(&window(), event, move |_: web_sys::Event| {
            if ticking.replace(true) {
                return;
            }
            let (update, ticking) = (update.clone(), ticking.clone());
            request_frame(move |_| {
                ticking.set(false);
                update();
                false
            });
        });
    }
}

/// Numbers count up when they come into view (`data-count`, `data-decimals`,
/// `data-prefix`, `data-suffix`), formatted for the language of the page.
fn counters() {
    let lang = document()
        .document_element()
        .and_then(|h| h.get_attribute("lang"))
        .and_then(|l| Lang::parse(&l))
        .unwrap_or(Lang::En);
    let options = IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(0.4));
    observe("[data-count]", &options, move |entry, io| {
        if !entry.is_intersecting() {
            return;
        }
        let el: HtmlElement = entry.target().unchecked_into();
        io.unobserve(&el);
        let data = el.dataset();
        let target: f64 = data
            .get("count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let decimals: usize = data
            .get("decimals")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let prefix = data.get("prefix").unwrap_or_default();
        let suffix = data.get("suffix").unwrap_or_default();
        let render = move |v: f64| format!("{prefix}{}{suffix}", number(lang, v, decimals));
        if reduced_motion() {
            el.set_text_content(Some(&render(target)));
            return;
        }
        let mut start = None;
        request_frame(move |now| {
            let t0 = *start.get_or_insert(now);
            let p = ((now - t0) / 1800.0).min(1.0);
            let eased = if p >= 1.0 {
                1.0
            } else {
                1.0 - 2f64.powf(-10.0 * p)
            };
            el.set_text_content(Some(&render(target * eased)));
            p < 1.0
        });
    });
}

/// "Copy" buttons: the text of `data-copy`, or of the visible code block of the
/// nearest `[data-copy-root]`.
fn copy_buttons() {
    on(&document(), "click", |e: web_sys::MouseEvent| {
        let Some(button) = e
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .and_then(|t| t.closest(".copy").ok().flatten())
        else {
            return;
        };
        let text = button.get_attribute("data-copy").or_else(|| {
            let root = button.closest("[data-copy-root]").ok().flatten()?;
            root.query_selector(".panel:not([hidden]) pre, pre")
                .ok()
                .flatten()?
                .text_content()
        });
        let Some(text) = text else { return };
        let promise = window().navigator().clipboard().write_text(&text);
        wasm_bindgen_futures::spawn_local(async move {
            if wasm_bindgen_futures::JsFuture::from(promise).await.is_err() {
                return;
            }
            let label = button
                .query_selector("span")
                .ok()
                .flatten()
                .unwrap_or_else(|| button.clone());
            let before = label.text_content();
            if let Some(done) = button.get_attribute("data-copied") {
                label.set_text_content(Some(&done));
            }
            let _ = button.class_list().add_1("copy--done");
            later(1500, move || {
                label.set_text_content(before.as_deref());
                let _ = button.class_list().remove_1("copy--done");
            });
        });
    });
}

/// The mobile menu, the docs sidebar, and the big search field of the docs hub.
fn toggles(root: &HtmlElement) {
    let pairs = [
        ("[data-toggle-menu]", "menu-open"),
        ("[data-toggle-side]", "side-open"),
    ];
    for (selector, class) in pairs {
        for button in all(selector) {
            let (root, target) = (root.clone(), button.clone());
            on(&button, "click", move |_: web_sys::Event| {
                let open = root.class_list().toggle(class).unwrap_or(false);
                let _ = target.set_attribute("aria-expanded", if open { "true" } else { "false" });
            });
        }
    }
    // A link of the menu or of the sidebar, or the scrim, closes them.
    for el in all(".menu a, .doc-side a, .docs__scrim") {
        let root = root.clone();
        on(&el, "click", move |_: web_sys::Event| {
            let _ = root.class_list().remove_2("menu-open", "side-open");
        });
    }
    let root_key = root.clone();
    on(&window(), "keydown", move |e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            let _ = root_key.class_list().remove_2("menu-open", "side-open");
        }
    });
    for el in all("[data-search-open]") {
        on(&el, "click", |_: web_sys::Event| {
            if let Some(button) = all("[data-search-button]").into_iter().next() {
                button.click();
            }
        });
    }
}

/// "On this page": highlights the section being read.
fn toc() {
    let links = all(".doc-toc a");
    if links.is_empty() {
        return;
    }
    let options = IntersectionObserverInit::new();
    options.set_root_margin("-15% 0px -70% 0px");
    observe(
        ".doc-body h2[id], .doc-body h3[id]",
        &options,
        move |entry, _| {
            if !entry.is_intersecting() {
                return;
            }
            let id = entry.target().id();
            for link in &links {
                let active = link.get_attribute("href").as_deref() == Some(&format!("#{id}"));
                let _ = link.class_list().toggle_with_force("is-active", active);
            }
        },
    );
}

/// A dot follows the pointer, a ring catches up with it; the ring grows over links
/// and shows the `data-cursor` label of what it hovers.
fn cursor() {
    let (Some(dot), Some(ring)) = (
        all(".cursor__dot").into_iter().next(),
        all(".cursor__ring").into_iter().next(),
    ) else {
        return;
    };
    let target = Rc::new(Cell::new((-100.0f64, -100.0f64)));
    {
        let (target, dot, ring) = (target.clone(), dot.clone(), ring.clone());
        on_passive(&window(), "pointermove", move |e: web_sys::PointerEvent| {
            let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
            target.set((x, y));
            set(&dot, "transform", &format!("translate3d({x}px,{y}px,0)"));
            let hovered = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|t| t.closest("a, button, input, [data-cursor]").ok().flatten());
            let label = hovered
                .as_ref()
                .and_then(|el| el.get_attribute("data-cursor"))
                .unwrap_or_default();
            let list = ring.class_list();
            let _ = list.toggle_with_force("cursor__ring--hover", hovered.is_some());
            let _ = list.toggle_with_force("cursor__ring--label", !label.is_empty());
            if let Some(span) = ring.query_selector("span").ok().flatten()
                && span.text_content().unwrap_or_default() != label
            {
                span.set_text_content(Some(&label));
            }
        });
    }
    for (event, down) in [("pointerdown", true), ("pointerup", false)] {
        let ring = ring.clone();
        on(&window(), event, move |_: web_sys::Event| {
            let _ = ring
                .class_list()
                .toggle_with_force("cursor__ring--down", down);
        });
    }
    let mut pos = (-100.0f64, -100.0f64);
    request_frame(move |_| {
        let (tx, ty) = target.get();
        if (tx - pos.0).abs() > 0.1 || (ty - pos.1).abs() > 0.1 {
            pos.0 += (tx - pos.0) * 0.17;
            pos.1 += (ty - pos.1) * 0.17;
            set(
                &ring,
                "transform",
                &format!("translate3d({:.1}px,{:.1}px,0)", pos.0, pos.1),
            );
        }
        true
    });
}

/// Buttons lean towards the pointer (`data-magnetic`).
fn magnetic() {
    for el in all("[data-magnetic]") {
        let target = el.clone();
        on(&el, "pointermove", move |e: web_sys::PointerEvent| {
            let r = target.get_bounding_client_rect();
            let x = f64::from(e.client_x()) - r.left() - r.width() / 2.0;
            let y = f64::from(e.client_y()) - r.top() - r.height() / 2.0;
            set(&target, "--mx", &format!("{:.1}px", x * 0.3));
            set(&target, "--my", &format!("{:.1}px", y * 0.3));
        });
        let target = el.clone();
        on(&el, "pointerleave", move |_: web_sys::Event| {
            set(&target, "--mx", "0px");
            set(&target, "--my", "0px");
        });
    }
}

/// Cards tilt in 3D towards the pointer, with a light that follows it (`data-tilt`).
fn tilt() {
    for el in all("[data-tilt]") {
        let max: f64 = el
            .get_attribute("data-tilt")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10.0);
        let target = el.clone();
        on(&el, "pointermove", move |e: web_sys::PointerEvent| {
            let r = target.get_bounding_client_rect();
            let x = (f64::from(e.client_x()) - r.left()) / r.width().max(1.0);
            let y = (f64::from(e.client_y()) - r.top()) / r.height().max(1.0);
            set(&target, "--px", &format!("{:.1}%", x * 100.0));
            set(&target, "--py", &format!("{:.1}%", y * 100.0));
            set(&target, "--ry", &format!("{:.2}deg", (x - 0.5) * max));
            set(&target, "--rx", &format!("{:.2}deg", (0.5 - y) * max));
        });
        let target = el.clone();
        on(&el, "pointerleave", move |_: web_sys::Event| {
            set(&target, "--rx", "0deg");
            set(&target, "--ry", "0deg");
        });
    }
}

/// The hero (its 3D scene and its blobs) follows the mouse, with some inertia.
fn hero_mouse() {
    let Some(hero) = all(".hero").into_iter().next() else {
        return;
    };
    let mouse = Rc::new(Cell::new((0.0f64, 0.0f64)));
    let running = Rc::new(Cell::new(false));
    let eased = Rc::new(Cell::new((0.0f64, 0.0f64)));
    on_passive(&window(), "pointermove", move |e: web_sys::PointerEvent| {
        let width = window()
            .inner_width()
            .ok()
            .and_then(|w| w.as_f64())
            .unwrap_or(1.0);
        mouse.set((
            f64::from(e.client_x()) / width.max(1.0) - 0.5,
            f64::from(e.client_y()) / viewport_height().max(1.0) - 0.5,
        ));
        if running.replace(true) {
            return;
        }
        let (hero, mouse, running, eased) =
            (hero.clone(), mouse.clone(), running.clone(), eased.clone());
        request_frame(move |_| {
            let (mx, my) = mouse.get();
            let (ex, ey) = eased.get();
            let next = (ex + (mx - ex) * 0.06, ey + (my - ey) * 0.06);
            eased.set(next);
            set(&hero, "--hx", &format!("{:.4}", next.0));
            set(&hero, "--hy", &format!("{:.4}", next.1));
            let moving = (mx - next.0).abs() > 0.001 || (my - next.1).abs() > 0.001;
            running.set(moving);
            moving
        });
    });
}
