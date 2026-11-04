# 🪐 Orbite — un site vitrine complet écrit avec Paquets

Un site de studio créatif, du serveur à la 3D, écrit en Rust avec [Paquets](../README.md) :
pages rendues sur le serveur avec leurs balises SEO, PWA installable et hors ligne, scènes
3D temps réel en WebAssembly (`paquets::three`), physique (`paquets::three::Physics`),
formulaires enregistrés en base, et beaucoup d'animations.

> ⚠️ **Orbite est un studio fictif.** Coordonnées, clients, chiffres et témoignages sont des
> exemples : remplacez-les dans `src/contenu.rs` avant toute mise en ligne.

## Ce qu'il y a dedans

| | |
|---|---|
| **Pages** | Accueil, Services & tarifs, Réalisations, Studio, Contact, Merci, Mentions légales, Confidentialité, Plan du site, 404 animée |
| **3D (Rust → WebAssembly)** | Héros : gemme, coque filaire, anneaux et 46 paquets en orbite, qui suivent le pointeur et le défilement · Atelier : objet configurable (forme, teinte, métal, filaire, rotation auto) avec `OrbitControls` · Laboratoire : pluie de paquets avec masse, gravité, rebonds et collisions |
| **Animations** | Titres révélés mot à mot, apparitions au défilement, cartes inclinées en 3D avec lueur sous le pointeur, boutons magnétiques, curseur personnalisé, compteurs, frise qui se remplit, défilants de logos et de témoignages, cubes et planète en CSS 3D, cartes d'équipe qui se retournent, transitions entre pages (View Transitions), menu mobile en cercle, glitch 404 |
| **Pied de page** | Newsletter, appel à l'action, 4 colonnes de liens, coordonnées, réseaux, statut de disponibilité, bouton d'installation de l'app, grand nom révélé au défilement, ligne légale, retour en haut |
| **PWA** | Manifeste, service worker hors ligne, icônes dédiées (`public/icons/`), bouton « Installer l'application » |
| **Données** | `#[model] Message` (contact) et `#[model] Abonne` (newsletter), champ piège anti-robots, validation côté serveur |
| **SEO** | Titre, description, Open Graph (`public/og.png`), canonical, JSON-LD `Organization`, `sitemap.xml`, `robots.txt` |
| **Sécurité** | Le bouclier de Paquets (WAF, limite de débit, CSRF, en-têtes) et une CSP stricte : aucun script inline hors empreintes |
| **Accessibilité** | Lien d'évitement, navigation clavier, `aria-*` sur les contrôles, `prefers-reduced-motion` respecté (CSS, JS et 3D) |

## Organisation

| Fichier | Rôle |
|---|---|
| `src/main.rs` | Le serveur : mise en page (en-tête, pied de page), pages, formulaires, PWA, sitemap |
| `src/contenu.rs` | **Tous les textes** : coordonnées, services, projets, équipe, FAQ, pages légales |
| `src/icones.rs` | Les icônes SVG |
| `src/lib.rs` | Les îlots (serveur + navigateur) : `hero3d`, `atelier`, `pluie`, `tarifs` |
| `src/scenes.rs` | Les scènes 3D (navigateur seulement) |
| `public/style.css` | Le thème et les animations |
| `public/motion.js` | Les mouvements de la page (défilement, inclinaison, curseur, menu…) |

## Lancer

Depuis ce dossier (`vitrine/`) :

```bash
# Une fois : la cible WebAssembly et wasm-bindgen (même version que Cargo.toml)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129

# 1. Le bundle du navigateur (îlots + 3D) → pkg/
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/orbite.wasm

# 2. Le serveur : http://localhost:3000
cargo run --release
```

Sans `pkg/`, le site fonctionne quand même (pages, formulaires, animations CSS et JS), sans
la 3D ni les îlots interactifs.

Variables d'environnement : `PORT`, `DATABASE_URL` (en mémoire par défaut ;
`sqlite://orbite.db` ou PostgreSQL pour garder les messages), `SITE_URL` (adresse publique,
pour les URL canoniques et le sitemap).

## Vérifier

```bash
cargo test                                              # chaque page, les îlots, la PWA
cargo clippy --all-targets
cargo clippy --lib --target wasm32-unknown-unknown
```
