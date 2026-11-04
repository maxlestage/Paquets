//! Ce que le serveur et le navigateur partagent : les îlots.
//!
//! Le serveur les rend en HTML (src/main.rs) ; compilé en WebAssembly, ce fichier les
//! rend interactifs dans le navigateur et lance les scènes 3D de `paquets::three`
//! (src/scenes.rs).

use paquets::prelude::*;

#[cfg(target_arch = "wasm32")]
mod scenes;

/// Les îlots, sous le nom donné à `island(...)`.
pub const ISLANDS: [(&str, paquets::active::IslandFn); 4] = [
    ("hero3d", hero3d),
    ("atelier", atelier),
    ("pluie", pluie),
    ("tarifs", tarifs),
];

/// Les formes que l'atelier sait montrer.
pub const FORMES: [&str; 4] = ["Paquet", "Tore", "Gemme", "Capsule"];

/// Les teintes de l'atelier : nom et couleur.
pub const TEINTES: [(&str, u32); 5] = [
    ("Violet", 0x7c5cff),
    ("Corail", 0xff6b6b),
    ("Menthe", 0x2ee6a6),
    ("Or", 0xffc857),
    ("Glacier", 0x5cc8ff),
];

/// Le fond du héros : une constellation 3D qui suit le pointeur et le défilement.
pub fn hero3d(_props: &str) -> Node {
    let fps = use_state(0u32);
    let r = Reglages::default();
    fragment([
        canvas()
            .class("hero-canvas")
            .attr("aria-hidden", "true")
            .use_action(move |canvas| lancer(canvas, Scene3d::Hero, fps, r)),
        p().class("hud")
            .attr("aria-hidden", "true")
            .child(span().class("hud-dot"))
            .text("paquets::three · WebGL 2 · ")
            .text_dyn(move || match fps.get() {
                0 => "—".into(),
                n => n.to_string(),
            })
            .text(" ips"),
    ])
}

/// L'atelier : un objet 3D à faire tourner du doigt, dont on choisit la forme, la teinte
/// et le rendu.
pub fn atelier(_props: &str) -> Node {
    let reglages = Reglages::default();
    let fps = use_state(0u32);
    let r = reglages;
    div()
        .class("atelier")
        .child(
            div()
                .class("atelier-stage")
                .child(
                    canvas()
                        .class("atelier-canvas")
                        .attr("role", "img")
                        .attr("aria-label", "Objet 3D interactif : faites-le tourner")
                        .use_action(move |canvas| lancer(canvas, Scene3d::Atelier, fps, r)),
                )
                .child(
                    p().class("stage-hint")
                        .text("↻ Glissez pour tourner · molette pour zoomer · ")
                        .text_dyn(move || match fps.get() {
                            0 => "—".into(),
                            n => format!("{n} ips"),
                        }),
                ),
        )
        .child(
            div()
                .class("atelier-panel")
                .child(h3().text("Configurez"))
                .child(p().class("muted").text("Forme"))
                .child(
                    div()
                        .class("chips")
                        .attr("role", "radiogroup")
                        .attr("aria-label", "Forme")
                        .children(FORMES.iter().enumerate().map(|(i, nom)| {
                            button()
                                .class("chip")
                                .attr("type", "button")
                                .attr("role", "radio")
                                .attr_dyn("aria-checked", move || (r.forme.get() == i).to_string())
                                .on_click(move |_| r.forme.set(i))
                                .text(*nom)
                        })),
                )
                .child(p().class("muted").text("Teinte"))
                .child(
                    div()
                        .class("swatches")
                        .attr("role", "radiogroup")
                        .attr("aria-label", "Teinte")
                        .children(TEINTES.iter().enumerate().map(|(i, (nom, hex))| {
                            button()
                                .class("swatch")
                                .attr("type", "button")
                                .attr("role", "radio")
                                .attr("title", *nom)
                                .attr("aria-label", *nom)
                                .attr("style", format!("--c:#{hex:06x}"))
                                .attr_dyn("aria-checked", move || (r.teinte.get() == i).to_string())
                                .on_click(move |_| r.teinte.set(i))
                        })),
                )
                .child(p().class("muted").text("Rendu"))
                .child(
                    div()
                        .class("toggles")
                        .child(toggle("Métal", r.metal))
                        .child(toggle("Filaire", r.filaire))
                        .child(toggle("Rotation auto", r.auto)),
                ),
        )
        .into()
}

fn toggle(label: &'static str, state: State<bool>) -> Element {
    button()
        .class("toggle")
        .attr("type", "button")
        .attr_dyn("aria-pressed", move || state.get().to_string())
        .on_click(move |_| state.update(|v| *v = !*v))
        .child(span().class("knob").attr("aria-hidden", "true"))
        .text(label)
}

/// La pluie de paquets : des boîtes qui tombent, rebondissent et s'empilent, avec la
/// physique de `paquets::three::Physics`.
pub fn pluie(_props: &str) -> Node {
    let reglages = Reglages::default();
    let compte = use_state(0u32);
    let r = reglages;
    div()
        .class("pluie")
        .child(
            canvas()
                .class("pluie-canvas")
                .attr("role", "img")
                .attr("aria-label", "Des paquets 3D tombent et s'empilent")
                .use_action(move |canvas| lancer(canvas, Scene3d::Pluie, compte, r)),
        )
        .child(
            div()
                .class("pluie-bar")
                .child(
                    button()
                        .class("btn btn-primary magnetic")
                        .attr("type", "button")
                        .on_click(move |_| r.lacher.update(|n| *n += 12))
                        .text("📦 Lâcher 12 paquets"),
                )
                .child(
                    button()
                        .class("btn btn-ghost")
                        .attr("type", "button")
                        .on_click(move |_| r.secousse.update(|n| *n += 1))
                        .text("💥 Secouer"),
                )
                .child(
                    span()
                        .class("pluie-count")
                        .text_dyn(move || match compte.get() {
                            0 => "Cliquez pour lâcher des paquets".into(),
                            1 => "1 paquet en jeu".into(),
                            n => format!("{n} paquets en jeu"),
                        }),
                ),
        )
        .into()
}

/// Les forfaits, en paiement mensuel ou annuel (deux mois offerts).
pub fn tarifs(_props: &str) -> Node {
    let annuel = use_state(false);
    let forfaits: [(&str, u32, &str, &[&str], bool); 3] = [
        (
            "Essentiel",
            490,
            "Un site vitrine rapide, beau et référencé.",
            &[
                "Jusqu'à 5 pages",
                "Animations au défilement",
                "PWA installable",
                "SEO technique complet",
                "Hébergement inclus",
            ],
            false,
        ),
        (
            "Studio",
            1290,
            "Le site qui marque : 3D temps réel et identité forte.",
            &[
                "Pages illimitées",
                "Scènes 3D WebGL sur mesure",
                "Micro-interactions avancées",
                "Back-office et base de données",
                "Analyse de performance mensuelle",
                "Support prioritaire",
            ],
            true,
        ),
        (
            "Orbite",
            2900,
            "Une équipe dédiée pour vos produits numériques.",
            &[
                "Équipe produit dédiée",
                "Applications web et API",
                "Expériences 3D et physique",
                "Audit sécurité trimestriel",
                "SLA 99,95 %",
            ],
            false,
        ),
    ];
    div()
        .class("tarifs")
        .child(
            div()
                .class("billing")
                .child(span().class_if("on", move || !annuel.get()).text("Mensuel"))
                .child(
                    button()
                        .class("switch")
                        .attr("type", "button")
                        .attr("role", "switch")
                        .attr("aria-label", "Paiement annuel")
                        .attr_dyn("aria-checked", move || annuel.get().to_string())
                        .on_click(move |_| annuel.update(|a| *a = !*a))
                        .child(span().class("switch-thumb")),
                )
                .child(
                    span()
                        .class_if("on", move || annuel.get())
                        .text("Annuel ")
                        .child(span().class("badge").text("−2 mois")),
                ),
        )
        .child(
            div()
                .class("plans")
                .children(forfaits.into_iter().enumerate().map(
                    |(i, (nom, prix, pitch, points, vedette))| {
                        div()
                            .class("plan tilt")
                            .class_if("featured", move || vedette)
                            .attr("data-reveal", "up")
                            .attr("style", format!("--d:{}ms", i * 120))
                            .child(if vedette {
                                span().class("ribbon").text("Le plus choisi")
                            } else {
                                span().class("ribbon-none")
                            })
                            .child(h3().text(nom))
                            .child(p().class("muted").text(pitch))
                            .child(
                                p().class("price")
                                    .child(strong().text_dyn(move || {
                                        if annuel.get() {
                                            format_prix(prix * 10)
                                        } else {
                                            format_prix(prix)
                                        }
                                    }))
                                    .child(span().text_dyn(move || {
                                        if annuel.get() {
                                            " € / an"
                                        } else {
                                            " € / mois"
                                        }
                                        .into()
                                    })),
                            )
                            .child(
                                ul().class("checks")
                                    .children(points.iter().map(|p| li().text(*p))),
                            )
                            .child(
                                a().class(if vedette {
                                    "btn btn-primary magnetic"
                                } else {
                                    "btn btn-ghost magnetic"
                                })
                                .href(format!("/contact?offre={nom}"))
                                .text("Démarrer"),
                            )
                    },
                )),
        )
        .into()
}

/// `12900` → `12 900`.
pub fn format_prix(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{202f}');
        }
        out.push(c);
    }
    out
}

/// Quelle scène lancer dans un canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene3d {
    /// Le fond du héros.
    Hero,
    /// L'atelier interactif.
    Atelier,
    /// La pluie de paquets.
    Pluie,
}

/// Ce que les boutons d'un îlot disent à sa scène.
#[derive(Clone, Copy)]
pub struct Reglages {
    /// Index dans [`FORMES`].
    pub forme: State<usize>,
    /// Index dans [`TEINTES`].
    pub teinte: State<usize>,
    /// Matériau métallique.
    pub metal: State<bool>,
    /// Rendu en fil de fer.
    pub filaire: State<bool>,
    /// Rotation automatique.
    pub auto: State<bool>,
    /// Paquets demandés depuis le début.
    pub lacher: State<u32>,
    /// Secousses demandées depuis le début.
    pub secousse: State<u32>,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            forme: use_state(0),
            teinte: use_state(0),
            metal: use_state(true),
            filaire: use_state(false),
            auto: use_state(true),
            lacher: use_state(0),
            secousse: use_state(0),
        }
    }
}

/// Lance la scène dans le navigateur et rend ce qui l'arrête.
#[cfg(target_arch = "wasm32")]
fn lancer(
    canvas: &paquets::active::Dom,
    scene: Scene3d,
    info: State<u32>,
    reglages: Reglages,
) -> impl FnOnce() + use<> {
    let running = scenes::start(canvas, scene, info, reglages);
    move || drop(running)
}

/// Rien à lancer sur le serveur.
#[cfg(not(target_arch = "wasm32"))]
fn lancer(
    _canvas: &paquets::active::Dom,
    _scene: Scene3d,
    _info: State<u32>,
    _reglages: Reglages,
) -> impl FnOnce() + use<> {
    || {}
}

/// Le point d'entrée du navigateur : rend chaque îlot de la page interactif.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&ISLANDS);
}
