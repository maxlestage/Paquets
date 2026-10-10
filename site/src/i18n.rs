//! The languages of the site (English, French, Spanish): every text and the number
//! formats. The language of a page comes from its URL (`/en/`, `/fr/`, `/es/`).
//!
//! Each language is a [`Texts`] constant: forgetting a translation is a compile error.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Lang {
    Fr,
    En,
    Es,
}

impl Lang {
    /// In the order of the selector.
    pub const ALL: [Lang; 3] = [Lang::En, Lang::Fr, Lang::Es];

    pub fn code(self) -> &'static str {
        self.pick(["fr", "en", "es"])
    }

    pub fn name(self) -> &'static str {
        self.pick(["Français", "English", "Español"])
    }

    /// The Open Graph locale.
    pub fn locale(self) -> &'static str {
        self.pick(["fr_FR", "en_US", "es_ES"])
    }

    /// Picks the value of this language in `[fr, en, es]`.
    pub fn pick<T: Copy>(self, values: [T; 3]) -> T {
        values[match self {
            Lang::Fr => 0,
            Lang::En => 1,
            Lang::Es => 2,
        }]
    }

    /// `en`, `fr` or `es` exactly.
    pub fn parse(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    /// The language of a tag such as `fr-CA` or `ES`.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        Lang::parse(&tag.get(..2)?.to_ascii_lowercase())
    }

    pub fn texts(self) -> &'static Texts {
        self.pick([&FR, &EN, &ES])
    }
}

/// The preferred supported language of an `Accept-Language` header, English by default.
pub fn pick_lang(header: &str) -> Lang {
    let mut ranked: Vec<(Lang, f32)> = header
        .split(',')
        .filter_map(|part| {
            let mut pieces = part.trim().split(";q=");
            let lang = Lang::from_tag(pieces.next()?)?;
            let q = pieces
                .next()
                .and_then(|q| q.trim().parse().ok())
                .unwrap_or(1.0);
            Some((lang, q))
        })
        .filter(|(_, q)| *q > 0.0)
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.first().map_or(Lang::En, |r| r.0)
}

/// Formats a number for the language: `2 781 541` / `2,781,541` /
/// `2.781.541`, and `2,78` / `2.78`.
pub fn number(lang: Lang, value: f64, decimals: usize) -> String {
    let (group, point) = lang.pick([('\u{202f}', ','), (',', '.'), ('.', ',')]);
    let s = format!("{value:.decimals$}");
    let (int, frac) = s
        .split_once('.')
        .map_or((s.as_str(), None), |(i, f)| (i, Some(f)));
    let mut out = String::new();
    let digits: Vec<char> = int.chars().collect();
    // Spanish only groups numbers of five digits or more.
    let grouped = digits.len() > 4 || (digits.len() == 4 && lang != Lang::Es);
    for (i, c) in digits.iter().enumerate() {
        if grouped && i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(group);
        }
        out.push(*c);
    }
    if let Some(f) = frac {
        out.push(point);
        out.push_str(f);
    }
    out
}

/// A title and its text.
pub struct Item {
    pub title: &'static str,
    pub text: &'static str,
}

const fn item(title: &'static str, text: &'static str) -> Item {
    Item { title, text }
}

/// The texts of one of the four packages.
pub struct PkgText {
    pub role: &'static str,
    pub text: &'static str,
    /// What its key figure measures.
    pub figure: &'static str,
}

/// The texts of a code sample of the showcase.
pub struct SampleText {
    pub label: &'static str,
    pub caption: &'static str,
    /// The comments of the code, in the order of the `§N` markers.
    pub comments: &'static [&'static str],
}

pub struct Texts {
    // Meta
    pub meta_title: &'static str,
    pub docs_meta: &'static str,
    // Navigation
    pub nav_links: [&'static str; 4],
    pub nav_docs: &'static str,
    pub nav_aria: &'static str,
    pub brand_aria: &'static str,
    pub menu: &'static str,
    pub lang_aria: &'static str,
    pub loader: &'static str,
    // Hero
    pub hero_badge: &'static str,
    pub hero_title: [&'static str; 3],
    /// The line of the title painted with the gradient.
    pub hero_accent: usize,
    pub hero_lead: &'static str,
    pub hero_start: &'static str,
    pub hero_github: &'static str,
    pub hero_scene: &'static str,
    pub copy: &'static str,
    pub copied: &'static str,
    pub copy_aria: &'static str,
    pub scroll: &'static str,
    pub scroll_aria: &'static str,
    // Marquee
    pub marquee_aria: &'static str,
    pub marquee: [&'static str; 8],
    // Packages
    pub pkg_label: &'static str,
    pub pkg_title: &'static str,
    pub pkg_lead: &'static str,
    pub packages: [PkgText; 4],
    pub pkg_repo: &'static str,
    pub pkg_alone: &'static str,
    // Code showcase
    pub code_label: &'static str,
    pub code_title: &'static str,
    pub code_lead: &'static str,
    pub samples: [SampleText; 5],
    // Features
    pub features_label: &'static str,
    pub features_title: &'static str,
    pub features_lead: &'static str,
    pub features: [Item; 8],
    // Numbers
    pub numbers_label: &'static str,
    pub numbers_title: &'static str,
    pub numbers: [Item; 6],
    // From zero to production
    pub journey_label: &'static str,
    pub journey_title: &'static str,
    pub journey_lead: &'static str,
    pub journey: [Item; 4],
    pub term_out: [&'static str; 4],
    pub term_replay: &'static str,
    // Deploy from your phone
    pub deploy_label: &'static str,
    pub deploy_title: &'static str,
    pub deploy_lead: &'static str,
    pub deploy_steps: [Item; 3],
    pub deploy_button_alt: &'static str,
    pub deploy_note: &'static str,
    pub deploy_guide: &'static str,
    pub phone: [&'static str; 8],
    // Docs preview
    pub docs_label: &'static str,
    pub docs_home_title: &'static str,
    pub docs_home_lead: &'static str,
    pub docs_browse: &'static str,
    // Call to action
    pub cta_title: [&'static str; 2],
    pub cta_start: &'static str,
    pub cta_star: &'static str,
    // Footer
    pub footer_tagline: &'static str,
    pub footer_cols: [&'static str; 4],
    pub footer_product: [&'static str; 5],
    pub footer_resources: [&'static str; 4],
    pub footer_made: &'static str,
    pub footer_lang: &'static str,
    pub totop: &'static str,
    pub totop_aria: &'static str,
    pub totop_cursor: &'static str,
    // Documentation
    pub docs_title: &'static str,
    pub docs_lead: &'static str,
    pub search: &'static str,
    pub docs_search: &'static str,
    pub docs_search_hint: &'static str,
    pub docs_no_results: &'static str,
    pub docs_toc: &'static str,
    pub docs_prev: &'static str,
    pub docs_next: &'static str,
    pub docs_edit: &'static str,
    pub docs_missing: &'static str,
    pub docs_menu: &'static str,
    pub docs_api: &'static str,
    pub docs_quick: &'static str,
    pub docs_pages: &'static str,
    pub docs_fallback: &'static str,
    pub callouts: [&'static str; 5],
    // Other pages
    pub demo: &'static str,
    pub skip: &'static str,
    pub nf_title: &'static str,
    pub nf_text: &'static str,
    pub nf_home: &'static str,
}

impl Texts {
    /// Every string of the language, in a fixed order (the destructuring
    /// has no `..`: adding a field without listing it here does not compile).
    #[cfg(test)]
    pub fn strings(&self) -> Vec<&'static str> {
        let Texts {
            meta_title,
            docs_meta,
            nav_links,
            nav_docs,
            nav_aria,
            brand_aria,
            menu,
            lang_aria,
            loader,
            hero_badge,
            hero_title,
            hero_accent: _,
            hero_lead,
            hero_start,
            hero_github,
            hero_scene,
            copy,
            copied,
            copy_aria,
            scroll,
            scroll_aria,
            marquee_aria,
            marquee,
            pkg_label,
            pkg_title,
            pkg_lead,
            packages,
            pkg_repo,
            pkg_alone,
            code_label,
            code_title,
            code_lead,
            samples,
            features_label,
            features_title,
            features_lead,
            features,
            numbers_label,
            numbers_title,
            numbers,
            journey_label,
            journey_title,
            journey_lead,
            journey,
            term_out,
            term_replay,
            deploy_label,
            deploy_title,
            deploy_lead,
            deploy_steps,
            deploy_button_alt,
            deploy_note,
            deploy_guide,
            phone,
            docs_label,
            docs_home_title,
            docs_home_lead,
            docs_browse,
            cta_title,
            cta_start,
            cta_star,
            footer_tagline,
            footer_cols,
            footer_product,
            footer_resources,
            footer_made,
            footer_lang,
            totop,
            totop_aria,
            totop_cursor,
            docs_title,
            docs_lead,
            search,
            docs_search,
            docs_search_hint,
            docs_no_results,
            docs_toc,
            docs_prev,
            docs_next,
            docs_edit,
            docs_missing,
            docs_menu,
            docs_api,
            docs_quick,
            docs_pages,
            docs_fallback,
            callouts,
            demo,
            skip,
            nf_title,
            nf_text,
            nf_home,
        } = self;
        let mut out: Vec<&'static str> = vec![
            meta_title,
            docs_meta,
            nav_docs,
            nav_aria,
            brand_aria,
            menu,
            lang_aria,
            loader,
            hero_badge,
            hero_lead,
            hero_start,
            hero_github,
            hero_scene,
            copy,
            copied,
            copy_aria,
            scroll,
            scroll_aria,
            marquee_aria,
            pkg_label,
            pkg_title,
            pkg_lead,
            pkg_repo,
            pkg_alone,
            code_label,
            code_title,
            code_lead,
            features_label,
            features_title,
            features_lead,
            numbers_label,
            numbers_title,
            journey_label,
            journey_title,
            journey_lead,
            term_replay,
            deploy_label,
            deploy_title,
            deploy_lead,
            deploy_button_alt,
            deploy_note,
            deploy_guide,
            docs_label,
            docs_home_title,
            docs_home_lead,
            docs_browse,
            cta_start,
            cta_star,
            footer_tagline,
            footer_made,
            footer_lang,
            totop,
            totop_aria,
            totop_cursor,
            docs_title,
            docs_lead,
            search,
            docs_search,
            docs_search_hint,
            docs_no_results,
            docs_toc,
            docs_prev,
            docs_next,
            docs_edit,
            docs_missing,
            docs_menu,
            docs_api,
            docs_quick,
            docs_pages,
            docs_fallback,
            demo,
            skip,
            nf_title,
            nf_text,
            nf_home,
        ];
        out.extend(nav_links);
        out.extend(hero_title);
        out.extend(marquee);
        for p in packages {
            out.extend([p.role, p.text, p.figure]);
        }
        for s in samples {
            out.extend([s.label, s.caption]);
            out.extend(s.comments);
        }
        for i in features
            .iter()
            .chain(numbers)
            .chain(journey)
            .chain(deploy_steps)
        {
            out.extend([i.title, i.text]);
        }
        out.extend(term_out);
        out.extend(phone);
        out.extend(cta_title);
        out.extend(footer_cols);
        out.extend(footer_product);
        out.extend(footer_resources);
        out.extend(callouts);
        out
    }
}

pub const EN: Texts = Texts {
    meta_title: "Paquets — the simplest full-stack framework for Rust",
    docs_meta: "Paquets documentation",
    nav_links: ["Packages", "Code", "Features", "Deploy"],
    nav_docs: "Docs",
    nav_aria: "Main navigation",
    brand_aria: "Paquets, back to the top",
    menu: "Open the menu",
    lang_aria: "Language",
    loader: "Unpacking",
    hero_badge: "v0.1 — 4 packages, 1 crate",
    hero_title: ["The simplest", "full-stack framework", "for Rust."],
    hero_accent: 0,
    hero_lead: "One crate, one import, one App. A fast server, security by default, any database, a REST API, SEO pages and WebAssembly islands — already wired together.",
    hero_start: "Get started",
    hero_github: "GitHub",
    hero_scene: "Four cubes — vitesse, securite, memoire-orm and active — fly into a single box: Paquets.",
    copy: "Copy",
    copied: "Copied!",
    copy_aria: "Copy the command",
    scroll: "Scroll",
    scroll_aria: "Scroll down",
    marquee_aria: "What Paquets brings",
    marquee: [
        "Simplicity",
        "Secure by default",
        "Any database",
        "SEO pages",
        "WebAssembly islands",
        "Deploy from your phone",
        "Fully tested",
        "Ready for crates.io",
    ],
    pkg_label: "Under the hood",
    pkg_title: "4 packages, 1 framework.",
    pkg_lead: "Paquets assembles four crates by the same author. Each one is fast, tested and useful on its own — together, they make a full-stack framework.",
    packages: [
        PkgText {
            role: "HTTP server",
            text: "An Express-style API: app.get, req.param, middleware, Router, WebSocket, HTTP/3. Faster than actix-web and axum.",
            figure: "requests per second on GET /",
        },
        PkgText {
            role: "Security shield",
            text: "WAF (SQLi, XSS, RCE, Log4Shell, SSRF…), rate limiting, CSRF, security headers and CSP, bot blocker, hardened cookies.",
            figure: "for the whole security pipeline",
        },
        PkgText {
            role: "Type-safe ORM",
            text: "Async models and queries checked at compile time, on memory, SQLite, PostgreSQL, MySQL/MariaDB, Redis and MongoDB.",
            figure: "databases, one API",
        },
        PkgText {
            role: "Front end",
            text: "Components are Rust functions with use_state. Server rendering with complete SEO tags, and WebAssembly islands for interactivity.",
            figure: "gzip for a counter island",
        },
    ],
    pkg_repo: "Repository",
    pkg_alone: "Each package also works on its own.",
    code_label: "Show me the code",
    code_title: "Less code. Same power.",
    code_lead: "Everything below is real Rust that compiles today — the whole API comes with a single import.",
    samples: [
        SampleText {
            label: "Hello world",
            caption: "One import, one App. Security, logs, the database and static files are already set up: there is nothing to configure.",
            comments: &[
                "security, logs, database, static files",
                "http://localhost:3000, or $PORT on Heroku",
            ],
        },
        SampleText {
            label: "REST API",
            caption: "One line for a complete REST API: paginated list, read, create, replace, update, delete. The ? operator turns every error into the right status code.",
            comments: &[
                "table \"todo\", created at startup",
                "primary key, auto-increment",
                "400 if it is not a number",
                "404 if it does not exist",
                "GET, POST, PUT, PATCH, DELETE",
            ],
        },
        SampleText {
            label: "Pages",
            caption: "Pages rendered on the server, with complete SEO tags, a clean default theme and their own Content Security Policy — in plain Rust.",
            comments: &[
                "server-rendered HTML: title, Open Graph, CSP, theme",
                "shown in search results and link previews",
            ],
        },
        SampleText {
            label: "Islands",
            caption: "Interactivity only where you need it: a component is a Rust function, rendered on the server and brought to life in the browser by WebAssembly.",
            comments: &[
                "shared by the server and the browser",
                "browser side: 36.8 kB gzip for this counter",
                "the island is rendered on the server, then hydrated",
            ],
        },
        SampleText {
            label: "Tests",
            caption: "Test the whole stack — routes, security, database, pages — in memory: no port, no logs, a fresh database every time.",
            comments: &[
                "in memory: no port, no logs, a fresh database",
                "201 Created",
            ],
        },
    ],
    features_label: "Features",
    features_title: "Everything you need. Nothing to wire.",
    features_lead: "Paquets makes the decisions you would make anyway, so you can start building right away.",
    features: [
        item(
            "Simplicity",
            "One crate, one import, one App. Sensible defaults, and everything stays changeable.",
        ),
        item(
            "Secure by default",
            "WAF, rate limiting, CSRF, security headers and hardened cookies in front of every request, in about 6 µs.",
        ),
        item(
            "Any database",
            "In memory, SQLite, PostgreSQL, MySQL/MariaDB, Redis or MongoDB: set DATABASE_URL and keep the same code.",
        ),
        item(
            "SEO-ready pages",
            "Server-rendered HTML with title, description, Open Graph, canonical URL and a strict CSP.",
        ),
        item(
            "WebAssembly islands",
            "Add interactivity one component at a time. The bundle only loads on the pages that need it.",
        ),
        item(
            "Deploy from your phone",
            "A Deploy to Heroku button, or automatic deploys on every push to master, set up from the GitHub app.",
        ),
        item(
            "Tested",
            "An in-memory test client for the whole stack, and a CI that checks formatting, lints, tests and the build.",
        ),
        item(
            "Ready for crates.io",
            "Publish with a GitHub release: the crate on crates.io, the API reference on docs.rs.",
        ),
    ],
    numbers_label: "By the numbers",
    numbers_title: "Fast by design.",
    numbers: [
        item(
            "requests per second",
            "GET / on two cores — ahead of actix-web and axum.",
        ),
        item(
            "pipelined requests per second",
            "HTTP/1.1 pipelining, still on two cores.",
        ),
        item(
            "security pipeline",
            "WAF, rate limiting, CSRF and headers: the whole shield, per request.",
        ),
        item(
            "island bundle",
            "Gzipped WebAssembly for a counter island.",
        ),
        item(
            "databases",
            "From in-memory to MongoDB, with a single ORM.",
        ),
        item(
            "languages",
            "The documentation exists in English, French and Spanish.",
        ),
    ],
    journey_label: "From zero to production",
    journey_title: "Four steps. Not one more.",
    journey_lead: "Create, develop, push: GitHub merges, Heroku deploys. You can even take the last steps from your phone.",
    journey: [
        item(
            "Create",
            "A complete project: server, pages, islands, Dockerfile, Heroku config and GitHub workflows.",
        ),
        item(
            "Develop",
            "Builds the islands if there are any, then runs your app on localhost:3000.",
        ),
        item(
            "Push",
            "The CI runs the tests, then merges your branch into master automatically.",
        ),
        item(
            "Go live",
            "Every push to master is deployed to Heroku through the Platform API. No CLI required.",
        ),
    ],
    term_out: [
        "Installed the paquets command",
        "Created my-app: server, pages, islands, Docker, CI",
        "Listening on http://localhost:3000",
        "Tests passed · merged into master · live on Heroku",
    ],
    term_replay: "Replay",
    deploy_label: "Deploy from your phone",
    deploy_title: "Ship it from the couch.",
    deploy_lead: "No laptop required: one button creates your app on Heroku, then every push to master goes live on its own.",
    deploy_steps: [
        item(
            "Tap the button",
            "Deploy to Heroku creates the app from the repository, built with Docker, with Heroku Postgres if you want it.",
        ),
        item(
            "Connect GitHub",
            "Add the HEROKU_API_KEY secret and the HEROKU_APP_NAME variable, right from the GitHub app.",
        ),
        item(
            "Push, it's live",
            "Every push to master is deployed automatically by a GitHub workflow.",
        ),
    ],
    deploy_button_alt: "Deploy to Heroku",
    deploy_note: "Heroku has no free plan: an Eco dyno costs about $5 a month.",
    deploy_guide: "Read the deployment guide",
    phone: [
        "Deploy to Heroku",
        "my-paquets-app",
        "Building with Docker…",
        "Your app is live",
        "Open app",
        "App name",
        "Region",
        "Europe",
    ],
    docs_label: "Documentation",
    docs_home_title: "Learn it in an afternoon.",
    docs_home_lead: "24 pages in English, French and Spanish, from your first route to production.",
    docs_browse: "Browse the docs",
    cta_title: ["Ready to", "unpack?"],
    cta_start: "Get started",
    cta_star: "Star on GitHub",
    footer_tagline: "The simplest full-stack framework for Rust.",
    footer_cols: ["Product", "Documentation", "Packages", "Resources"],
    footer_product: ["Packages", "Code", "Features", "Numbers", "Deploy"],
    footer_resources: ["GitHub", "crates.io", "docs.rs", "Deploy to Heroku"],
    footer_made: "Made with Rust, Yew & WebAssembly",
    footer_lang: "Language",
    totop: "BACK TO TOP • BACK TO TOP •",
    totop_aria: "Back to top",
    totop_cursor: "Top",
    docs_title: "Documentation",
    docs_lead: "Everything you need to build, test and deploy a Paquets app, step by step.",
    search: "Search",
    docs_search: "Search the docs…",
    docs_search_hint: "Press / to search",
    docs_no_results: "No results for “{}”.",
    docs_toc: "On this page",
    docs_prev: "Previous",
    docs_next: "Next",
    docs_edit: "Edit this page on GitHub",
    docs_missing: "This page is not available in this language yet.",
    docs_menu: "Contents",
    docs_api: "Full API reference (docs.rs)",
    docs_quick: "Start here",
    docs_pages: "{} pages",
    docs_fallback: "This page is not translated yet: here is the English version.",
    callouts: ["Note", "Tip", "Important", "Warning", "Caution"],
    demo: "Live demo",
    skip: "Skip to content",
    nf_title: "This box is empty.",
    nf_text: "The page you are looking for does not exist, or it has moved. Everything else is still packed and ready.",
    nf_home: "Back to the home page",
};

pub const FR: Texts = Texts {
    meta_title: "Paquets — le framework full-stack le plus simple pour Rust",
    docs_meta: "Documentation de Paquets",
    nav_links: ["Paquets", "Code", "Fonctionnalités", "Déployer"],
    nav_docs: "Docs",
    nav_aria: "Navigation principale",
    brand_aria: "Paquets, retour en haut",
    menu: "Ouvrir le menu",
    lang_aria: "Langue",
    loader: "Déballage",
    hero_badge: "v0.1 — 4 paquets, 1 crate",
    hero_title: ["Le framework", "full-stack le plus", "simple pour Rust."],
    hero_accent: 2,
    hero_lead: "Une crate, un import, une App. Un serveur rapide, la sécurité par défaut, la base de données de votre choix, une API REST, des pages SEO et des îlots WebAssembly — déjà branchés ensemble.",
    hero_start: "Commencer",
    hero_github: "GitHub",
    hero_scene: "Quatre cubes — vitesse, securite, memoire-orm et active — s'envolent dans une seule boîte : Paquets.",
    copy: "Copier",
    copied: "Copié !",
    copy_aria: "Copier la commande",
    scroll: "Défiler",
    scroll_aria: "Défiler vers le bas",
    marquee_aria: "Ce qu'apporte Paquets",
    marquee: [
        "Simplicité",
        "Sécurisé par défaut",
        "Toutes les bases",
        "Pages SEO",
        "Îlots WebAssembly",
        "Déployé depuis un téléphone",
        "Entièrement testé",
        "Prêt pour crates.io",
    ],
    pkg_label: "Sous le capot",
    pkg_title: "4 paquets, 1 framework.",
    pkg_lead: "Paquets assemble quatre crates du même auteur. Chacune est rapide, testée et utile toute seule — ensemble, elles forment un framework full-stack.",
    packages: [
        PkgText {
            role: "Serveur HTTP",
            text: "Une API à la Express : app.get, req.param, middlewares, Router, WebSocket, HTTP/3. Plus rapide qu'actix-web et axum.",
            figure: "requêtes par seconde sur GET /",
        },
        PkgText {
            role: "Bouclier de sécurité",
            text: "WAF (SQLi, XSS, RCE, Log4Shell, SSRF…), limitation de débit, CSRF, en-têtes de sécurité et CSP, blocage des robots, cookies durcis.",
            figure: "pour tout le pipeline de sécurité",
        },
        PkgText {
            role: "ORM typé",
            text: "Modèles et requêtes asynchrones vérifiés à la compilation, en mémoire, SQLite, PostgreSQL, MySQL/MariaDB, Redis et MongoDB.",
            figure: "bases de données, une seule API",
        },
        PkgText {
            role: "Front-end",
            text: "Les composants sont des fonctions Rust avec use_state. Rendu serveur avec toutes les balises SEO, et îlots WebAssembly pour l'interactivité.",
            figure: "gzip pour un îlot compteur",
        },
    ],
    pkg_repo: "Dépôt",
    pkg_alone: "Chaque paquet s'utilise aussi tout seul.",
    code_label: "Montrez-moi le code",
    code_title: "Moins de code. Autant de puissance.",
    code_lead: "Tout ce qui suit est du vrai Rust qui compile aujourd'hui — toute l'API arrive avec un seul import.",
    samples: [
        SampleText {
            label: "Hello world",
            caption: "Un import, une App. La sécurité, les logs, la base de données et les fichiers statiques sont déjà en place : rien à configurer.",
            comments: &[
                "sécurité, logs, base de données, fichiers statiques",
                "http://localhost:3000, ou $PORT sur Heroku",
            ],
        },
        SampleText {
            label: "API REST",
            caption: "Une ligne pour une API REST complète : liste paginée, lecture, création, remplacement, modification, suppression. Avec ?, chaque erreur devient le bon code HTTP.",
            comments: &[
                "table « todo », créée au démarrage",
                "clé primaire, auto-incrémentée",
                "400 si ce n'est pas un nombre",
                "404 s'il n'existe pas",
                "GET, POST, PUT, PATCH, DELETE",
            ],
        },
        SampleText {
            label: "Pages",
            caption: "Des pages rendues sur le serveur, avec toutes les balises SEO, un thème soigné par défaut et leur propre Content Security Policy — en Rust, tout simplement.",
            comments: &[
                "HTML rendu sur le serveur : titre, Open Graph, CSP, thème",
                "affichée dans les résultats de recherche et les aperçus",
            ],
        },
        SampleText {
            label: "Îlots",
            caption: "De l'interactivité seulement là où il en faut : un composant est une fonction Rust, rendue sur le serveur puis animée dans le navigateur par WebAssembly.",
            comments: &[
                "partagé par le serveur et le navigateur",
                "côté navigateur : 36,8 ko gzip pour ce compteur",
                "l'îlot est rendu sur le serveur, puis hydraté",
            ],
        },
        SampleText {
            label: "Tests",
            caption: "Testez toute la pile — routes, sécurité, base de données, pages — en mémoire : pas de port, pas de logs, une base neuve à chaque fois.",
            comments: &[
                "en mémoire : pas de port, pas de logs, une base neuve",
                "201 Created",
            ],
        },
    ],
    features_label: "Fonctionnalités",
    features_title: "Tout ce qu'il faut. Rien à brancher.",
    features_lead: "Paquets prend les décisions que vous auriez prises de toute façon : vous construisez dès la première minute.",
    features: [
        item(
            "Simplicité",
            "Une crate, un import, une App. De bons réglages par défaut, et tout reste modifiable.",
        ),
        item(
            "Sécurisé par défaut",
            "WAF, limitation de débit, CSRF, en-têtes de sécurité et cookies durcis devant chaque requête, en 6 µs environ.",
        ),
        item(
            "Toutes les bases",
            "En mémoire, SQLite, PostgreSQL, MySQL/MariaDB, Redis ou MongoDB : changez DATABASE_URL, gardez le même code.",
        ),
        item(
            "Pages prêtes pour le SEO",
            "Du HTML rendu sur le serveur avec titre, description, Open Graph, URL canonique et une CSP stricte.",
        ),
        item(
            "Îlots WebAssembly",
            "Ajoutez de l'interactivité composant par composant. Le bundle n'est chargé que par les pages qui en ont besoin.",
        ),
        item(
            "Déployé depuis un téléphone",
            "Un bouton Deploy to Heroku, ou un déploiement automatique à chaque push sur master, réglé depuis l'app GitHub.",
        ),
        item(
            "Testé",
            "Un client de test en mémoire pour toute la pile, et une CI qui vérifie le format, les lints, les tests et le build.",
        ),
        item(
            "Prêt pour crates.io",
            "Publication par une release GitHub : la crate sur crates.io, la référence de l'API sur docs.rs.",
        ),
    ],
    numbers_label: "En chiffres",
    numbers_title: "Rapide par conception.",
    numbers: [
        item(
            "requêtes par seconde",
            "GET / sur deux cœurs — devant actix-web et axum.",
        ),
        item(
            "requêtes en pipeline par seconde",
            "Pipelining HTTP/1.1, toujours sur deux cœurs.",
        ),
        item(
            "pipeline de sécurité",
            "WAF, limitation de débit, CSRF et en-têtes : tout le bouclier, par requête.",
        ),
        item(
            "bundle d'un îlot",
            "WebAssembly compressé en gzip pour un îlot compteur.",
        ),
        item(
            "bases de données",
            "De la mémoire à MongoDB, avec un seul ORM.",
        ),
        item(
            "langues",
            "La documentation existe en français, en anglais et en espagnol.",
        ),
    ],
    journey_label: "De zéro à la production",
    journey_title: "Quatre étapes. Pas une de plus.",
    journey_lead: "Créez, développez, poussez : GitHub fusionne, Heroku déploie. Les dernières étapes se font même depuis un téléphone.",
    journey: [
        item(
            "Créer",
            "Un projet complet : serveur, pages, îlots, Dockerfile, configuration Heroku et workflows GitHub.",
        ),
        item(
            "Développer",
            "Compile les îlots s'il y en a, puis lance votre app sur localhost:3000.",
        ),
        item(
            "Pousser",
            "La CI lance les tests, puis fusionne votre branche dans master automatiquement.",
        ),
        item(
            "Mettre en ligne",
            "Chaque push sur master est déployé sur Heroku par l'API Platform. Aucune CLI nécessaire.",
        ),
    ],
    term_out: [
        "Commande paquets installée",
        "my-app créé : serveur, pages, îlots, Docker, CI",
        "En écoute sur http://localhost:3000",
        "Tests OK · fusionné dans master · en ligne sur Heroku",
    ],
    term_replay: "Rejouer",
    deploy_label: "Déployer depuis un téléphone",
    deploy_title: "Mettez en ligne depuis le canapé.",
    deploy_lead: "Pas besoin d'ordinateur : un bouton crée votre app sur Heroku, puis chaque push sur master part en ligne tout seul.",
    deploy_steps: [
        item(
            "Touchez le bouton",
            "Deploy to Heroku crée l'app à partir du dépôt, construite avec Docker, avec Heroku Postgres si vous le souhaitez.",
        ),
        item(
            "Reliez GitHub",
            "Ajoutez le secret HEROKU_API_KEY et la variable HEROKU_APP_NAME, directement depuis l'app GitHub.",
        ),
        item(
            "Poussez, c'est en ligne",
            "Chaque push sur master est déployé automatiquement par un workflow GitHub.",
        ),
    ],
    deploy_button_alt: "Déployer sur Heroku",
    deploy_note: "Heroku n'a pas d'offre gratuite : un dyno Eco coûte environ 5 $ par mois.",
    deploy_guide: "Lire le guide de déploiement",
    phone: [
        "Déployer sur Heroku",
        "mon-app-paquets",
        "Construction avec Docker…",
        "Votre app est en ligne",
        "Ouvrir l'app",
        "Nom de l'app",
        "Région",
        "Europe",
    ],
    docs_label: "Documentation",
    docs_home_title: "Maîtrisé en un après-midi.",
    docs_home_lead: "24 pages en français, en anglais et en espagnol, de votre première route à la production.",
    docs_browse: "Parcourir la documentation",
    cta_title: ["Prêt à", "déballer ?"],
    cta_start: "Commencer",
    cta_star: "Une étoile sur GitHub",
    footer_tagline: "Le framework full-stack le plus simple pour Rust.",
    footer_cols: ["Produit", "Documentation", "Paquets", "Ressources"],
    footer_product: ["Paquets", "Code", "Fonctionnalités", "Chiffres", "Déployer"],
    footer_resources: ["GitHub", "crates.io", "docs.rs", "Déployer sur Heroku"],
    footer_made: "Fait avec Rust, Yew et WebAssembly",
    footer_lang: "Langue",
    totop: "RETOUR EN HAUT • RETOUR EN HAUT •",
    totop_aria: "Retour en haut",
    totop_cursor: "Haut",
    docs_title: "Documentation",
    docs_lead: "Tout ce qu'il faut pour créer, tester et déployer une app Paquets, pas à pas.",
    search: "Rechercher",
    docs_search: "Rechercher dans la documentation…",
    docs_search_hint: "Appuyez sur / pour chercher",
    docs_no_results: "Aucun résultat pour « {} ».",
    docs_toc: "Sur cette page",
    docs_prev: "Précédent",
    docs_next: "Suivant",
    docs_edit: "Modifier cette page sur GitHub",
    docs_missing: "Cette page n'est pas encore disponible dans cette langue.",
    docs_menu: "Sommaire",
    docs_api: "Référence complète de l'API (docs.rs)",
    docs_quick: "Pour commencer",
    docs_pages: "{} pages",
    docs_fallback: "Cette page n'est pas encore traduite : voici la version anglaise.",
    callouts: ["Note", "Astuce", "Important", "Attention", "Prudence"],
    demo: "Démo en direct",
    skip: "Aller au contenu",
    nf_title: "Ce carton est vide.",
    nf_text: "La page que vous cherchez n'existe pas, ou elle a déménagé. Tout le reste est bien emballé et prêt à partir.",
    nf_home: "Retour à l'accueil",
};

pub const ES: Texts = Texts {
    meta_title: "Paquets — el framework full-stack más sencillo para Rust",
    docs_meta: "Documentación de Paquets",
    nav_links: ["Paquetes", "Código", "Funciones", "Desplegar"],
    nav_docs: "Docs",
    nav_aria: "Navegación principal",
    brand_aria: "Paquets, volver arriba",
    menu: "Abrir el menú",
    lang_aria: "Idioma",
    loader: "Desempaquetando",
    hero_badge: "v0.1 — 4 paquetes, 1 crate",
    hero_title: ["El framework", "full-stack más", "sencillo para Rust."],
    hero_accent: 2,
    hero_lead: "Un crate, un import, una App. Un servidor rápido, seguridad por defecto, la base de datos que quieras, una API REST, páginas SEO e islas WebAssembly — ya conectados entre sí.",
    hero_start: "Empezar",
    hero_github: "GitHub",
    hero_scene: "Cuatro cubos — vitesse, securite, memoire-orm y active — vuelan dentro de una sola caja: Paquets.",
    copy: "Copiar",
    copied: "¡Copiado!",
    copy_aria: "Copiar el comando",
    scroll: "Desliza",
    scroll_aria: "Desplazarse hacia abajo",
    marquee_aria: "Lo que aporta Paquets",
    marquee: [
        "Sencillez",
        "Seguro por defecto",
        "Cualquier base de datos",
        "Páginas SEO",
        "Islas WebAssembly",
        "Despliega desde el móvil",
        "Totalmente probado",
        "Listo para crates.io",
    ],
    pkg_label: "Bajo el capó",
    pkg_title: "4 paquetes, 1 framework.",
    pkg_lead: "Paquets reúne cuatro crates del mismo autor. Cada uno es rápido, está probado y sirve por sí solo — juntos, forman un framework full-stack.",
    packages: [
        PkgText {
            role: "Servidor HTTP",
            text: "Una API al estilo Express: app.get, req.param, middlewares, Router, WebSocket, HTTP/3. Más rápido que actix-web y axum.",
            figure: "peticiones por segundo en GET /",
        },
        PkgText {
            role: "Escudo de seguridad",
            text: "WAF (SQLi, XSS, RCE, Log4Shell, SSRF…), limitación de tráfico, CSRF, cabeceras de seguridad y CSP, bloqueo de bots, cookies reforzadas.",
            figure: "para todo el pipeline de seguridad",
        },
        PkgText {
            role: "ORM con tipos",
            text: "Modelos y consultas asíncronos comprobados al compilar, en memoria, SQLite, PostgreSQL, MySQL/MariaDB, Redis y MongoDB.",
            figure: "bases de datos, una sola API",
        },
        PkgText {
            role: "Front-end",
            text: "Los componentes son funciones Rust con use_state. Renderizado en el servidor con todas las etiquetas SEO, e islas WebAssembly para la interactividad.",
            figure: "gzip para una isla contador",
        },
    ],
    pkg_repo: "Repositorio",
    pkg_alone: "Cada paquete también funciona por separado.",
    code_label: "Enséñame el código",
    code_title: "Menos código. La misma potencia.",
    code_lead: "Todo lo que sigue es Rust de verdad que compila hoy — toda la API llega con un solo import.",
    samples: [
        SampleText {
            label: "Hello world",
            caption: "Un import, una App. La seguridad, los logs, la base de datos y los archivos estáticos ya están listos: no hay nada que configurar.",
            comments: &[
                "seguridad, logs, base de datos, archivos estáticos",
                "http://localhost:3000, o $PORT en Heroku",
            ],
        },
        SampleText {
            label: "API REST",
            caption: "Una línea para una API REST completa: lista paginada, lectura, creación, reemplazo, modificación y borrado. Con ?, cada error se convierte en el código HTTP correcto.",
            comments: &[
                "tabla «todo», creada al arrancar",
                "clave primaria, autoincremental",
                "400 si no es un número",
                "404 si no existe",
                "GET, POST, PUT, PATCH, DELETE",
            ],
        },
        SampleText {
            label: "Páginas",
            caption: "Páginas renderizadas en el servidor, con todas las etiquetas SEO, un tema cuidado por defecto y su propia Content Security Policy — en Rust, sin más.",
            comments: &[
                "HTML renderizado en el servidor: título, Open Graph, CSP, tema",
                "aparece en los buscadores y en las vistas previas",
            ],
        },
        SampleText {
            label: "Islas",
            caption: "Interactividad solo donde hace falta: un componente es una función Rust, renderizada en el servidor y animada en el navegador con WebAssembly.",
            comments: &[
                "compartido por el servidor y el navegador",
                "en el navegador: 36,8 kB gzip para este contador",
                "la isla se renderiza en el servidor y luego se hidrata",
            ],
        },
        SampleText {
            label: "Tests",
            caption: "Prueba toda la pila — rutas, seguridad, base de datos, páginas — en memoria: sin puerto, sin logs, con una base nueva cada vez.",
            comments: &[
                "en memoria: sin puerto, sin logs, una base nueva",
                "201 Created",
            ],
        },
    ],
    features_label: "Funciones",
    features_title: "Todo lo que necesitas. Nada que conectar.",
    features_lead: "Paquets toma las decisiones que tomarías de todos modos, para que empieces a construir desde el primer minuto.",
    features: [
        item(
            "Sencillez",
            "Un crate, un import, una App. Buenos valores por defecto, y todo sigue siendo configurable.",
        ),
        item(
            "Seguro por defecto",
            "WAF, limitación de tráfico, CSRF, cabeceras de seguridad y cookies reforzadas delante de cada petición, en unos 6 µs.",
        ),
        item(
            "Cualquier base de datos",
            "En memoria, SQLite, PostgreSQL, MySQL/MariaDB, Redis o MongoDB: cambia DATABASE_URL y conserva el mismo código.",
        ),
        item(
            "Páginas listas para el SEO",
            "HTML renderizado en el servidor con título, descripción, Open Graph, URL canónica y una CSP estricta.",
        ),
        item(
            "Islas WebAssembly",
            "Añade interactividad componente a componente. El bundle solo se carga en las páginas que lo necesitan.",
        ),
        item(
            "Despliega desde el móvil",
            "Un botón Deploy to Heroku, o despliegues automáticos con cada push a master, configurados desde la app de GitHub.",
        ),
        item(
            "Probado",
            "Un cliente de pruebas en memoria para toda la pila, y una CI que revisa el formato, los lints, los tests y la compilación.",
        ),
        item(
            "Listo para crates.io",
            "Publica con una release de GitHub: el crate en crates.io, la referencia de la API en docs.rs.",
        ),
    ],
    numbers_label: "En cifras",
    numbers_title: "Rápido por diseño.",
    numbers: [
        item(
            "peticiones por segundo",
            "GET / con dos núcleos — por delante de actix-web y axum.",
        ),
        item(
            "peticiones en pipeline por segundo",
            "Pipelining HTTP/1.1, también con dos núcleos.",
        ),
        item(
            "pipeline de seguridad",
            "WAF, limitación de tráfico, CSRF y cabeceras: todo el escudo, por petición.",
        ),
        item(
            "bundle de una isla",
            "WebAssembly comprimido con gzip para una isla contador.",
        ),
        item(
            "bases de datos",
            "De la memoria a MongoDB, con un solo ORM.",
        ),
        item(
            "idiomas",
            "La documentación está en español, inglés y francés.",
        ),
    ],
    journey_label: "De cero a producción",
    journey_title: "Cuatro pasos. Ni uno más.",
    journey_lead: "Crea, desarrolla, haz push: GitHub fusiona y Heroku despliega. Los últimos pasos los puedes dar incluso desde el móvil.",
    journey: [
        item(
            "Crear",
            "Un proyecto completo: servidor, páginas, islas, Dockerfile, configuración de Heroku y workflows de GitHub.",
        ),
        item(
            "Desarrollar",
            "Compila las islas si las hay y luego arranca tu app en localhost:3000.",
        ),
        item(
            "Hacer push",
            "La CI pasa los tests y luego fusiona tu rama en master automáticamente.",
        ),
        item(
            "Publicar",
            "Cada push a master se despliega en Heroku mediante la Platform API. Sin CLI.",
        ),
    ],
    term_out: [
        "Comando paquets instalado",
        "my-app creado: servidor, páginas, islas, Docker, CI",
        "Escuchando en http://localhost:3000",
        "Tests OK · fusionado en master · en línea en Heroku",
    ],
    term_replay: "Repetir",
    deploy_label: "Despliega desde el móvil",
    deploy_title: "Publica desde el sofá.",
    deploy_lead: "Sin ordenador: un botón crea tu app en Heroku, y después cada push a master se publica solo.",
    deploy_steps: [
        item(
            "Pulsa el botón",
            "Deploy to Heroku crea la app a partir del repositorio, construida con Docker, con Heroku Postgres si lo quieres.",
        ),
        item(
            "Conecta GitHub",
            "Añade el secreto HEROKU_API_KEY y la variable HEROKU_APP_NAME, directamente desde la app de GitHub.",
        ),
        item(
            "Push, y está en línea",
            "Cada push a master se despliega automáticamente con un workflow de GitHub.",
        ),
    ],
    deploy_button_alt: "Desplegar en Heroku",
    deploy_note: "Heroku no tiene plan gratuito: un dyno Eco cuesta unos 5 $ al mes.",
    deploy_guide: "Leer la guía de despliegue",
    phone: [
        "Desplegar en Heroku",
        "mi-app-paquets",
        "Construyendo con Docker…",
        "Tu app está en línea",
        "Abrir la app",
        "Nombre de la app",
        "Región",
        "Europa",
    ],
    docs_label: "Documentación",
    docs_home_title: "Se aprende en una tarde.",
    docs_home_lead: "24 páginas en español, inglés y francés, desde tu primera ruta hasta producción.",
    docs_browse: "Explorar la documentación",
    cta_title: ["¿Listo para", "desempaquetar?"],
    cta_start: "Empezar",
    cta_star: "Una estrella en GitHub",
    footer_tagline: "El framework full-stack más sencillo para Rust.",
    footer_cols: ["Producto", "Documentación", "Paquetes", "Recursos"],
    footer_product: ["Paquetes", "Código", "Funciones", "Cifras", "Desplegar"],
    footer_resources: ["GitHub", "crates.io", "docs.rs", "Desplegar en Heroku"],
    footer_made: "Hecho con Rust, Yew y WebAssembly",
    footer_lang: "Idioma",
    totop: "VOLVER ARRIBA • VOLVER ARRIBA •",
    totop_aria: "Volver arriba",
    totop_cursor: "Arriba",
    docs_title: "Documentación",
    docs_lead: "Todo lo que necesitas para crear, probar y desplegar una app Paquets, paso a paso.",
    search: "Buscar",
    docs_search: "Buscar en la documentación…",
    docs_search_hint: "Pulsa / para buscar",
    docs_no_results: "Sin resultados para «{}».",
    docs_toc: "En esta página",
    docs_prev: "Anterior",
    docs_next: "Siguiente",
    docs_edit: "Editar esta página en GitHub",
    docs_missing: "Esta página aún no está disponible en este idioma.",
    docs_menu: "Índice",
    docs_api: "Referencia completa de la API (docs.rs)",
    docs_quick: "Para empezar",
    docs_pages: "{} páginas",
    docs_fallback: "Esta página aún no está traducida: aquí tienes la versión en inglés.",
    callouts: ["Nota", "Consejo", "Importante", "Advertencia", "Precaución"],
    demo: "Demo en vivo",
    skip: "Saltar al contenido",
    nf_title: "Esta caja está vacía.",
    nf_text: "La página que buscas no existe o se ha mudado. Todo lo demás sigue empaquetado y listo.",
    nf_home: "Volver al inicio",
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{SAMPLES, markers};

    #[test]
    fn formats_numbers_per_language() {
        assert_eq!(number(Lang::Fr, 2_781_541.0, 0), "2\u{202f}781\u{202f}541");
        assert_eq!(number(Lang::En, 2_781_541.0, 0), "2,781,541");
        assert_eq!(number(Lang::Es, 2_781_541.0, 0), "2.781.541");
        assert_eq!(number(Lang::Es, 2_026.0, 0), "2026");
        assert_eq!(number(Lang::Fr, 2.78, 2), "2,78");
        assert_eq!(number(Lang::En, 36.8, 1), "36.8");
        assert_eq!(number(Lang::Es, 36.84, 1), "36,8");
    }

    #[test]
    fn detects_language_codes() {
        assert_eq!(Lang::from_tag("fr-CA"), Some(Lang::Fr));
        assert_eq!(Lang::from_tag("ES"), Some(Lang::Es));
        assert_eq!(Lang::parse("en"), Some(Lang::En));
        assert_eq!(Lang::parse("EN"), None);
        assert_eq!(Lang::from_tag("de-DE"), None);
        assert_eq!(Lang::from_tag("e"), None);
        assert_eq!(Lang::ALL.map(Lang::code), ["en", "fr", "es"]);
    }

    #[test]
    fn picks_the_preferred_language() {
        assert_eq!(pick_lang("fr-FR,fr;q=0.9,en;q=0.8"), Lang::Fr);
        assert_eq!(pick_lang("de-DE, es;q=0.5, en;q=0.4"), Lang::Es);
        assert_eq!(pick_lang("de, it"), Lang::En);
        assert_eq!(pick_lang(""), Lang::En);
        assert_eq!(pick_lang("fr;q=0, en"), Lang::En);
    }

    #[test]
    fn every_text_is_filled() {
        for lang in Lang::ALL {
            for (i, s) in lang.texts().strings().iter().enumerate() {
                assert!(!s.trim().is_empty(), "{}: text #{i} is empty", lang.code());
                assert_eq!(s.trim(), *s, "{}: spaces around «{s}»", lang.code());
            }
        }
        let n = EN.strings().len();
        assert!(FR.strings().len() == n && ES.strings().len() == n);
    }

    #[test]
    fn languages_are_really_translated() {
        let en = EN.strings();
        for lang in [Lang::Fr, Lang::Es] {
            let other = lang.texts().strings();
            let same: Vec<&str> = en
                .iter()
                .zip(&other)
                .filter(|(a, b)| a == b)
                .map(|(a, _)| *a)
                .collect();
            // Proper names (GitHub, Docs, Hello world…) stay the same.
            assert!(
                same.len() * 100 / en.len() < 12,
                "{}: {} texts identical to English: {same:?}",
                lang.code(),
                same.len()
            );
        }
    }

    #[test]
    fn placeholders_match() {
        for lang in Lang::ALL {
            let t = lang.texts();
            assert!(t.docs_no_results.contains("{}"));
            assert!(t.docs_pages.contains("{}"));
            assert!(t.hero_accent < t.hero_title.len());
            let en = EN.strings();
            for (a, b) in en.iter().zip(t.strings()) {
                assert_eq!(a.matches("{}").count(), b.matches("{}").count(), "«{b}»");
            }
        }
    }

    #[test]
    fn sample_comments_match_the_code() {
        for lang in Lang::ALL {
            for (sample, text) in SAMPLES.iter().zip(&lang.texts().samples) {
                assert_eq!(
                    markers(sample.code),
                    text.comments.len(),
                    "{}: {}",
                    lang.code(),
                    text.label
                );
            }
        }
    }
}
