//! The table of contents of the documentation: categories and pages, in
//! reading order. The pages themselves are the `docs/<lang>/<slug>.md` files
//! of the repository (same slugs in every language).

use crate::i18n::Lang;

/// A category: titles `[fr, en, es]` and icon.
pub struct Category {
    pub title: [&'static str; 3],
    pub icon: &'static str,
}

pub struct Page {
    pub slug: &'static str,
    pub category: usize,
    pub title: [&'static str; 3],
    pub summary: [&'static str; 3],
}

impl Page {
    pub fn title(&self, lang: Lang) -> &'static str {
        lang.pick(self.title)
    }

    pub fn summary(&self, lang: Lang) -> &'static str {
        lang.pick(self.summary)
    }
}

pub const CATEGORIES: [Category; 5] = [
    Category {
        title: ["Démarrer", "Getting started", "Primeros pasos"],
        icon: "rocket",
    },
    Category {
        title: ["L'essentiel", "Essentials", "Lo esencial"],
        icon: "layers",
    },
    Category {
        title: ["Aller plus loin", "Going further", "Más allá"],
        icon: "spark",
    },
    Category {
        title: ["Déployer", "Deploy", "Desplegar"],
        icon: "cloud",
    },
    Category {
        title: ["Référence", "Reference", "Referencia"],
        icon: "book",
    },
];

pub const PAGES: [Page; 28] = [
    // ----- Getting started
    Page {
        slug: "introduction",
        category: 0,
        title: ["Introduction", "Introduction", "Introducción"],
        summary: [
            "Ce qu'est Paquets, ses quatre paquets, et comment lire cette documentation.",
            "What Paquets is, its four packages, and how to read these docs.",
            "Qué es Paquets, sus cuatro paquetes y cómo leer esta documentación.",
        ],
    },
    Page {
        slug: "installation",
        category: 0,
        title: ["Installation", "Installation", "Instalación"],
        summary: [
            "Installer Rust, la commande paquets et la cible WebAssembly.",
            "Install Rust, the paquets command and the WebAssembly target.",
            "Instala Rust, el comando paquets y el target WebAssembly.",
        ],
    },
    Page {
        slug: "quick-start",
        category: 0,
        title: ["Démarrage rapide", "Quick start", "Inicio rápido"],
        summary: [
            "Une app complète en dix minutes : page, API REST et base de données.",
            "A complete app in ten minutes: a page, a REST API and a database.",
            "Una app completa en diez minutos: una página, una API REST y una base de datos.",
        ],
    },
    Page {
        slug: "project-structure",
        category: 0,
        title: [
            "Structure d'un projet",
            "Project structure",
            "Estructura del proyecto",
        ],
        summary: [
            "Ce que crée paquets new, fichier par fichier.",
            "What paquets new creates, file by file.",
            "Lo que crea paquets new, archivo por archivo.",
        ],
    },
    // ----- Essentials
    Page {
        slug: "routing",
        category: 1,
        title: ["Routes", "Routing", "Enrutamiento"],
        summary: [
            "Routes, paramètres, routeurs et middlewares.",
            "Routes, parameters, routers and middleware.",
            "Rutas, parámetros, routers y middlewares.",
        ],
    },
    Page {
        slug: "requests-responses",
        category: 1,
        title: [
            "Requêtes et réponses",
            "Requests and responses",
            "Peticiones y respuestas",
        ],
        summary: [
            "Lire la requête, et tout ce qu'un handler peut renvoyer.",
            "Reading the request, and everything a handler can return.",
            "Leer la petición, y todo lo que un handler puede devolver.",
        ],
    },
    Page {
        slug: "pages",
        category: 1,
        title: ["Pages", "Pages", "Páginas"],
        summary: [
            "Des pages HTML rendues sur le serveur, avec SEO, thème et CSP.",
            "Server-rendered HTML pages, with SEO, a theme and a CSP.",
            "Páginas HTML renderizadas en el servidor, con SEO, tema y CSP.",
        ],
    },
    Page {
        slug: "database",
        category: 1,
        title: ["Base de données", "Database", "Base de datos"],
        summary: [
            "Modèles, requêtes, transactions, et les six bases prises en charge.",
            "Models, queries, transactions, and the six supported databases.",
            "Modelos, consultas, transacciones y las seis bases compatibles.",
        ],
    },
    Page {
        slug: "rest-api",
        category: 1,
        title: ["API REST", "REST API", "API REST"],
        summary: [
            "Une API REST complète en une ligne avec app.api.",
            "A complete REST API in one line with app.api.",
            "Una API REST completa en una línea con app.api.",
        ],
    },
    Page {
        slug: "security",
        category: 1,
        title: ["Sécurité", "Security", "Seguridad"],
        summary: [
            "Le bouclier par défaut, et comment le régler.",
            "The default shield, and how to tune it.",
            "El escudo por defecto, y cómo ajustarlo.",
        ],
    },
    // ----- Going further
    Page {
        slug: "islands",
        category: 2,
        title: ["Îlots", "Islands", "Islas"],
        summary: [
            "De l'interactivité dans le navigateur, composant par composant.",
            "Interactivity in the browser, one component at a time.",
            "Interactividad en el navegador, componente a componente.",
        ],
    },
    Page {
        slug: "websocket",
        category: 2,
        title: ["WebSocket", "WebSocket", "WebSocket"],
        summary: [
            "Le temps réel avec app.ws : chat, notifications, jeux.",
            "Real time with app.ws: chat, notifications, games.",
            "Tiempo real con app.ws: chat, notificaciones, juegos.",
        ],
    },
    Page {
        slug: "testing",
        category: 2,
        title: ["Tests", "Testing", "Pruebas"],
        summary: [
            "Tester toute l'app en mémoire avec le client de test.",
            "Test the whole app in memory with the test client.",
            "Prueba toda la app en memoria con el cliente de pruebas.",
        ],
    },
    Page {
        slug: "errors",
        category: 2,
        title: ["Erreurs", "Errors", "Errores"],
        summary: [
            "L'opérateur ?, les codes HTTP et les pages d'erreur.",
            "The ? operator, status codes and error pages.",
            "El operador ?, los códigos HTTP y las páginas de error.",
        ],
    },
    Page {
        slug: "configuration",
        category: 2,
        title: ["Configuration", "Configuration", "Configuración"],
        summary: [
            "Port, base de données, variables d'environnement et features Cargo.",
            "Port, database, environment variables and Cargo features.",
            "Puerto, base de datos, variables de entorno y features de Cargo.",
        ],
    },
    Page {
        slug: "performance",
        category: 2,
        title: ["Performances", "Performance", "Rendimiento"],
        summary: [
            "Les chiffres, d'où ils viennent, et comment garder une app rapide.",
            "The numbers, where they come from, and how to keep your app fast.",
            "Las cifras, de dónde salen y cómo mantener tu app rápida.",
        ],
    },
    Page {
        slug: "authentication",
        category: 2,
        title: ["Authentification", "Authentication", "Autenticación"],
        summary: [
            "Inscription, connexion, mot de passe oublié et codes à deux facteurs en une ligne.",
            "Sign-up, login, forgotten passwords and two-factor codes in one line.",
            "Registro, inicio de sesión, contraseña olvidada y códigos de dos factores en una línea.",
        ],
    },
    Page {
        slug: "admin",
        category: 2,
        title: [
            "Panneau d'administration",
            "Admin panel",
            "Panel de administración",
        ],
        summary: [
            "Lister, chercher, créer, modifier et supprimer les lignes de vos modèles.",
            "List, search, create, edit and delete the rows of your models.",
            "Listar, buscar, crear, editar y borrar las filas de tus modelos.",
        ],
    },
    Page {
        slug: "generators",
        category: 2,
        title: ["Générateurs", "Generators", "Generadores"],
        summary: [
            "make:model, make:api, make:page… : le code écrit et branché pour vous.",
            "make:model, make:api, make:page…: the code written and wired for you.",
            "make:model, make:api, make:page…: el código escrito y conectado por ti.",
        ],
    },
    Page {
        slug: "operations",
        category: 2,
        title: ["Exploitation", "Operations", "Operaciones"],
        summary: [
            "OpenAPI, liste des routes, migrations, santé, métriques et rechargement automatique.",
            "OpenAPI, the route list, migrations, health, metrics and live reload.",
            "OpenAPI, lista de rutas, migraciones, salud, métricas y recarga automática.",
        ],
    },
    // ----- Deploy
    Page {
        slug: "deploy-heroku-mobile",
        category: 3,
        title: [
            "Déployer sur Heroku depuis votre téléphone",
            "Deploy to Heroku from your phone",
            "Desplegar en Heroku desde el móvil",
        ],
        summary: [
            "En ligne sans ordinateur : le bouton, puis les déploiements automatiques.",
            "Go live without a computer: the button, then automatic deploys.",
            "En línea sin ordenador: el botón y luego los despliegues automáticos.",
        ],
    },
    Page {
        slug: "deploy-docker",
        category: 3,
        title: [
            "Déployer avec Docker",
            "Deploy with Docker",
            "Desplegar con Docker",
        ],
        summary: [
            "L'image de production et les plateformes qui la font tourner.",
            "The production image and the platforms that run it.",
            "La imagen de producción y las plataformas que la ejecutan.",
        ],
    },
    Page {
        slug: "publish-crates-io",
        category: 3,
        title: [
            "Publier sur crates.io",
            "Publish on crates.io",
            "Publicar en crates.io",
        ],
        summary: [
            "Publier une version depuis une release GitHub.",
            "Publish a version from a GitHub release.",
            "Publica una versión desde una release de GitHub.",
        ],
    },
    Page {
        slug: "automation",
        category: 3,
        title: ["Automatisation", "Automation", "Automatización"],
        summary: [
            "CI, fusion automatique dans master et déploiement sur Heroku.",
            "CI, automatic merge into master and Heroku deployment.",
            "CI, fusión automática en master y despliegue en Heroku.",
        ],
    },
    // ----- Reference
    Page {
        slug: "cli",
        category: 4,
        title: [
            "Ligne de commande",
            "Command line tool",
            "Herramienta de línea de comandos",
        ],
        summary: [
            "new, dev, build, doctor : toutes les commandes et options.",
            "new, dev, build, doctor: every command and option.",
            "new, dev, build, doctor: todos los comandos y opciones.",
        ],
    },
    Page {
        slug: "cheatsheet",
        category: 4,
        title: ["Aide-mémoire", "Cheat sheet", "Chuleta"],
        summary: [
            "Toute l'API sur une seule page.",
            "The whole API on a single page.",
            "Toda la API en una sola página.",
        ],
    },
    Page {
        slug: "packages",
        category: 4,
        title: [
            "Les quatre paquets",
            "The four packages",
            "Los cuatro paquetes",
        ],
        summary: [
            "vitesse, securite, memoire-orm et active : utiliser chacun directement.",
            "vitesse, securite, memoire-orm and active: using each one directly.",
            "vitesse, securite, memoire-orm y active: usar cada uno directamente.",
        ],
    },
    Page {
        slug: "faq",
        category: 4,
        title: ["FAQ", "FAQ", "Preguntas frecuentes"],
        summary: [
            "Les questions fréquentes, et ce que Paquets ne fait pas (encore).",
            "Common questions, and what Paquets does not do (yet).",
            "Preguntas habituales, y lo que Paquets no hace (todavía).",
        ],
    },
];

pub fn find(slug: &str) -> Option<(usize, &'static Page)> {
    PAGES.iter().enumerate().find(|(_, p)| p.slug == slug)
}

/// The pages of a category, in reading order.
pub fn pages_of(category: usize) -> impl Iterator<Item = &'static Page> {
    PAGES.iter().filter(move |p| p.category == category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_is_well_formed() {
        let mut slugs: Vec<&str> = PAGES.iter().map(|p| p.slug).collect();
        // Pages are grouped by category, in order.
        assert!(PAGES.windows(2).all(|w| w[0].category <= w[1].category));
        assert!(PAGES.iter().all(|p| p.category < CATEGORIES.len()));
        assert!((0..CATEGORIES.len()).all(|c| pages_of(c).count() >= 4));
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), PAGES.len(), "duplicate slug");
        for page in &PAGES {
            assert!(
                page.slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-'),
                "{}",
                page.slug
            );
            for lang in Lang::ALL {
                assert!(!page.title(lang).is_empty() && !page.summary(lang).is_empty());
            }
        }
    }

    #[test]
    fn index_matches_the_brief() {
        let by_category: Vec<Vec<&str>> = (0..CATEGORIES.len())
            .map(|c| pages_of(c).map(|p| p.slug).collect())
            .collect();
        assert_eq!(
            by_category,
            [
                vec![
                    "introduction",
                    "installation",
                    "quick-start",
                    "project-structure"
                ],
                vec![
                    "routing",
                    "requests-responses",
                    "pages",
                    "database",
                    "rest-api",
                    "security"
                ],
                vec![
                    "islands",
                    "websocket",
                    "testing",
                    "errors",
                    "configuration",
                    "performance",
                    "authentication",
                    "admin",
                    "generators",
                    "operations"
                ],
                vec![
                    "deploy-heroku-mobile",
                    "deploy-docker",
                    "publish-crates-io",
                    "automation"
                ],
                vec!["cli", "cheatsheet", "packages", "faq"],
            ]
        );
    }
}
