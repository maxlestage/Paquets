//! Orbite — le serveur : les pages (rendues sur le serveur, avec leurs balises SEO), les
//! formulaires de contact et de newsletter enregistrés en base, la PWA, le plan du site.
//!
//!     cargo run        # http://localhost:3000 (PORT sur Heroku)
//!
//! Les scènes 3D et les éléments interactifs sont des îlots (src/lib.rs), compilés en
//! WebAssembly dans `pkg/` : voir le README.

use paquets::prelude::*;

mod contenu;
mod icones;

use contenu::*;
use icones::icone;

// ————————————————————————————————————————————————————————————————————————————
// Données
// ————————————————————————————————————————————————————————————————————————————

/// Un message envoyé depuis la page de contact.
#[model]
struct Message {
    id: i64,
    nom: String,
    email: String,
    entreprise: String,
    offre: String,
    budget: String,
    contenu: String,
    /// Secondes depuis le 1er janvier 1970.
    recu_le: i64,
}

/// Une adresse inscrite à la newsletter.
#[model]
struct Abonne {
    id: i64,
    email: String,
    inscrit_le: i64,
}

/// Les champs du formulaire de contact.
#[data]
struct FormulaireContact {
    nom: String,
    email: String,
    #[serde(default)]
    entreprise: String,
    #[serde(default)]
    offre: String,
    #[serde(default)]
    budget: String,
    contenu: String,
    /// Un champ caché que seuls les robots remplissent.
    #[serde(default)]
    site_web: String,
}

/// Le champ de la newsletter.
#[data]
struct FormulaireNewsletter {
    email: String,
    #[serde(default)]
    site_web: String,
}

fn maintenant() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn email_valide(email: &str) -> bool {
    let email = email.trim();
    let Some((avant, apres)) = email.split_once('@') else {
        return false;
    };
    !avant.is_empty()
        && apres.contains('.')
        && !apres.starts_with('.')
        && !apres.ends_with('.')
        && email.len() <= 254
        && !email.contains(char::is_whitespace)
}

/// Coupe un texte trop long.
fn borne(texte: &str, max: usize) -> String {
    texte.trim().chars().take(max).collect()
}

// ————————————————————————————————————————————————————————————————————————————
// La mise en page commune : en-tête, contenu, pied de page
// ————————————————————————————————————————————————————————————————————————————

/// L'adresse publique du site, pour les URL canoniques et Open Graph.
fn site_url() -> String {
    std::env::var("SITE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".into())
        .trim_end_matches('/')
        .to_owned()
}

/// Une page du site : `chemin` marque le lien actif du menu.
fn page(chemin: &str, titre: &str, description: &str, contenu: Vec<Node>) -> Page {
    let base = site_url();
    let json_ld = format!(
        r#"{{"@context":"https://schema.org","@type":"Organization","name":"{STUDIO}","url":"{base}","email":"{EMAIL}","telephone":"{TELEPHONE}","address":{{"@type":"PostalAddress","streetAddress":"{RUE}","postalCode":"{CODE_POSTAL}","addressLocality":"{VILLE}","addressCountry":"FR"}}}}"#
    );
    let mut page = Page::new(titre)
        .lang("fr")
        .description(description)
        .seo(
            Seo::new()
                .site_name(STUDIO)
                .title_template(format!("%s · {STUDIO}"))
                .base_url(base)
                .canonical(chemin)
                .image("/og.png")
                .locale("fr_FR")
                .json_ld(json_ld),
        )
        .plain()
        .stylesheet(
            "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=Inter:wght@400;500;600&display=swap",
        )
        .stylesheet("/style.css")
        .head(concat!(
            r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml"/>"#,
            r#"<link rel="apple-touch-icon" href="/icons/apple-touch-icon.png"/>"#,
            r#"<meta name="color-scheme" content="dark"/>"#,
        ))
        .view_transitions()
        .prefetch(Prefetch::Hover);
    page = page.body(Node::from(curseur()));
    page = page.body(island("mouvement", "", orbite::mouvement));
    page = page.body(Node::from(entete(chemin)));
    page = page.body(Node::from(
        html::main().id("contenu").class("page").children(contenu),
    ));
    page.body(Node::from(pied_de_page()))
}

fn curseur() -> Element {
    div()
        .class("cursor")
        .attr("aria-hidden", "true")
        .child(div().class("cursor-dot"))
        .child(div().class("cursor-ring"))
}

fn logo() -> Element {
    a().class("logo")
        .href("/")
        .attr("aria-label", format!("{STUDIO}, accueil"))
        .child(span().class("logo-mark").html(icone("logo")))
        .child(
            span()
                .class("logo-text")
                .text("Orbite")
                .child(span().text(".studio")),
        )
}

fn entete(chemin: &str) -> Element {
    let liens = MENU.iter().map(|(href, label)| {
        let actif = *href == chemin || (*href != "/" && chemin.starts_with(href));
        a().href(*href)
            .class("nav-link")
            .class_if("active", move || actif)
            .attr("data-text", *label)
            .child(span().text(*label))
    });
    header()
        .class("site-header")
        .child(a().class("skip").href("#contenu").text("Aller au contenu"))
        .child(div().class("progress").attr("aria-hidden", "true"))
        .child(
            div()
                .class("nav-shell")
                .child(logo())
                .child(
                    nav()
                        .class("nav")
                        .id("menu")
                        .attr("aria-label", "Navigation principale")
                        .children(liens)
                        .child(
                            a().class("btn btn-primary btn-sm magnetic nav-cta")
                                .href("/contact")
                                .text("Démarrer un projet")
                                .child(span().class("arrow").html(icone("fleche"))),
                        ),
                )
                .child(
                    button()
                        .class("burger")
                        .attr("type", "button")
                        .attr("aria-label", "Ouvrir le menu")
                        .attr("aria-expanded", "false")
                        .attr("aria-controls", "menu")
                        .child(span())
                        .child(span())
                        .child(span()),
                ),
        )
}

fn pied_de_page() -> Element {
    let colonne = |titre: &'static str, liens: &'static [(&'static str, &'static str)]| {
        div().class("footer-col").child(h4().text(titre)).child(
            ul().children(
                liens
                    .iter()
                    .map(|(href, label)| li().child(a().href(*href).class("u-link").text(*label))),
            ),
        )
    };
    footer()
        .class("site-footer")
        .child(div().class("footer-glow").attr("aria-hidden", "true"))
        // Le bandeau : newsletter et appel à l'action.
        .child(
            div()
                .class("container footer-top")
                .attr("data-reveal", "up")
                .child(
                    div()
                        .class("newsletter")
                        .id("newsletter")
                        .child(span().class("eyebrow").text("Lettre d'orbite"))
                        .child(h3().text("Une idée par mois, zéro spam."))
                        .child(p().class("muted").text(
                            "Coulisses de nos projets 3D, veille WebGL et conseils de performance. Désinscription en un clic.",
                        ))
                        .child(
                            form()
                                .class("newsletter-form")
                                .attr("method", "post")
                                .attr("action", "/newsletter")
                                .child(
                                    label()
                                        .class("sr-only")
                                        .attr("for", "nl-email")
                                        .text("Adresse e-mail"),
                                )
                                .child(
                                    input()
                                        .id("nl-email")
                                        .attr("type", "email")
                                        .attr("name", "email")
                                        .attr("autocomplete", "email")
                                        .attr("placeholder", "vous@entreprise.fr")
                                        .attr("required", ""),
                                )
                                .child(
                                    input()
                                        .class("hp")
                                        .attr("type", "text")
                                        .attr("name", "site_web")
                                        .attr("tabindex", "-1")
                                        .attr("autocomplete", "off")
                                        .attr("aria-hidden", "true"),
                                )
                                .child(
                                    button()
                                        .class("btn btn-primary magnetic")
                                        .attr("type", "submit")
                                        .text("S'inscrire")
                                        .child(span().class("arrow").html(icone("fleche"))),
                                ),
                        )
                        .child(p().class("fine").text(
                            "En vous inscrivant, vous acceptez notre politique de confidentialité.",
                        )),
                )
                .child(
                    div()
                        .class("footer-cta")
                        .child(p().class("big").text("Un projet en tête ?"))
                        .child(
                            a().class("cta-orb magnetic")
                                .href("/contact")
                                .attr("aria-label", "Parlons-en")
                                .child(span().text("Parlons-en"))
                                .child(span().class("orb-ring").attr("aria-hidden", "true")),
                        ),
                ),
        )
        // Les colonnes.
        .child(
            div()
                .class("container footer-grid")
                .child(
                    div()
                        .class("footer-brand")
                        .child(logo())
                        .child(p().class("muted").text(
                            "Studio créatif indépendant. Nous concevons des sites et des expériences 3D temps réel, rapides, accessibles et sécurisés.",
                        ))
                        .child(
                            div()
                                .class("socials")
                                .children(RESEAUX.iter().map(|(nom, href, ico)| {
                                    a().class("social magnetic")
                                        .href(*href)
                                        .attr("aria-label", *nom)
                                        .attr("rel", "noopener noreferrer")
                                        .attr("target", "_blank")
                                        .html(icone(ico))
                                })),
                        )
                        .child(
                            div()
                                .class("status")
                                .child(span().class("pulse").attr("aria-hidden", "true"))
                                .text("Disponibles pour de nouveaux projets — T1 2027"),
                        ),
                )
                .child(colonne("Studio", PIED_STUDIO))
                .child(colonne("Expertises", PIED_SERVICES))
                .child(colonne("Ressources", PIED_RESSOURCES))
                .child(
                    div()
                        .class("footer-col footer-contact")
                        .child(h4().text("Contact"))
                        .child(
                            address()
                                .child(
                                    p().child(span().class("ico").html(icone("lieu")))
                                        .child(span().text(format!("{RUE}, {CODE_POSTAL} {VILLE}"))),
                                )
                                .child(
                                    p().child(span().class("ico").html(icone("mail")))
                                        .child(a().class("u-link").href(format!("mailto:{EMAIL}")).text(EMAIL)),
                                )
                                .child(
                                    p().child(span().class("ico").html(icone("tel")))
                                        .child(
                                            a().class("u-link")
                                                .href(format!("tel:{}", TELEPHONE.replace(' ', "")))
                                                .text(TELEPHONE),
                                        ),
                                )
                                .child(
                                    p().child(span().class("ico").html(icone("horloge")))
                                        .child(span().text("Lun – ven · 9 h – 18 h")),
                                ),
                        )
                        .child(
                            button()
                                .class("btn btn-ghost btn-sm install-app")
                                .attr("type", "button")
                                .attr("hidden", "")
                                .child(span().class("ico").html(icone("telecharger")))
                                .text("Installer l'application"),
                        ),
                ),
        )
        // Le grand nom.
        .child(
            div()
                .class("footer-word")
                .attr("aria-hidden", "true")
                .child(span().text("ORBITE")),
        )
        // La ligne légale.
        .child(
            div()
                .class("container footer-bottom")
                .child(p().text(format!("© {ANNEE} {STUDIO} SAS · SIRET {SIRET} · Tous droits réservés.")))
                .child(
                    nav()
                        .class("legal")
                        .attr("aria-label", "Informations légales")
                        .child(a().class("u-link").href("/mentions-legales").text("Mentions légales"))
                        .child(a().class("u-link").href("/confidentialite").text("Confidentialité"))
                        .child(a().class("u-link").href("/plan-du-site").text("Plan du site"))
                        .child(a().class("u-link").href("/sitemap.xml").text("sitemap.xml")),
                )
                .child(
                    p().class("made")
                        .text("Fait avec ")
                        .child(a().class("u-link").href("https://github.com/maxlestage/Paquets").text("Paquets 📦"))
                        .text(" · Rust · WebAssembly"),
                )
                .child(
                    button()
                        .class("to-top magnetic")
                        .attr("type", "button")
                        .attr("aria-label", "Revenir en haut")
                        .html(icone("haut")),
                ),
        )
}

// ————————————————————————————————————————————————————————————————————————————
// Briques des pages
// ————————————————————————————————————————————————————————————————————————————

/// Un titre de section : sur-titre, titre (mots révélés un par un) et chapeau.
fn titre_section(eyebrow: &str, titre: &str, chapeau: &str) -> Element {
    div()
        .class("section-head")
        .child(
            span()
                .class("eyebrow")
                .attr("data-reveal", "up")
                .text(eyebrow.to_owned()),
        )
        .child(mots(h2(), titre))
        .child(
            p().class("lead")
                .attr("data-reveal", "up")
                .attr("style", "--d:200ms")
                .text(chapeau.to_owned()),
        )
}

/// Découpe un titre en mots, pour qu'ils apparaissent l'un après l'autre. Les mots entre
/// `*` prennent le dégradé.
fn mots(el: Element, texte: &str) -> Element {
    let mut el = el.class("split").attr("aria-label", texte.replace('*', ""));
    for (i, mot) in texte.split(' ').enumerate() {
        // `*mot*`, éventuellement suivi d'une ponctuation : le mot prend le dégradé.
        let (grad, reste) = match mot.strip_prefix('*').and_then(|m| m.split_once('*')) {
            Some((g, r)) => (g.to_owned(), r.to_owned()),
            None => (String::new(), mot.to_owned()),
        };
        let mut wi = span().class("wi").attr("style", format!("--i:{i}"));
        if !grad.is_empty() {
            wi = wi.child(span().class("grad").text(grad));
        }
        if !reste.is_empty() {
            wi = wi.text(reste);
        }
        el = el
            .child(span().class("w").attr("aria-hidden", "true").child(wi))
            .text(" ");
    }
    el
}

/// L'en-tête des pages intérieures.
fn bandeau(eyebrow: &str, titre: &str, chapeau: &str) -> Node {
    section()
        .class("page-hero")
        .child(div().class("aurora").attr("aria-hidden", "true"))
        .child(div().class("grid-floor").attr("aria-hidden", "true"))
        .child(
            div()
                .class("container")
                .child(
                    span()
                        .class("eyebrow")
                        .attr("data-reveal", "up")
                        .text(eyebrow.to_owned()),
                )
                .child(mots(h1(), titre))
                .child(
                    p().class("lead")
                        .attr("data-reveal", "up")
                        .attr("style", "--d:250ms")
                        .text(chapeau.to_owned()),
                ),
        )
        .into()
}

/// Un cube en CSS 3D, qui tourne.
fn cube_css(classe: &str) -> Element {
    div()
        .class(format!("css-cube {classe}"))
        .attr("aria-hidden", "true")
        .children(["f1", "f2", "f3", "f4", "f5", "f6"].map(|f| div().class(format!("face {f}"))))
}

fn carte_service(i: usize, s: &Service) -> Element {
    article()
        .class("card service tilt glow")
        .attr("data-reveal", "up")
        .attr("style", format!("--d:{}ms", i * 90))
        .child(div().class("card-ico").html(icone(s.icone)))
        .child(h3().text(s.titre))
        .child(p().class("muted").text(s.texte))
        .child(
            ul().class("tags")
                .children(s.tags.iter().map(|t| li().text(*t))),
        )
}

fn carte_projet(i: usize, pr: &Projet) -> Element {
    article()
        .class("project tilt")
        .attr("data-reveal", "up")
        .attr("data-cat", pr.categorie)
        .attr(
            "style",
            format!(
                "--d:{}ms;--a:{};--b:{}",
                (i % 3) * 110,
                pr.couleurs.0,
                pr.couleurs.1
            ),
        )
        .child(
            div()
                .class("project-visual")
                .attr("aria-hidden", "true")
                .child(div().class("pv-orb"))
                .child(div().class("pv-ring"))
                .child(cube_css("pv-cube"))
                .child(span().class("pv-index").text(format!("{:02}", i + 1))),
        )
        .child(
            div()
                .class("project-body")
                .child(
                    p().class("project-meta")
                        .child(span().text(pr.client))
                        .child(span().text(pr.annee)),
                )
                .child(h3().text(pr.titre))
                .child(p().class("muted").text(pr.texte))
                .child(
                    ul().class("tags")
                        .children(pr.tags.iter().map(|t| li().text(*t))),
                )
                .child(
                    p().class("project-result")
                        .child(strong().text(pr.resultat.0))
                        .text(format!(" {}", pr.resultat.1)),
                ),
        )
}

fn chiffres() -> Element {
    div()
        .class("stats")
        .children(CHIFFRES.iter().enumerate().map(|(i, (n, suffixe, label))| {
            div()
                .class("stat")
                .attr("data-reveal", "up")
                .attr("style", format!("--d:{}ms", i * 100))
                .child(
                    p().class("stat-num")
                        .child(span().attr("data-count", n.to_string()).text("0"))
                        .text(*suffixe),
                )
                .child(p().class("muted").text(*label))
        }))
}

fn defilant(rangee: &[&'static str], inverse: bool) -> Element {
    let items = || {
        rangee.iter().map(|nom| {
            span()
                .class("marquee-item")
                .child(span().class("star").text("✦"))
                .text(*nom)
        })
    };
    div()
        .class(if inverse {
            "marquee reverse"
        } else {
            "marquee"
        })
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("marquee-track")
                .children(items())
                .children(items()),
        )
}

fn appel_final() -> Node {
    section()
        .class("section")
        .child(
            div()
                .class("container")
                .child(
                    div()
                        .class("cta-band")
                        .attr("data-reveal", "zoom")
                        .child(div().class("cta-orbits").attr("aria-hidden", "true").children(
                            (0..3).map(|i| div().class(format!("orbit o{i}")).child(span())),
                        ))
                        .child(span().class("eyebrow").text("Prochaine étape"))
                        .child(mots(h2(), "Faisons décoller *votre* projet"))
                        .child(p().class("lead").text(
                            "Un appel de 30 minutes, gratuit, pour cadrer vos objectifs et repartir avec une estimation claire.",
                        ))
                        .child(
                            div()
                                .class("actions")
                                .child(
                                    a().class("btn btn-primary btn-lg magnetic")
                                        .href("/contact")
                                        .text("Réserver un appel")
                                        .child(span().class("arrow").html(icone("fleche"))),
                                )
                                .child(
                                    a().class("btn btn-ghost btn-lg magnetic")
                                        .href("/realisations")
                                        .text("Voir nos réalisations"),
                                ),
                        ),
                ),
        )
        .into()
}

// ————————————————————————————————————————————————————————————————————————————
// Les pages
// ————————————————————————————————————————————————————————————————————————————

async fn accueil(_req: Request) -> Page {
    let hero = section()
        .class("hero")
        .child(div().class("aurora").attr("aria-hidden", "true"))
        .child(island("hero3d", "", orbite::hero3d))
        .child(
            div()
                .class("container hero-inner")
                .child(
                    a().class("pill")
                        .href("/realisations")
                        .attr("data-reveal", "down")
                        .child(span().class("pill-tag").text("Nouveau"))
                        .text("Nos scènes 3D tournent en Rust + WebAssembly")
                        .child(span().class("arrow").html(icone("fleche"))),
                )
                .child(mots(h1().class("hero-title"), "Des sites qui *prennent* *vie* en 3D."))
                .child(
                    p().class("lead hero-lead")
                        .attr("data-reveal", "up")
                        .attr("style", "--d:500ms")
                        .text(
                            "Orbite conçoit des expériences web immersives : 3D temps réel, animations fluides et performances irréprochables. Installables, rapides, et pensées pour convertir.",
                        ),
                )
                .child(
                    div()
                        .class("actions")
                        .attr("data-reveal", "up")
                        .attr("style", "--d:650ms")
                        .child(
                            a().class("btn btn-primary btn-lg magnetic")
                                .href("/contact")
                                .text("Démarrer un projet")
                                .child(span().class("arrow").html(icone("fleche"))),
                        )
                        .child(
                            a().class("btn btn-ghost btn-lg magnetic")
                                .href("#atelier")
                                .child(span().class("play").html(icone("play")))
                                .text("Essayer la 3D"),
                        ),
                )
                .child(
                    ul().class("hero-proof")
                        .attr("data-reveal", "up")
                        .attr("style", "--d:800ms")
                        .child(li().child(strong().text("100")).text(" Lighthouse"))
                        .child(li().child(strong().text("< 1 s")).text(" premier affichage"))
                        .child(li().child(strong().text("4,9/5")).text(" satisfaction client")),
                ),
        )
        .child(cube_css("hero-cube"))
        .child(
            a().class("scroll-hint")
                .href("#expertises")
                .attr("aria-label", "Défiler vers la suite")
                .child(span().class("mouse").child(span())),
        );

    let clients = section()
        .class("logos")
        .attr("aria-label", "Ils nous font confiance")
        .child(p().class("logos-title").text("Ils nous font confiance"))
        .child(defilant(&CLIENTS, false));

    let expertises = section()
        .class("section")
        .id("expertises")
        .child(
            div()
                .class("container")
                .child(titre_section(
                    "Expertises",
                    "Du concept au *lancement*, tout sous un même toit",
                    "Une équipe resserrée de designers, développeurs et artistes 3D. Un seul interlocuteur, une exigence constante.",
                ))
                .child(div().class("grid-3").children(SERVICES.iter().enumerate().map(|(i, s)| carte_service(i, s)))),
        );

    let atelier = section()
        .class("section section-alt")
        .id("atelier")
        .child(
            div()
                .class("container")
                .child(titre_section(
                    "Atelier 3D",
                    "Touchez, tournez, *configurez*",
                    "Cet objet est rendu en temps réel par notre moteur Rust compilé en WebAssembly. Choisissez une forme, une teinte, un rendu : tout se met à jour à 60 images par seconde.",
                ))
                .child(div().attr("data-reveal", "zoom").child(island("atelier", "", orbite::atelier))),
        );

    let methode = section().class("section").id("methode").child(
        div()
            .class("container")
            .child(titre_section(
                "Méthode",
                "Quatre étapes, *zéro* surprise",
                "Un processus éprouvé, des livrables à chaque étape et un planning tenu.",
            ))
            .child(
                ol().class("timeline")
                    .child(
                        div()
                            .class("timeline-line")
                            .attr("aria-hidden", "true")
                            .child(span()),
                    )
                    .children(ETAPES.iter().enumerate().map(|(i, (titre, duree, texte))| {
                        li().class("step")
                            .attr("data-reveal", if i % 2 == 0 { "left" } else { "right" })
                            .child(span().class("step-num").text(format!("0{}", i + 1)))
                            .child(
                                div()
                                    .class("step-card glow")
                                    .child(span().class("step-time").text(*duree))
                                    .child(h3().text(*titre))
                                    .child(p().class("muted").text(*texte)),
                            )
                    })),
            ),
    );

    let impact = section().class("section section-alt").child(
        div()
            .class("container")
            .child(titre_section(
                "Impact",
                "Des chiffres qui *parlent*",
                "Ce que nos sites changent, mesuré sur nos projets livrés depuis 2019.",
            ))
            .child(chiffres()),
    );

    let travaux = section()
        .class("section")
        .child(
            div()
                .class("container")
                .child(titre_section(
                    "Réalisations",
                    "Quelques projets *récents*",
                    "Sites vitrines, configurateurs 3D, applications : chaque projet est une pièce unique.",
                ))
                .child(div().class("grid-3 projects").children(PROJETS.iter().take(3).enumerate().map(|(i, p)| carte_projet(i, p))))
                .child(
                    div()
                        .class("center")
                        .attr("data-reveal", "up")
                        .child(
                            a().class("btn btn-ghost magnetic")
                                .href("/realisations")
                                .text("Toutes les réalisations")
                                .child(span().class("arrow").html(icone("fleche"))),
                        ),
                ),
        );

    let avis = section()
        .class("section section-alt testimonials")
        .child(div().class("container").child(titre_section(
            "Témoignages",
            "Ils en *parlent* mieux que nous",
            "Des équipes produit, des marques et des fondateurs qui nous ont confié leur image.",
        )))
        .child(rangee_avis(&AVIS[..3], false))
        .child(rangee_avis(&AVIS[3..], true));

    page(
        "/",
        "Studio créatif 3D & web",
        "Orbite conçoit des sites web immersifs : 3D temps réel, animations, PWA installables et performances irréprochables. Studio créatif à Lyon.",
        vec![
            hero.into(),
            clients.into(),
            expertises.into(),
            atelier.into(),
            methode.into(),
            impact.into(),
            travaux.into(),
            avis.into(),
            appel_final(),
        ],
    )
}

fn rangee_avis(avis: &[Avis], inverse: bool) -> Element {
    let cartes = || {
        avis.iter().map(|a| {
            figure()
                .class("quote")
                .child(
                    div()
                        .class("stars")
                        .attr("aria-label", "5 étoiles sur 5")
                        .text("★★★★★"),
                )
                .child(blockquote().child(p().text(a.texte)))
                .child(
                    figcaption()
                        .child(
                            span()
                                .class("avatar")
                                .attr("style", format!("--a:{}", a.couleur))
                                .text(a.initiales),
                        )
                        .child(
                            span()
                                .child(strong().text(a.nom))
                                .child(span().class("muted").text(a.role)),
                        ),
                )
        })
    };
    div()
        .class(if inverse { "quotes reverse" } else { "quotes" })
        .child(
            div()
                .class("quotes-track")
                .children(cartes())
                .children(cartes()),
        )
}

async fn services(_req: Request) -> Page {
    let detail = section().class("section").child(
        div().class("container").child(
            div().class("grid-3").children(
                SERVICES
                    .iter()
                    .enumerate()
                    .map(|(i, s)| carte_service(i, s)),
            ),
        ),
    );
    let tarifs = section()
        .class("section section-alt")
        .id("tarifs")
        .child(
            div()
                .class("container")
                .child(titre_section(
                    "Tarifs",
                    "Des forfaits *clairs*, sans surprise",
                    "Hébergement, maintenance et mises à jour de sécurité inclus. Sans engagement au-delà de 3 mois.",
                ))
                .child(island("tarifs", "", orbite::tarifs)),
        );
    let faq = section().class("section").id("faq").child(
        div()
            .class("container narrow")
            .child(titre_section(
                "FAQ",
                "Vos questions, nos *réponses*",
                "Vous ne trouvez pas la vôtre ? Écrivez-nous, nous répondons sous 24 h ouvrées.",
            ))
            .child(
                div()
                    .class("faq")
                    .children(FAQ.iter().enumerate().map(|(i, (q, r))| {
                        details()
                            .class("faq-item")
                            .attr("data-reveal", "up")
                            .attr("style", format!("--d:{}ms", i * 70))
                            .child(
                                summary()
                                    .child(span().text(*q))
                                    .child(span().class("plus").attr("aria-hidden", "true")),
                            )
                            .child(div().class("faq-body").child(p().text(*r)))
                    })),
            ),
    );
    page(
        "/services",
        "Services et tarifs",
        "Sites vitrines, expériences 3D WebGL, applications web, PWA, SEO et performance : les expertises et les forfaits d'Orbite.",
        vec![
            bandeau(
                "Services",
                "Ce que nous *faisons* de mieux",
                "Six expertises complémentaires, réunies pour livrer des sites qui marquent et qui performent.",
            ),
            detail.into(),
            tarifs.into(),
            faq.into(),
            appel_final(),
        ],
    )
}

async fn realisations(_req: Request) -> Page {
    let mut categories: Vec<&str> = vec!["Tous"];
    for p in PROJETS {
        if !categories.contains(&p.categorie) {
            categories.push(p.categorie);
        }
    }
    let grille = section().class("section").child(
        div()
            .class("container")
            .child(
                div()
                    .class("filters")
                    .attr("role", "toolbar")
                    .attr("aria-label", "Filtrer les projets")
                    .attr("data-reveal", "up")
                    .children(categories.iter().enumerate().map(|(i, c)| {
                        button()
                            .class("chip")
                            .attr("type", "button")
                            .attr("data-filter", if i == 0 { "*" } else { c })
                            .attr("aria-pressed", if i == 0 { "true" } else { "false" })
                            .text(*c)
                    })),
            )
            .child(
                div()
                    .class("grid-3 projects")
                    .children(PROJETS.iter().enumerate().map(|(i, p)| carte_projet(i, p))),
            ),
    );
    let jeu = section()
        .class("section section-alt")
        .id("physique")
        .child(
            div()
                .class("container")
                .child(titre_section(
                    "Laboratoire",
                    "Une pluie de *paquets*",
                    "Masse, gravité, rebonds et collisions : chaque boîte est un corps physique simulé en Rust. Lâchez-en, secouez le plateau, tournez autour.",
                ))
                .child(div().attr("data-reveal", "zoom").child(island("pluie", "", orbite::pluie))),
        );
    page(
        "/realisations",
        "Réalisations",
        "Configurateurs 3D, sites vitrines immersifs, applications web : une sélection de projets conçus par Orbite.",
        vec![
            bandeau(
                "Réalisations",
                "Des projets qui *sortent* de l'écran",
                "Une sélection de nos travaux récents, du site de marque au configurateur 3D temps réel.",
            ),
            grille.into(),
            jeu.into(),
            appel_final(),
        ],
    )
}

async fn studio(_req: Request) -> Page {
    let manifeste = section().class("section").child(
        div()
            .class("container split-2")
            .child(
                div()
                    .child(span().class("eyebrow").attr("data-reveal", "up").text("Manifeste"))
                    .child(mots(h2(), "Le web mérite *mieux* que des gabarits"))
                    .child(p().class("lead").attr("data-reveal", "up").text(
                        "Nous croyons qu'un site peut être à la fois beau, rapide, accessible et sûr. Que la 3D n'est pas un gadget quand elle raconte quelque chose. Et qu'un bon outil se fait oublier.",
                    ))
                    .child(p().class("muted").attr("data-reveal", "up").text(
                        "C'est pour cela que nous construisons avec Paquets, un framework Rust : des pages rendues sur le serveur, de la 3D compilée en WebAssembly, un bouclier de sécurité devant chaque requête.",
                    )),
            )
            .child(
                div()
                    .class("manifesto-visual")
                    .attr("data-reveal", "zoom")
                    .attr("aria-hidden", "true")
                    .child(div().class("planet"))
                    .child(div().class("ring r1"))
                    .child(div().class("ring r2"))
                    .child(cube_css("moon-cube")),
            ),
    );
    let valeurs = section().class("section section-alt").child(
        div()
            .class("container")
            .child(titre_section(
                "Valeurs",
                "Ce qui nous *guide*",
                "Quatre principes que nous appliquons à chaque ligne de code et à chaque pixel.",
            ))
            .child(
                div()
                    .class("grid-4")
                    .children(VALEURS.iter().enumerate().map(|(i, (ico, titre, texte))| {
                        div()
                            .class("card value glow tilt")
                            .attr("data-reveal", "up")
                            .attr("style", format!("--d:{}ms", i * 90))
                            .child(div().class("card-ico").html(icone(ico)))
                            .child(h3().text(*titre))
                            .child(p().class("muted").text(*texte))
                    })),
            ),
    );
    let equipe = section().class("section").id("equipe").child(
        div()
            .class("container")
            .child(titre_section(
                "Équipe",
                "Les *humains* derrière l'orbite",
                "Quatre métiers, une même obsession du détail.",
            ))
            .child(
                div()
                    .class("grid-4 team")
                    .children(EQUIPE.iter().enumerate().map(|(i, m)| {
                        article()
                            .class("member")
                            .attr("data-reveal", "up")
                            .attr(
                                "style",
                                format!(
                                    "--d:{}ms;--a:{};--b:{}",
                                    i * 90,
                                    m.couleurs.0,
                                    m.couleurs.1
                                ),
                            )
                            .child(
                                div()
                                    .class("member-card")
                                    .child(
                                        div()
                                            .class("member-front")
                                            .child(div().class("member-avatar").text(m.initiales))
                                            .child(h3().text(m.nom))
                                            .child(p().class("muted").text(m.role)),
                                    )
                                    .child(
                                        div()
                                            .class("member-back")
                                            .child(p().text(m.bio))
                                            .child(p().class("fun").text(m.detail)),
                                    ),
                            )
                    })),
            ),
    );
    let histoire = section().class("section section-alt").child(
        div()
            .class("container narrow")
            .child(titre_section(
                "Histoire",
                "Sept ans en *orbite*",
                "D'un duo dans un atelier lyonnais à un studio reconnu pour ses expériences 3D.",
            ))
            .child(
                ol().class("history")
                    .children(HISTOIRE.iter().map(|(annee, titre, texte)| {
                        li().attr("data-reveal", "left")
                            .child(span().class("year").text(*annee))
                            .child(
                                div()
                                    .child(h3().text(*titre))
                                    .child(p().class("muted").text(*texte)),
                            )
                    })),
            ),
    );
    page(
        "/studio",
        "Le studio",
        "Orbite est un studio créatif indépendant basé à Lyon : designers, développeurs Rust et artistes 3D au service d'un web plus beau et plus rapide.",
        vec![
            bandeau(
                "Le studio",
                "Petite équipe, *grande* ambition",
                "Designers, développeurs et artistes 3D réunis à Lyon pour faire du web un terrain d'expression.",
            ),
            manifeste.into(),
            valeurs.into(),
            equipe.into(),
            histoire.into(),
            appel_final(),
        ],
    )
}

async fn contact(req: Request) -> Page {
    let offre = req
        .query("offre")
        .map(|o| o.into_owned())
        .unwrap_or_default();
    let erreur = req.query("erreur").is_some();
    let choix = |valeur: &str, label: &str, choisi: bool| {
        let mut o = option()
            .attr("value", valeur.to_owned())
            .text(label.to_owned());
        if choisi {
            o = o.attr("selected", "");
        }
        o
    };
    let formulaire = form()
        .class("contact-form card")
        .attr("method", "post")
        .attr("action", "/contact")
        .attr("data-reveal", "up")
        .child(if erreur {
            p().class("alert").attr("role", "alert").text(
                "Merci de vérifier votre nom, votre e-mail et votre message (20 caractères minimum).",
            )
        } else {
            p().class("alert-none")
        })
        .child(
            div()
                .class("row-2")
                .child(champ("nom", "Nom complet", "text", "Camille Martin", true, "name"))
                .child(champ("email", "E-mail professionnel", "email", "camille@entreprise.fr", true, "email")),
        )
        .child(
            div()
                .class("row-2")
                .child(champ("entreprise", "Entreprise", "text", "Votre société", false, "organization"))
                .child(
                    div()
                        .class("field")
                        .child(label().attr("for", "offre").text("Offre envisagée"))
                        .child(
                            select()
                                .id("offre")
                                .attr("name", "offre")
                                .child(choix("", "Je ne sais pas encore", offre.is_empty()))
                                .children(["Essentiel", "Studio", "Orbite"].map(|o| choix(o, o, offre == o))),
                        ),
                ),
        )
        .child(
            fieldset()
                .class("field budgets")
                .child(legend().text("Budget"))
                .child(div().class("chips").children(BUDGETS.iter().enumerate().map(|(i, b)| {
                    let id = format!("budget-{i}");
                    label()
                        .class("chip-radio")
                        .attr("for", id.clone())
                        .child({
                            let mut r = input()
                                .id(id)
                                .attr("type", "radio")
                                .attr("name", "budget")
                                .attr("value", *b);
                            if i == 1 {
                                r = r.attr("checked", "");
                            }
                            r
                        })
                        .child(span().text(*b))
                }))),
        )
        .child(
            div()
                .class("field")
                .child(label().attr("for", "contenu").text("Votre projet"))
                .child(
                    textarea()
                        .id("contenu")
                        .attr("name", "contenu")
                        .attr("rows", "6")
                        .attr("minlength", "20")
                        .attr("maxlength", "4000")
                        .attr("required", "")
                        .attr("placeholder", "Objectifs, délais, inspirations…"),
                ),
        )
        .child(
            input()
                .class("hp")
                .attr("type", "text")
                .attr("name", "site_web")
                .attr("tabindex", "-1")
                .attr("autocomplete", "off")
                .attr("aria-hidden", "true"),
        )
        .child(
            div()
                .class("form-foot")
                .child(p().class("fine").text(
                    "Vos données servent uniquement à vous répondre. Voir notre politique de confidentialité.",
                ))
                .child(
                    button()
                        .class("btn btn-primary btn-lg magnetic")
                        .attr("type", "submit")
                        .text("Envoyer")
                        .child(span().class("arrow").html(icone("fleche"))),
                ),
        );
    let infos = aside()
        .class("contact-aside")
        .child(
            div()
                .class("card radar-card")
                .attr("data-reveal", "up")
                .child(
                    div()
                        .class("radar")
                        .attr("aria-hidden", "true")
                        .child(span().class("sweep"))
                        .child(span().class("blip")),
                )
                .child(h3().text("Le studio"))
                .child(
                    p().class("muted")
                        .text(format!("{RUE}, {CODE_POSTAL} {VILLE}")),
                )
                .child(
                    p().child(
                        a().class("u-link")
                            .href(format!("mailto:{EMAIL}"))
                            .text(EMAIL),
                    ),
                )
                .child(
                    p().child(
                        a().class("u-link")
                            .href(format!("tel:{}", TELEPHONE.replace(' ', "")))
                            .text(TELEPHONE),
                    ),
                ),
        )
        .child(
            div()
                .class("card")
                .attr("data-reveal", "up")
                .attr("style", "--d:120ms")
                .child(h3().text("Ce qui se passe ensuite"))
                .child(
                    ol().class("next-steps").children(
                        [
                            "Nous lisons votre message et revenons vers vous sous 24 h ouvrées.",
                            "Un appel découverte de 30 minutes pour cadrer le besoin.",
                            "Une proposition détaillée, chiffrée et planifiée sous 5 jours.",
                        ]
                        .map(|t| li().text(t)),
                    ),
                ),
        );
    page(
        "/contact",
        "Contact",
        "Parlez-nous de votre projet : site vitrine, expérience 3D ou application web. Réponse sous 24 h ouvrées.",
        vec![
            bandeau(
                "Contact",
                "Parlons de votre *projet*",
                "Quelques lignes suffisent. Nous revenons vers vous sous 24 h ouvrées avec des questions, des idées et un premier cadrage.",
            ),
            section()
                .class("section")
                .child(
                    div()
                        .class("container contact-grid")
                        .child(formulaire)
                        .child(infos),
                )
                .into(),
        ],
    )
}

fn champ(
    nom: &str,
    libelle: &str,
    genre: &str,
    exemple: &str,
    requis: bool,
    auto: &str,
) -> Element {
    let mut entree = input()
        .id(nom.to_owned())
        .attr("name", nom.to_owned())
        .attr("type", genre.to_owned())
        .attr("placeholder", exemple.to_owned())
        .attr("autocomplete", auto.to_owned())
        .attr("maxlength", "200");
    if requis {
        entree = entree.attr("required", "");
    }
    div()
        .class("field")
        .child(label().attr("for", nom.to_owned()).text(libelle.to_owned()))
        .child(entree)
}

async fn envoyer_contact(req: Request) -> Result<Redirect> {
    let f: FormulaireContact = req.form().await?;
    if !f.site_web.is_empty() {
        // Un robot : on fait comme si tout allait bien.
        return Ok(Redirect::see_other("/merci?type=contact"));
    }
    if f.nom.trim().is_empty() || !email_valide(&f.email) || f.contenu.trim().chars().count() < 20 {
        return Ok(Redirect::see_other("/contact?erreur=1#contenu"));
    }
    let mut message = Message {
        id: 0,
        nom: borne(&f.nom, 200),
        email: borne(&f.email, 254),
        entreprise: borne(&f.entreprise, 200),
        offre: borne(&f.offre, 40),
        budget: borne(&f.budget, 40),
        contenu: borne(&f.contenu, 4000),
        recu_le: maintenant(),
    };
    message.insert(req.db()).await?;
    Ok(Redirect::see_other("/merci?type=contact"))
}

async fn inscrire(req: Request) -> Result<Redirect> {
    let f: FormulaireNewsletter = req.form().await?;
    if !f.site_web.is_empty() {
        return Ok(Redirect::see_other("/merci?type=newsletter"));
    }
    if !email_valide(&f.email) {
        return Ok(Redirect::see_other("/merci?type=erreur"));
    }
    let email = borne(&f.email, 254).to_lowercase();
    let deja = Abonne::query()
        .filter(Abonne::EMAIL.eq(email.clone()))
        .all(req.db())
        .await?;
    if deja.is_empty() {
        let mut abonne = Abonne {
            id: 0,
            email,
            inscrit_le: maintenant(),
        };
        abonne.insert(req.db()).await?;
    }
    Ok(Redirect::see_other("/merci?type=newsletter"))
}

async fn merci(req: Request) -> Page {
    let (titre, texte, ok) = match req.query("type").as_deref() {
        Some("newsletter") => (
            "Bienvenue à *bord* !",
            "Votre inscription est confirmée. Prochaine lettre d'orbite : début du mois prochain.",
            true,
        ),
        Some("erreur") => (
            "Adresse *invalide*",
            "Cette adresse e-mail ne semble pas valide. Vous pouvez réessayer depuis le pied de page.",
            false,
        ),
        _ => (
            "Message *reçu* !",
            "Merci ! Nous lisons votre message et revenons vers vous sous 24 h ouvrées.",
            true,
        ),
    };
    page(
        "/merci",
        "Merci",
        "Merci pour votre message.",
        vec![
            section()
                .class("page-hero thanks")
                .child(div().class("aurora").attr("aria-hidden", "true"))
                .child(
                    div()
                        .class("container center")
                        .child(
                            div()
                                .class(if ok { "check-anim" } else { "check-anim bad" })
                                .attr("aria-hidden", "true")
                                .html(icone(if ok { "check" } else { "croix" })),
                        )
                        .child(mots(h1(), titre))
                        .child(p().class("lead").attr("data-reveal", "up").text(texte))
                        .child(
                            div()
                                .class("actions center")
                                .attr("data-reveal", "up")
                                .child(
                                    a().class("btn btn-primary magnetic")
                                        .href("/")
                                        .text("Retour à l'accueil"),
                                )
                                .child(
                                    a().class("btn btn-ghost magnetic")
                                        .href("/realisations")
                                        .text("Voir nos réalisations"),
                                ),
                        ),
                )
                .into(),
        ],
    )
    .seo(Seo::new().noindex(true))
}

/// Une page de texte : mentions légales, confidentialité.
fn page_texte(
    chemin: &str,
    titre: &str,
    description: &str,
    sections: &[(&'static str, &'static str)],
) -> Page {
    page(
        chemin,
        titre,
        description,
        vec![
            bandeau("Informations", titre, description),
            section()
                .class("section")
                .child(
                    div()
                        .class("container narrow prose")
                        .children(sections.iter().map(|(h, t)| {
                            div()
                                .attr("data-reveal", "up")
                                .child(h2().text(*h))
                                .children(t.split("\n\n").map(|para| p().text(para.to_owned())))
                        })),
                )
                .into(),
        ],
    )
}

async fn mentions(_req: Request) -> Page {
    page_texte(
        "/mentions-legales",
        "Mentions légales",
        "Éditeur, hébergeur et propriété intellectuelle du site Orbite.",
        &MENTIONS,
    )
}

async fn confidentialite(_req: Request) -> Page {
    page_texte(
        "/confidentialite",
        "Politique de confidentialité",
        "Quelles données nous collectons, pourquoi, combien de temps, et vos droits.",
        &CONFIDENTIALITE,
    )
}

async fn plan(_req: Request) -> Page {
    page(
        "/plan-du-site",
        "Plan du site",
        "Toutes les pages du site Orbite.",
        vec![
            bandeau(
                "Navigation",
                "Plan du *site*",
                "Toutes nos pages, en un coup d'œil.",
            ),
            section()
                .class("section")
                .child(
                    div()
                        .class("container narrow")
                        .child(ul().class("sitemap").children(PAGES.iter().map(
                            |(href, label)| {
                                li().attr("data-reveal", "left")
                                    .child(a().class("u-link").href(*href).text(*label))
                            },
                        ))),
                )
                .into(),
        ],
    )
}

async fn introuvable(_req: Request) -> Page {
    page(
        "/404",
        "Page introuvable",
        "Cette page s'est perdue dans l'espace.",
        vec![
            section()
                .class("page-hero lost")
                .child(div().class("aurora").attr("aria-hidden", "true"))
                .child(div().class("grid-floor").attr("aria-hidden", "true"))
                .child(
                    div()
                        .class("container center")
                        .child(p().class("glitch").attr("data-text", "404").text("404"))
                        .child(cube_css("lost-cube"))
                        .child(mots(h1(), "Perdu dans l'*espace*"))
                        .child(p().class("lead").text(
                            "Cette page a quitté notre orbite. Elle a peut-être été déplacée, ou n'a jamais existé.",
                        ))
                        .child(
                            div()
                                .class("actions center")
                                .child(a().class("btn btn-primary magnetic").href("/").text("Retour sur Terre"))
                                .child(a().class("btn btn-ghost magnetic").href("/contact").text("Nous écrire")),
                        ),
                )
                .into(),
        ],
    )
    .seo(Seo::new().noindex(true))
    .status(404)
}

async fn sitemap(_req: Request) -> Response {
    let base = site_url();
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );
    for (href, _) in PAGES {
        xml.push_str(&format!("<url><loc>{base}{href}</loc></url>"));
    }
    xml.push_str("</urlset>");
    Response::new()
        .header("content-type", "application/xml; charset=utf-8")
        .send(xml)
}

async fn robots(_req: Request) -> String {
    format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
        site_url()
    )
}

fn app() -> App {
    let mut app = App::new();
    app.get("/", accueil);
    app.get("/services", services);
    app.get("/realisations", realisations);
    app.get("/studio", studio);
    app.get("/contact", contact);
    app.post("/contact", envoyer_contact);
    app.post("/newsletter", inscrire);
    app.get("/merci", merci);
    app.get("/mentions-legales", mentions);
    app.get("/confidentialite", confidentialite);
    app.get("/plan-du-site", plan);
    app.get("/sitemap.xml", sitemap);
    app.get("/robots.txt", robots);
    app.model::<Message>();
    app.model::<Abonne>();
    app.health("/health");
    app.pwa(
        Pwa::new(STUDIO)
            .short_name("Orbite")
            .description("Studio créatif 3D & web : sites immersifs, rapides et installables.")
            .lang("fr")
            .start_url("/")
            .display("standalone")
            .theme_color("#0b0a14")
            .background_color("#0b0a14")
            .categories(["business", "design"])
            .icon("/icons/icon-192.png", "192x192", "image/png")
            .icon("/icons/icon-512.png", "512x512", "image/png")
            .maskable_icon("/icons/icon-maskable-512.png", "512x512", "image/png"),
    );
    app.fallback(introuvable);
    app
}

fn main() -> std::io::Result<()> {
    app().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn chaque_page_repond() {
        let client = app().test().await;
        for (chemin, _) in PAGES {
            let res = client.get(chemin).await;
            assert_eq!(res.status(), 200, "{chemin}");
            let html = res.text();
            assert!(html.contains("<html lang=\"fr\""), "{chemin}");
            assert!(html.contains("site-footer"), "{chemin}");
            assert!(html.contains("manifest.webmanifest"), "{chemin}");
        }
        assert_eq!(client.get("/nulle-part").await.status(), 404);
        assert_eq!(client.get("/manifest.webmanifest").await.status(), 200);
        assert_eq!(client.get("/sw.js").await.status(), 200);
        assert_eq!(client.get("/sitemap.xml").await.status(), 200);
        assert_eq!(client.get("/health").await.status(), 200);
    }

    #[tokio::test]
    async fn les_iles_sont_rendues() {
        let client = app().test().await;
        let accueil = client.get("/").await.text();
        assert!(accueil.contains("data-island=\"hero3d\""));
        assert!(accueil.contains("data-island=\"atelier\""));
        assert!(
            client
                .get("/realisations")
                .await
                .text()
                .contains("data-island=\"pluie\"")
        );
        assert!(
            client
                .get("/services")
                .await
                .text()
                .contains("data-island=\"tarifs\"")
        );
    }

    #[test]
    fn emails() {
        assert!(email_valide("camille@exemple.fr"));
        assert!(!email_valide("camille"));
        assert!(!email_valide("camille@exemple"));
        assert!(!email_valide("a b@exemple.fr"));
    }
}
