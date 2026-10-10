//! Browser side of the terminal island: types the session when it scrolls into view,
//! and lights up the matching step of the timeline next to it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use paquets::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

use super::{all, reduced_motion, request_frame};
use crate::islands::{Line, step_at, total};

thread_local! {
    static LINES: RefCell<Option<Rc<Vec<Line>>>> = const { RefCell::new(None) };
    /// Each replay gets a number: an older animation stops when a newer one starts.
    static RUN: Cell<u32> = const { Cell::new(0) };
}

/// Milliseconds per tick.
const TICK: f64 = 34.0;

pub fn start(el: &web_sys::Element, tick: State<usize>, lines: Rc<Vec<Line>>) {
    let total = total(&lines);
    LINES.with(|l| *l.borrow_mut() = Some(lines));
    steps(total);
    if reduced_motion() {
        return;
    }
    let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, io: IntersectionObserver| {
            let visible = entries.iter().any(|e| {
                e.dyn_into::<IntersectionObserverEntry>()
                    .is_ok_and(|e| e.is_intersecting())
            });
            if visible {
                io.disconnect();
                replay(tick, total);
            }
        },
    );
    let options = IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(0.35));
    if let Ok(io) =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
    {
        io.observe(el);
    }
    callback.forget();
}

/// Types the whole session again.
pub fn replay(tick: State<usize>, total: usize) {
    let run = RUN.with(|r| {
        r.set(r.get() + 1);
        r.get()
    });
    tick.set(0);
    steps(0);
    let mut start = None;
    request_frame(move |now| {
        if RUN.with(Cell::get) != run {
            return false;
        }
        let t0 = *start.get_or_insert(now);
        let n = (((now - t0) / TICK) as usize).min(total);
        if n != tick.get() {
            tick.set(n);
            steps(n);
        }
        n < total
    });
}

/// Marks the steps of the timeline: done, active, or still to come.
fn steps(tick: usize) {
    let Some(step) = LINES.with(|l| l.borrow().as_ref().map(|lines| step_at(lines, tick))) else {
        return;
    };
    for (i, el) in all(".timeline__step").into_iter().enumerate() {
        let list = el.class_list();
        let _ = list.toggle_with_force("is-active", step == Some(i));
        let _ = list.toggle_with_force("is-done", step.is_some_and(|s| s > i));
    }
}
