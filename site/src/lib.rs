//! The Paquets website, written with Paquets: the texts, routes and islands shared by
//! the server and the browser, the server (native) and the browser bundle (wasm32).

pub mod catalog;
pub mod data;
pub mod highlight;
pub mod i18n;
pub mod icons;
pub mod islands;
pub mod routes;
pub mod search;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
mod client;
