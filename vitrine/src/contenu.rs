//! Les textes du site. Tout est ici, pour être remplacé par les vôtres : coordonnées,
//! services, projets, équipe, témoignages, pages légales.
//!
//! ⚠️ Orbite est un studio fictif : noms, chiffres, clients et témoignages sont des
//! exemples à remplacer avant la mise en ligne.

pub const STUDIO: &str = "Orbite Studio";
pub const EMAIL: &str = "bonjour@orbite.studio";
pub const TELEPHONE: &str = "+33 4 78 00 00 00";
pub const RUE: &str = "12 rue de la Lumière";
pub const CODE_POSTAL: &str = "69002";
pub const VILLE: &str = "Lyon";
pub const ANNEE: &str = "2026";
pub const SIRET: &str = "000 000 000 00000";

/// Le menu principal.
pub const MENU: [(&str, &str); 4] = [
    ("/services", "Services"),
    ("/realisations", "Réalisations"),
    ("/studio", "Studio"),
    ("/contact", "Contact"),
];

/// Les pages du plan du site (et du sitemap.xml).
pub const PAGES: [(&str, &str); 8] = [
    ("/", "Accueil"),
    ("/services", "Services et tarifs"),
    ("/realisations", "Réalisations"),
    ("/studio", "Le studio"),
    ("/contact", "Contact"),
    ("/mentions-legales", "Mentions légales"),
    ("/confidentialite", "Politique de confidentialité"),
    ("/plan-du-site", "Plan du site"),
];

/// Nom, adresse et icône de chaque réseau.
pub const RESEAUX: [(&str, &str, &str); 4] = [
    ("LinkedIn", "https://www.linkedin.com/", "linkedin"),
    ("GitHub", "https://github.com/maxlestage/Paquets", "github"),
    ("Dribbble", "https://dribbble.com/", "dribbble"),
    ("Instagram", "https://www.instagram.com/", "instagram"),
];

pub const PIED_STUDIO: &[(&str, &str)] = &[
    ("/studio", "À propos"),
    ("/studio#equipe", "Équipe"),
    ("/realisations", "Réalisations"),
    ("/contact", "Nous rejoindre"),
];

pub const PIED_SERVICES: &[(&str, &str)] = &[
    ("/services", "Sites vitrines"),
    ("/services", "Expériences 3D"),
    ("/services", "Applications web"),
    ("/services#tarifs", "Tarifs"),
];

pub const PIED_RESSOURCES: &[(&str, &str)] = &[
    ("/services#faq", "FAQ"),
    ("/realisations#physique", "Laboratoire 3D"),
    ("/plan-du-site", "Plan du site"),
    ("/confidentialite", "Données personnelles"),
];

pub struct Service {
    pub icone: &'static str,
    pub titre: &'static str,
    pub texte: &'static str,
    pub tags: &'static [&'static str],
}

pub const SERVICES: [Service; 6] = [
    Service {
        icone: "cube",
        titre: "Expériences 3D",
        texte: "Scènes WebGL temps réel, configurateurs produits et visites interactives, fluides même sur mobile.",
        tags: &["WebGL 2", "Physique", "Rust"],
    },
    Service {
        icone: "ecran",
        titre: "Sites vitrines",
        texte: "Des sites de marque rapides et mémorables, rendus sur le serveur pour un référencement optimal.",
        tags: &["SEO", "Animations", "CMS"],
    },
    Service {
        icone: "eclair",
        titre: "Applications web",
        texte: "Back-offices, espaces clients, API REST : des produits robustes, typés de bout en bout.",
        tags: &["API", "Base de données", "Auth"],
    },
    Service {
        icone: "mobile",
        titre: "PWA installables",
        texte: "Votre site sur l'écran d'accueil, utilisable hors ligne, sans passer par les stores.",
        tags: &["Hors ligne", "Manifest", "Service worker"],
    },
    Service {
        icone: "bouclier",
        titre: "Sécurité & performance",
        texte: "Pare-feu applicatif, CSP stricte, 100/100 Lighthouse : la qualité se mesure.",
        tags: &["WAF", "CSP", "Core Web Vitals"],
    },
    Service {
        icone: "pinceau",
        titre: "Direction artistique",
        texte: "Identité visuelle, motion design et design system pour une marque cohérente partout.",
        tags: &["UI", "Motion", "Design system"],
    },
];

pub struct Projet {
    pub titre: &'static str,
    pub client: &'static str,
    pub annee: &'static str,
    pub categorie: &'static str,
    pub texte: &'static str,
    pub tags: &'static [&'static str],
    pub resultat: (&'static str, &'static str),
    pub couleurs: (&'static str, &'static str),
}

pub const PROJETS: [Projet; 6] = [
    Projet {
        titre: "Configurateur de vélos",
        client: "Kinetik",
        annee: "2026",
        categorie: "3D",
        texte: "Un configurateur temps réel : cadre, couleurs, roues, avec rendu physique et prix mis à jour à chaque clic.",
        tags: &["WebGL", "E-commerce"],
        resultat: ("+38 %", "de conversions"),
        couleurs: ("#7c5cff", "#2ee6a6"),
    },
    Projet {
        titre: "Maison Lumen",
        client: "Lumen & Co",
        annee: "2026",
        categorie: "Vitrine",
        texte: "Le site d'une maison de luminaires : animations au défilement et catalogue rendu sur le serveur.",
        tags: &["SEO", "Motion"],
        resultat: ("×2,4", "de trafic organique"),
        couleurs: ("#ffc857", "#ff6b6b"),
    },
    Projet {
        titre: "Nordlys Explorer",
        client: "Nordlys",
        annee: "2025",
        categorie: "3D",
        texte: "Une visite interactive d'aurores boréales pour une agence de voyages, installable en PWA.",
        tags: &["PWA", "Hors ligne"],
        resultat: ("4 min 12", "de session moyenne"),
        couleurs: ("#5cc8ff", "#2ee6a6"),
    },
    Projet {
        titre: "Espace client Arcadia",
        client: "Arcadia",
        annee: "2025",
        categorie: "Application",
        texte: "Un espace client sécurisé avec tableau de bord, documents et messagerie, pour 40 000 utilisateurs.",
        tags: &["Auth", "API REST"],
        resultat: ("−62 %", "d'appels au support"),
        couleurs: ("#ff6b6b", "#7c5cff"),
    },
    Projet {
        titre: "Solstice Festival",
        client: "Solstice",
        annee: "2024",
        categorie: "Vitrine",
        texte: "Billetterie et programme d'un festival, avec une scène 3D qui suit le coucher du soleil en direct.",
        tags: &["3D", "Billetterie"],
        resultat: ("12 000", "billets en 48 h"),
        couleurs: ("#ff6b6b", "#ffc857"),
    },
    Projet {
        titre: "Polaris Labs",
        client: "Polaris Labs",
        annee: "2024",
        categorie: "Application",
        texte: "Une plateforme de suivi d'expériences scientifiques, avec visualisation 3D des molécules.",
        tags: &["Data", "WebGL"],
        resultat: ("99,98 %", "de disponibilité"),
        couleurs: ("#2ee6a6", "#5cc8ff"),
    },
];

/// Nombre, suffixe, libellé.
pub const CHIFFRES: [(u32, &str, &str); 4] = [
    (120, "+", "projets livrés"),
    (98, " %", "de clients satisfaits"),
    (100, "", "score Lighthouse moyen"),
    (7, " ans", "d'expérience"),
];

pub const CLIENTS: [&str; 8] = [
    "Lumen & Co",
    "Arcadia",
    "Nordlys",
    "Kinetik",
    "Solstice",
    "Vertexa",
    "Halcyon",
    "Polaris Labs",
];

/// Titre, durée, texte.
pub const ETAPES: [(&str, &str, &str); 4] = [
    (
        "Découverte",
        "Semaine 1",
        "Ateliers, analyse de vos utilisateurs et de vos concurrents, objectifs mesurables.",
    ),
    (
        "Conception",
        "Semaines 2 – 3",
        "Arborescence, maquettes interactives, direction artistique et prototypes 3D.",
    ),
    (
        "Production",
        "Semaines 4 – 7",
        "Développement, intégration 3D, contenus, tests sur tous les appareils.",
    ),
    (
        "Lancement",
        "Semaine 8",
        "Mise en ligne, suivi des performances et accompagnement sur la durée.",
    ),
];

pub struct Avis {
    pub texte: &'static str,
    pub nom: &'static str,
    pub role: &'static str,
    pub initiales: &'static str,
    pub couleur: &'static str,
}

pub const AVIS: [Avis; 6] = [
    Avis {
        texte: "Le configurateur 3D a transformé nos ventes en ligne. Les clients passent du temps à jouer avec, puis ils achètent.",
        nom: "Claire D.",
        role: "Directrice e-commerce, Kinetik",
        initiales: "CD",
        couleur: "#7c5cff",
    },
    Avis {
        texte: "Une équipe d'une rigueur rare : planning tenu, code impeccable et un site plus rapide que tout ce que nous avions connu.",
        nom: "Mehdi A.",
        role: "CTO, Arcadia",
        initiales: "MA",
        couleur: "#ff6b6b",
    },
    Avis {
        texte: "Ils ont compris notre marque en une réunion. Le résultat est élégant, vivant, et nos visiteurs restent.",
        nom: "Sofia L.",
        role: "Fondatrice, Lumen & Co",
        initiales: "SL",
        couleur: "#ffc857",
    },
    Avis {
        texte: "La PWA fonctionne hors ligne en pleine montagne. Nos guides l'utilisent tous les jours.",
        nom: "Erik N.",
        role: "Responsable produit, Nordlys",
        initiales: "EN",
        couleur: "#5cc8ff",
    },
    Avis {
        texte: "Sécurité, accessibilité, performance : tout était documenté et mesuré. Un vrai partenaire technique.",
        nom: "Julie R.",
        role: "DSI, Halcyon",
        initiales: "JR",
        couleur: "#2ee6a6",
    },
    Avis {
        texte: "Notre billetterie a tenu le pic de trafic sans broncher. Et la scène 3D du coucher de soleil a fait le tour des réseaux.",
        nom: "Thomas B.",
        role: "Directeur, Solstice Festival",
        initiales: "TB",
        couleur: "#ff6b6b",
    },
];

pub const FAQ: [(&str, &str); 6] = [
    (
        "Combien de temps prend un projet ?",
        "Comptez 4 à 8 semaines pour un site vitrine avec de la 3D, et 2 à 4 mois pour une application. Nous fixons un planning précis dès la proposition.",
    ),
    (
        "La 3D ralentit-elle le site ?",
        "Non. Les pages sont rendues sur le serveur et s'affichent immédiatement ; les scènes 3D, compilées en WebAssembly, se chargent ensuite et ne s'animent que lorsqu'elles sont visibles.",
    ),
    (
        "Mon site fonctionnera-t-il hors ligne ?",
        "Oui : chaque site que nous livrons est une PWA. Les pages visitées restent disponibles sans réseau, et le site peut être installé sur l'écran d'accueil.",
    ),
    (
        "Pourrai-je modifier mes contenus ?",
        "Oui. Selon vos besoins, nous intégrons un back-office simple ou un CMS, avec une formation d'une demi-journée.",
    ),
    (
        "Qui est propriétaire du code ?",
        "Vous. À la fin du projet, le code source, les maquettes et les modèles 3D vous sont cédés.",
    ),
    (
        "Proposez-vous la maintenance ?",
        "Tous nos forfaits incluent l'hébergement, les mises à jour de sécurité et un suivi mensuel des performances.",
    ),
];

/// Icône, titre, texte.
pub const VALEURS: [(&str, &str, &str); 4] = [
    (
        "eclair",
        "Vitesse",
        "Chaque milliseconde compte. Nous mesurons tout, sur de vrais appareils.",
    ),
    (
        "bouclier",
        "Confiance",
        "Sécurité par défaut, données minimisées, code propre et documenté.",
    ),
    (
        "coeur",
        "Accessibilité",
        "Navigation au clavier, contrastes, mouvement réduit : le web pour tous.",
    ),
    (
        "etoile",
        "Émerveillement",
        "La touche de magie qui fait qu'on se souvient de votre site.",
    ),
];

pub struct Membre {
    pub nom: &'static str,
    pub role: &'static str,
    pub initiales: &'static str,
    pub bio: &'static str,
    pub detail: &'static str,
    pub couleurs: (&'static str, &'static str),
}

pub const EQUIPE: [Membre; 4] = [
    Membre {
        nom: "Alix Moreau",
        role: "Cofondatrice · Direction artistique",
        initiales: "AM",
        bio: "Quinze ans de design de marque, obsédée par le mouvement et la typographie.",
        detail: "Collectionne les affiches de cinéma des années 70.",
        couleurs: ("#7c5cff", "#ff6b6b"),
    },
    Membre {
        nom: "Samuel Petit",
        role: "Cofondateur · Ingénierie",
        initiales: "SP",
        bio: "Développeur Rust et WebGL, il fait tourner la 3D à 60 images par seconde sur un vieux téléphone.",
        detail: "Grimpe le week-end, optimise la semaine.",
        couleurs: ("#2ee6a6", "#5cc8ff"),
    },
    Membre {
        nom: "Inès Garnier",
        role: "Artiste 3D",
        initiales: "IG",
        bio: "Modélisation, lumière et matières : elle donne du relief à chaque idée.",
        detail: "A modélisé son appartement entier, au millimètre.",
        couleurs: ("#ffc857", "#ff6b6b"),
    },
    Membre {
        nom: "Hugo Lambert",
        role: "Chef de projet",
        initiales: "HL",
        bio: "Le chef d'orchestre : planning, budget et sérénité de nos clients.",
        detail: "N'a jamais raté une échéance. Jamais.",
        couleurs: ("#5cc8ff", "#7c5cff"),
    },
];

/// Année, titre, texte.
pub const HISTOIRE: [(&str, &str, &str); 4] = [
    (
        "2019",
        "Le décollage",
        "Alix et Samuel fondent Orbite dans un atelier des pentes de la Croix-Rousse.",
    ),
    (
        "2021",
        "Première scène 3D",
        "Le configurateur Kinetik prouve que la 3D peut vendre.",
    ),
    (
        "2024",
        "Cap sur Rust",
        "Nous adoptons Rust et WebAssembly pour des sites plus rapides et plus sûrs.",
    ),
    (
        "2026",
        "Sept en orbite",
        "Une équipe de sept, et plus de 120 projets livrés en France et en Europe.",
    ),
];

pub const BUDGETS: [&str; 4] = ["< 5 k€", "5 – 15 k€", "15 – 40 k€", "> 40 k€"];

pub const MENTIONS: [(&str, &str); 4] = [
    (
        "Éditeur du site",
        "Orbite Studio SAS, au capital de 10 000 €, immatriculée au RCS de Lyon sous le numéro 000 000 000. Siège social : 12 rue de la Lumière, 69002 Lyon. Directeur de la publication : le représentant légal de la société.\n\nContact : bonjour@orbite.studio · +33 4 78 00 00 00.",
    ),
    (
        "Hébergement",
        "Le site est hébergé par votre prestataire (par exemple Heroku, Salesforce Inc., 415 Mission Street, San Francisco, CA 94105, États-Unis). Remplacez ces informations par celles de votre hébergeur.",
    ),
    (
        "Propriété intellectuelle",
        "L'ensemble des contenus de ce site (textes, visuels, scènes 3D, code) est protégé par le droit d'auteur. Toute reproduction sans autorisation écrite préalable est interdite.",
    ),
    (
        "Crédits",
        "Conception et réalisation : Orbite Studio. Site construit avec Paquets, framework Rust full-stack, et rendu 3D par paquets::three (WebGL 2).",
    ),
];

pub const CONFIDENTIALITE: [(&str, &str); 5] = [
    (
        "Données collectées",
        "Nous collectons uniquement les informations que vous nous transmettez via le formulaire de contact (nom, e-mail, entreprise, budget, message) et l'adresse e-mail saisie pour la newsletter.",
    ),
    (
        "Finalités",
        "Ces données servent exclusivement à répondre à votre demande et, si vous vous y êtes inscrit, à vous envoyer notre lettre mensuelle. Elles ne sont jamais vendues ni cédées.",
    ),
    (
        "Durée de conservation",
        "Les messages sont conservés 3 ans après notre dernier échange. Les adresses de la newsletter sont supprimées dès votre désinscription.",
    ),
    (
        "Cookies et mesure d'audience",
        "Ce site n'utilise aucun cookie publicitaire ni traceur tiers. Le mode hors ligne repose sur le cache de votre navigateur, que vous pouvez vider à tout moment.",
    ),
    (
        "Vos droits",
        "Conformément au RGPD, vous disposez d'un droit d'accès, de rectification, d'effacement et de portabilité de vos données. Écrivez-nous à bonjour@orbite.studio. Vous pouvez également saisir la CNIL.",
    ),
];
