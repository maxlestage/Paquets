//! Scroll and pointer effects: progress bar, navigation that hides when
//! scrolling down, parallax (`data-speed`), the progress of the "from zero to
//! production" timeline (`--p`), and the hero that follows the mouse.

use std::cell::Cell;
use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::dom::{self, RafLoop};

/// The effects live as long as this value is kept.
pub struct Effects {
    _anim: RafLoop,
    _listeners: Vec<EventListener>,
}

pub fn start() -> Effects {
    let progress = dom::by_id("progress");
    let nav = dom::by_id("nav");
    let journey = dom::by_id("journey");
    let hero = dom::by_id("top");
    let mut parallax: Vec<(HtmlElement, f64)> = Vec::new();
    if let Ok(nodes) = dom::document().query_selector_all("[data-speed]") {
        for i in 0..nodes.length() {
            let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else {
                continue;
            };
            let speed = el
                .get_attribute("data-speed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            parallax.push((el, speed));
        }
    }
    let still = dom::reduced_motion();
    let mut listeners = Vec::new();

    // The mouse position, from -0.5 to 0.5 on each axis.
    let mouse = Rc::new(Cell::new((0.0f64, 0.0f64)));
    if dom::fine_pointer() && !still {
        let mouse = mouse.clone();
        listeners.push(EventListener::new(&dom::window(), "mousemove", move |e| {
            let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() else {
                return;
            };
            let (w, h) = dom::viewport();
            mouse.set((
                f64::from(e.client_x()) / w.max(1.0) - 0.5,
                f64::from(e.client_y()) / h.max(1.0) - 0.5,
            ));
        }));
    }

    let mut last_y = -1.0f64;
    let mut eased = (0.0f64, 0.0f64);
    let anim = dom::raf_loop(move |_| {
        let y = dom::window().scroll_y().unwrap_or(0.0);
        let (_, h) = dom::viewport();
        if (y - last_y).abs() > 0.5 {
            if let Some(progress) = &progress {
                let max = dom::document()
                    .document_element()
                    .map_or(1.0, |e| f64::from(e.scroll_height()) - h)
                    .max(1.0);
                dom::set_var(
                    progress,
                    "transform",
                    &format!("scaleX({:.4})", (y / max).clamp(0.0, 1.0)),
                );
            }
            if let Some(nav) = &nav {
                let list = nav.class_list();
                let _ = list.toggle_with_force("nav--scrolled", y > 30.0);
                if y > last_y + 2.0 && y > 500.0 {
                    let _ = list.add_1("nav--hidden");
                } else if y < last_y - 2.0 {
                    let _ = list.remove_1("nav--hidden");
                }
            }
            if !still {
                for (el, speed) in &parallax {
                    dom::set_var(
                        el,
                        "transform",
                        &format!("translate3d(0,{:.1}px,0)", y * speed),
                    );
                }
            }
            if let Some(journey) = &journey {
                // 0 when the timeline enters the middle of the screen, 1 when
                // its end reaches it.
                let rect = journey.get_bounding_client_rect();
                let p = ((h * 0.6 - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
                dom::set_var(journey, "--p", &format!("{p:.4}"));
            }
            if let Some(hero) = &hero {
                let p = (y / h.max(1.0)).clamp(0.0, 1.0);
                dom::set_var(hero, "--scroll", &format!("{p:.4}"));
            }
            last_y = y;
        }
        if let Some(hero) = &hero {
            let (mx, my) = mouse.get();
            let next = (
                eased.0 + (mx - eased.0) * 0.06,
                eased.1 + (my - eased.1) * 0.06,
            );
            if (next.0 - eased.0).abs() > 0.0005 || (next.1 - eased.1).abs() > 0.0005 {
                eased = next;
                dom::set_var(hero, "--mx", &format!("{:.4}", eased.0));
                dom::set_var(hero, "--my", &format!("{:.4}", eased.1));
            }
        }
        true
    });
    Effects {
        _anim: anim,
        _listeners: listeners,
    }
}
