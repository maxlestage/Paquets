//! Chaque îlot se rend sur le serveur, seul ou à plusieurs dans une page.

use paquets::prelude::*;

#[test]
fn chaque_ile_se_rend() {
    for (nom, ile) in orbite::ISLANDS {
        let html = island(nom, "", ile).render();
        assert!(html.contains(&format!("data-island=\"{nom}\"")), "{nom}");
    }
}

#[test]
fn plusieurs_iles_dans_une_page() {
    let html = Node::from(
        div()
            .child(island("hero3d", "", orbite::hero3d))
            .child(island("atelier", "", orbite::atelier)),
    )
    .render();
    assert!(html.contains("hero-canvas") && html.contains("atelier-canvas"));
}

#[test]
fn prix() {
    assert_eq!(orbite::format_prix(490), "490");
    assert_eq!(orbite::format_prix(12900), "12\u{202f}900");
}
