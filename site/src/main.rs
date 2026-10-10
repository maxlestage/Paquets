//! The website of Paquets, in Rust with Yew (WebAssembly), in English,
//! French and Spanish, with its documentation hub.

mod components;
mod data;
mod docs;
mod dom;
mod highlight;
mod i18n;
mod route;

use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use yew::prelude::*;

use components::layout::{Cursor, Footer, Loader, Nav};
use components::sections::{
    Cta, Deploy, DocsPreview, Features, Hero, Journey, Marquee, Numbers, Packages, Showcase,
};
use docs::{DocsHome, DocsPage};
use i18n::{Lang, LangCtx};
use route::Route;

#[component]
fn Home() -> Html {
    let t = i18n::use_lang().lang.texts();
    use_effect_with(t.meta_title, |title| dom::document().set_title(title));
    html! {
        <>
            <Hero />
            <Marquee />
            <Packages />
            <Showcase />
            <Features />
            <Numbers />
            <Journey />
            <Deploy />
            <DocsPreview />
            <Cta />
        </>
    }
}

/// Scrolls to the section of the hash (`#packages`…), once the home page is
/// shown.
fn scroll_to_section(smooth: bool) {
    let hash = dom::window().location().hash().unwrap_or_default();
    let id = hash.trim_start_matches('#');
    if id.is_empty() || id.starts_with('/') {
        return;
    }
    if let Some(el) = dom::document().get_element_by_id(id) {
        if id == "top" {
            dom::scroll_top(smooth);
        } else {
            dom::scroll_to(&el, smooth);
        }
    }
}

#[component]
fn App() -> Html {
    let lang = use_state(Lang::detect);
    let route = use_state(route::current);
    let loaded = use_state(|| false);

    {
        let route = route.clone();
        use_effect_with((), move |_| {
            let listener = EventListener::new(&dom::window(), "hashchange", move |_| {
                let next = route::current();
                // A section of the home page, from the home page: the browser
                // already scrolls (smoothly, thanks to the CSS).
                if *route != next {
                    route.set(next);
                } else if next == Route::Home {
                    scroll_to_section(true);
                }
            });
            move || drop(listener)
        });
    }
    {
        let loaded = loaded.clone();
        let docs = route.is_docs();
        use_effect_with((), move |_| {
            let delay = match (dom::reduced_motion(), docs) {
                (true, _) => 0,
                (false, true) => 650,
                (false, false) => 1900,
            };
            let timeout = Timeout::new(delay, move || loaded.set(true));
            move || drop(timeout)
        });
    }
    use_effect_with(*loaded, |loaded| {
        if *loaded && let Some(body) = dom::document().body() {
            let _ = body.class_list().add_1("is-loaded");
        }
    });
    use_effect_with(*lang, |lang| {
        if let Some(root) = dom::document().document_element() {
            let _ = root.set_attribute("lang", lang.code());
        }
    });
    // Scroll effects and reveals start again on every screen change (home,
    // docs hub, a docs page).
    let screen = match &*route {
        Route::Home => "home".to_owned(),
        Route::DocsHome => "docs".to_owned(),
        Route::Doc { slug, .. } => (*slug).to_owned(),
    };
    use_effect_with((screen, *lang), |_| {
        let effects = components::effects::start();
        let reveal = dom::reveal_on_scroll();
        move || {
            drop(effects);
            drop(reveal);
        }
    });
    use_effect_with((*route).clone(), |route| {
        if *route == Route::Home {
            scroll_to_section(false);
        }
    });

    let context = LangCtx {
        lang: *lang,
        set: {
            let lang = lang.clone();
            Callback::from(move |l: Lang| {
                l.save();
                lang.set(l);
            })
        },
    };
    html! {
        <ContextProvider<LangCtx> {context}>
            <Loader done={*loaded} />
            <Cursor />
            <Nav docs={route.is_docs()} />
            <main class={classes!(route.is_docs().then_some("main--docs"))}>
                { match &*route {
                    Route::Home => html! { <Home /> },
                    Route::DocsHome => html! { <DocsHome /> },
                    Route::Doc { slug, anchor } => html! { <DocsPage slug={*slug} anchor={anchor.clone()} /> },
                } }
            </main>
            <Footer />
            <div class="noise" aria-hidden="true"></div>
        </ContextProvider<LangCtx>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
