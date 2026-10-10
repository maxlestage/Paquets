//! Installable apps (PWA): `app.pwa(Pwa::new("My app"))` serves the web app manifest
//! and an offline service worker (active's `pwa` module), and every [`Page`](crate::Page)
//! links them.

use std::path::Path;
use std::sync::OnceLock;

use active::pwa::{Manifest, ServiceWorker};
use vitesse::{Response, header};

/// The manifest given to [`App::pwa`](crate::App::pwa), once the app has started.
static MANIFEST: OnceLock<Manifest> = OnceLock::new();

/// Where the app serves them.
pub(crate) const MANIFEST_URL: &str = "/manifest.webmanifest";
pub(crate) const WORKER_URL: &str = "/sw.js";

/// Icons used when the manifest has none: the Paquets box.
const ICONS: [(&str, &str, &[u8]); 4] = [
    (
        "/paquets/icon-192.png",
        "192x192",
        include_bytes!("pwa/icon-192.png"),
    ),
    (
        "/paquets/icon-512.png",
        "512x512",
        include_bytes!("pwa/icon-512.png"),
    ),
    (
        "/paquets/icon-maskable-512.png",
        "512x512",
        include_bytes!("pwa/icon-maskable-512.png"),
    ),
    (
        "/paquets/apple-touch-icon.png",
        "180x180",
        include_bytes!("pwa/apple-touch-icon.png"),
    ),
];

/// The manifest of an installable app, for pages.
pub(crate) fn manifest() -> Option<&'static Manifest> {
    MANIFEST.get()
}

/// Registers the manifest, the worker (and the default icons) on the app.
pub(crate) fn install(
    app: &mut vitesse::App,
    manifest: Manifest,
    bundle: Option<&(String, String)>,
    network_only: &[String],
) {
    let manifest = if manifest.icons().is_empty() {
        for (url, _, bytes) in ICONS {
            app.get(url, move |_| async move {
                Response::new()
                    .header(header::CONTENT_TYPE, "image/png")
                    .header(header::CACHE_CONTROL, "public, max-age=604800")
                    .send(bytes)
            });
        }
        manifest
            .icon(ICONS[0].0, ICONS[0].1, "image/png")
            .icon(ICONS[1].0, ICONS[1].1, "image/png")
            .maskable_icon(ICONS[2].0, ICONS[2].1, "image/png")
    } else {
        manifest
    };

    let json = manifest.to_json();
    let worker = worker(&manifest, &json, bundle, network_only);
    app.get(MANIFEST_URL, move |_| {
        let json = json.clone();
        async move {
            Response::new()
                .header(header::CONTENT_TYPE, "application/manifest+json")
                .header(header::CACHE_CONTROL, "no-cache")
                .send(json)
        }
    });
    app.get(WORKER_URL, move |_| {
        let worker = worker.clone();
        async move {
            Response::new()
                .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
                .header(header::CACHE_CONTROL, "no-cache")
                .send(worker)
        }
    });
    let _ = MANIFEST.set(manifest);
}

/// The service worker: the start page, the islands bundle and the icons available
/// offline; the REST APIs always from the network. Its version changes with the
/// manifest and the bundle, so a deployment replaces the old caches.
fn worker(
    manifest: &Manifest,
    json: &str,
    bundle: Option<&(String, String)>,
    network_only: &[String],
) -> String {
    let start = start_page(manifest);
    let mut precache = vec![start.clone()];
    let mut fingerprint = Fnv::default();
    fingerprint.write(env!("CARGO_PKG_VERSION").as_bytes());
    fingerprint.write(json.as_bytes());
    if let Some((js, wasm)) = bundle {
        for url in [js, wasm] {
            precache.push(url.clone());
            if let Ok(bytes) = std::fs::read(Path::new(url.trim_start_matches('/'))) {
                fingerprint.write(&bytes);
            }
        }
    }
    for icon in manifest.icons() {
        if icon.src.starts_with('/') && !precache.contains(&icon.src) {
            precache.push(icon.src.clone());
        }
    }
    let mut worker = ServiceWorker::new(slug(manifest.name()), format!("{:x}", fingerprint.0))
        .precache(precache)
        .offline_page(start);
    for path in network_only {
        worker = worker.network_only(path.clone());
    }
    worker.to_js()
}

/// The start page as an absolute path (`./` or `.` is the root).
fn start_page(manifest: &Manifest) -> String {
    let json = manifest.to_json();
    let start = json
        .split("\"start_url\":\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or("/");
    match start {
        "" | "." | "./" => "/".to_owned(),
        s if s.starts_with('/') => s.to_owned(),
        s => format!("/{}", s.trim_start_matches("./")),
    }
}

/// `My App!` → `my-app`.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-');
    if out.is_empty() {
        "app".to_owned()
    } else {
        out.to_owned()
    }
}

/// FNV-1a: enough to tell two versions of the files apart.
struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_start_pages() {
        assert_eq!(slug("My App!"), "my-app");
        assert_eq!(slug("📦"), "app");
        assert_eq!(start_page(&Manifest::new("x")), "/");
        assert_eq!(start_page(&Manifest::new("x").start_url("/en/")), "/en/");
        assert_eq!(start_page(&Manifest::new("x").start_url("./app")), "/app");
    }

    #[test]
    fn the_worker_precaches_the_start_page_bundle_and_icons() {
        let manifest =
            Manifest::new("Todo")
                .start_url("/demo")
                .icon("/i.png", "192x192", "image/png");
        let bundle = ("/pkg/app.js".to_owned(), "/pkg/app_bg.wasm".to_owned());
        let js = worker(&manifest, "{}", Some(&bundle), &["/api/todos".to_owned()]);
        assert!(
            js.contains(r#"const PRECACHE = ["/demo","/pkg/app.js","/pkg/app_bg.wasm","/i.png"];"#)
        );
        assert!(js.contains(r#"const NETWORK_ONLY = ["/api/todos"];"#));
        assert!(js.contains(r#"const OFFLINE = "/demo";"#));
        assert!(js.contains(r#"const PREFIX = "todo-";"#));
    }
}
