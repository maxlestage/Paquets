//! An island with state, on a page with view transitions and prefetching: the first page
//! rendered with these options must not dispose the island's state before rendering it.
//! (One test in this file: the hashes of these options are found once per process.)

use paquets::prelude::*;

fn counter(_props: &str) -> Node {
    let count = use_state(7u32);
    p().text_dyn(move || count.get().to_string()).into()
}

#[test]
fn island_state_survives_the_first_page_with_options() {
    let html = Page::new("Home")
        .view_transitions()
        .prefetch(Prefetch::Hover)
        .prerender(Prefetch::Press)
        .body(island("counter", "", counter))
        .render();
    assert!(html.contains("data-island=\"counter\""));
    assert!(html.contains(">7<"));
}
