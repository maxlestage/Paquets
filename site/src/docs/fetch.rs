//! Loads the Markdown pages (`docs/<lang>/<slug>.md`, copied next to the
//! site by Trunk), with a cache so that each page is downloaded only once.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

use crate::dom;
use crate::i18n::Lang;

thread_local! {
    static CACHE: RefCell<HashMap<(Lang, &'static str), Rc<str>>> = RefCell::new(HashMap::new());
}

/// The page, if it is already in the cache.
pub fn cached(lang: Lang, slug: &'static str) -> Option<Rc<str>> {
    CACHE.with(|c| c.borrow().get(&(lang, slug)).cloned())
}

/// Downloads the page (or takes it from the cache). `None` if it does not
/// exist in this language or if the network failed.
pub async fn load(lang: Lang, slug: &'static str) -> Option<Rc<str>> {
    if let Some(hit) = cached(lang, slug) {
        return Some(hit);
    }
    let text = fetch_text(&format!("docs/{}/{slug}.md", lang.code())).await?;
    CACHE.with(|c| c.borrow_mut().insert((lang, slug), text.clone()));
    Some(text)
}

async fn fetch_text(url: &str) -> Option<Rc<str>> {
    let response = JsFuture::from(dom::window().fetch_with_str(url))
        .await
        .ok()?
        .dyn_into::<Response>()
        .ok()?;
    if !response.ok() {
        return None;
    }
    let text = JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()?;
    Some(Rc::from(text))
}
