//! Small DOM helpers: animation loops, "visible on screen" observers, motion
//! preferences and smooth scrolling.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry,
    IntersectionObserverInit, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition,
    ScrollToOptions, Window,
};

pub fn window() -> Window {
    web_sys::window().expect("no window")
}

pub fn document() -> web_sys::Document {
    window().document().expect("no document")
}

/// The visitor asked the system to reduce motion.
pub fn reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

/// The device has a real pointer (a mouse), not only a touch screen.
pub fn fine_pointer() -> bool {
    window()
        .match_media("(hover: hover) and (pointer: fine)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

fn behavior() -> ScrollBehavior {
    if reduced_motion() {
        ScrollBehavior::Instant
    } else {
        ScrollBehavior::Smooth
    }
}

/// Scrolls the page back to the top.
pub fn scroll_top(smooth: bool) {
    let options = ScrollToOptions::new();
    options.set_top(0.0);
    options.set_behavior(if smooth {
        behavior()
    } else {
        ScrollBehavior::Instant
    });
    window().scroll_to_with_scroll_to_options(&options);
}

/// Scrolls until `element` is at the top of the screen (under the fixed
/// navigation, thanks to `scroll-margin-top` in the CSS).
pub fn scroll_to(element: &Element, smooth: bool) {
    let options = ScrollIntoViewOptions::new();
    options.set_block(ScrollLogicalPosition::Start);
    options.set_behavior(if smooth {
        behavior()
    } else {
        ScrollBehavior::Instant
    });
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

/// A `requestAnimationFrame` loop; it stops when dropped.
pub struct RafLoop {
    running: Rc<Cell<bool>>,
}

impl Drop for RafLoop {
    fn drop(&mut self) {
        self.running.set(false);
    }
}

/// Calls `frame(time_ms)` on every frame; `frame` returns `false` to stop.
pub fn raf_loop(mut frame: impl FnMut(f64) -> bool + 'static) -> RafLoop {
    let running = Rc::new(Cell::new(true));
    type Slot = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
    let slot: Slot = Rc::new(RefCell::new(None));
    let next = slot.clone();
    let alive = running.clone();
    *slot.borrow_mut() = Some(Closure::new(move |t: f64| {
        if !alive.get() || !frame(t) {
            alive.set(false);
            return;
        }
        if let Some(cb) = next.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    }
    RafLoop { running }
}

/// Calls `on_visible` once, when `element` enters the screen. Returns the
/// observer (keep it, or disconnect it).
pub fn on_first_visible(
    element: &Element,
    threshold: f64,
    on_visible: impl FnOnce() + 'static,
) -> IntersectionObserver {
    let on_visible = RefCell::new(Some(on_visible));
    let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, observer: IntersectionObserver| {
            let visible = entries.iter().any(|e| {
                e.dyn_into::<IntersectionObserverEntry>()
                    .is_ok_and(|e| e.is_intersecting())
            });
            if visible {
                observer.disconnect();
                if let Some(f) = on_visible.borrow_mut().take() {
                    f();
                }
            }
        },
    );
    let options = IntersectionObserverInit::new();
    options.set_threshold(&threshold.into());
    let observer =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
            .expect("IntersectionObserver");
    callback.forget();
    observer.observe(element);
    observer
}

/// Reveals the `[data-reveal]` elements while it is kept alive.
pub struct Reveal {
    observer: IntersectionObserver,
    _callback: Closure<dyn FnMut(js_sys::Array, IntersectionObserver)>,
}

impl Drop for Reveal {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

/// Adds the `in` class to every `[data-reveal]` element that enters the
/// screen (the animations themselves are in CSS).
pub fn reveal_on_scroll() -> Option<Reveal> {
    let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
        |entries: js_sys::Array, observer: IntersectionObserver| {
            for entry in entries.iter() {
                let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() else {
                    continue;
                };
                if entry.is_intersecting() {
                    let target = entry.target();
                    let _ = target.class_list().add_1("in");
                    observer.unobserve(&target);
                }
            }
        },
    );
    let options = IntersectionObserverInit::new();
    options.set_threshold(&0.0.into());
    options.set_root_margin("0px 0px -8% 0px");
    let observer =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options).ok()?;
    if let Ok(nodes) = document().query_selector_all("[data-reveal]:not(.in)") {
        for i in 0..nodes.length() {
            if let Some(node) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                observer.observe(&node);
            }
        }
    }
    Some(Reveal {
        observer,
        _callback: callback,
    })
}

/// Sets a CSS property (or variable) on an element.
pub fn set_var(element: &HtmlElement, name: &str, value: &str) {
    let _ = element.style().set_property(name, value);
}

/// The size of the viewport, in CSS pixels.
pub fn viewport() -> (f64, f64) {
    let win = window();
    (
        win.inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        win.inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    )
}

/// The element with this id, as an `HtmlElement`.
pub fn by_id(id: &str) -> Option<HtmlElement> {
    document().get_element_by_id(id).and_then(|e| e.dyn_into().ok())
}

/// A random number in `[0, 1[`.
pub fn random() -> f64 {
    js_sys::Math::random()
}
