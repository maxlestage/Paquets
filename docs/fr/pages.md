# Pages

Une `Page` est un document HTML complet, rendu par le serveur et renvoyé par un handler. Elle arrive avec toutes les balises SEO (titre, description, Open Graph, cartes Twitter…), un thème intégré qui rend bien sans écrire de CSS, et une Content-Security-Policy qui autorise exactement ce que la page contient. Cette page montre comment construire des vues, utiliser le thème, définir les balises SEO et composer avec la politique de sécurité.

## Une première page

```rust
async fn home(_req: Request) -> Page {
    Page::new("Accueil")
        .description("Mon site, en Rust.")
        .body(h1().text("Bonjour 👋"))
        .body(p().text("Construit avec Paquets."))
}

app.get("/", home);
```

Le navigateur reçoit du HTML terminé :

```html
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Accueil</title><meta name="description" content="Mon site, en Rust."/>
<meta property="og:title" content="Accueil"/><meta property="og:description" content="Mon site, en Rust."/>
<meta property="og:type" content="website"/><meta name="twitter:card" content="summary"/>
<style>/* le thème */</style>...</head>
<body><h1>Bonjour 👋</h1><p>Construit avec Paquets.</p></body></html>
```

La réponse porte aussi un en-tête `Content-Security-Policy` calculé pour cette page, ainsi que les en-têtes de sécurité du bouclier.

## Les méthodes de Page

| Méthode | Effet |
|---|---|
| `Page::new(titre)` | une page avec son `<title>` (aussi utilisé pour Open Graph et Twitter) |
| `.description(texte)` | `<meta name="description">`, aussi utilisé pour Open Graph et Twitter |
| `.lang("fr")` | `<html lang="fr">` (`en` par défaut) |
| `.body(vue)` | ajoute du contenu au `<body>` ; appelez-la autant de fois que vous voulez |
| `.seo(Seo::new()...)` | d'autres balises SEO : URL canonique, image, `noindex`, langues, JSON-LD… |
| `.style(css)` | du CSS écrit dans la page, autorisé par sa politique de sécurité |
| `.stylesheet(href)` | un `<link rel="stylesheet">`, par exemple vers un fichier de `public/` |
| `.head(html)` | du HTML de confiance ajouté au `<head>` (une icône, un préchargement…) |
| `.plain()` | retire le thème intégré |
| `.status(404)` | le statut HTTP (`200` par défaut) |
| `.render()` | le HTML sous forme de `String`, sans l'envoyer |

## Construire des vues

Les vues se construisent avec **une fonction par balise HTML**, toutes dans le prélude : `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`, `table()`, `img()`… Chacune renvoie un élément que vous configurez avec des méthodes enchaînées :

```rust
let card = div()
    .class("card")
    .child(h2().text("Rapide"))
    .child(p().text("Rendu par le serveur."))
    .child(a().href("/docs").text("Lire la documentation"));
```

| Méthode | Effet |
|---|---|
| `.text(s)` | ajoute du texte, toujours échappé : `"<b>"` est affiché, jamais interprété |
| `.child(vue)` | ajoute un enfant : un élément, un texte, une `Option` (rien pour `None`)… |
| `.children(iter)` | ajoute plusieurs enfants, en général issus d'un `.map(...)` |
| `.class("a b")` | ajoute des classes ; rappelez-la pour en ajouter d'autres |
| `.id("x")`, `.href(url)`, `.style(css)` | les attributs courants |
| `.attr(nom, valeur)` | n'importe quel attribut : `.attr("type", "email")` |
| `.html(brut)` | du HTML brut, non échappé : uniquement pour du HTML de confiance |

Les listes viennent des itérateurs, et les conditions des `Option` :

```rust
async fn todos_page(req: Request) -> Result<Page> {
    let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
    let empty = todos.is_empty();
    Ok(Page::new("Tâches").body(
        section()
            .child(h1().text("Mes tâches"))
            .child(empty.then(|| p().class("muted").text("Rien à faire !")))
            .child(ul().children(todos.into_iter().map(|t| li().text(t.title)))),
    ))
}
```

Les arguments texte acceptent `&'static str` et `String`. Un `&str` emprunté doit d'abord devenir une `String` : `name.to_string()` ou `format!(...)`.

### Les composants sont des fonctions

Un composant est une simple fonction Rust qui renvoie une vue. Ses arguments sont ses props :

```rust
/// Une carte avec un titre, un texte et un lien facultatif.
fn card(title: &str, body: &str, link: Option<&str>) -> Element {
    article()
        .class("card")
        .child(h2().text(title.to_string()))
        .child(p().text(body.to_string()))
        .child(link.map(|href| a().href(href.to_string()).text("Lire la suite")))
}

fn features() -> Element {
    div()
        .class("grid")
        .child(card("Rapide", "Rendu par le serveur.", None))
        .child(card("Sécurisé", "Un bouclier devant chaque requête.", Some("/security")))
}
```

Renvoyez un `Element` quand le composant est un seul élément (l'appelant peut continuer à enchaîner `.class(...)`), et un `Node` quand il peut s'agir de plusieurs éléments frères ou de rien du tout. Un `Element` se transforme en `Node` avec `.into()`.

### Une mise en page commune

Écrivez une seule fois les parties communes à toutes les pages, dans une fonction qui renvoie une `Page` :

```rust
fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .head(r#"<link rel="icon" href="/favicon.svg"/>"#)
        .body(
            header()
                .child(a().href("/").child(strong().text("📦 Mon site")))
                .child(nav().child(a().href("/blog").text("Blog")).child(a().href("/about").text("À propos"))),
        )
        .body(main().child(content))
        .body(footer().text("© Mon site"))
}

async fn about(_req: Request) -> Page {
    layout("À propos", p().text("Nous fabriquons des choses en Rust."))
        .description("Qui nous sommes.")
}
```

### Construire la page à la fin du handler

Les vues vivent sur le thread qui les rend : elles ne peuvent donc pas être conservées de part et d'autre d'un `.await`. Faites d'abord tous les `.await` (base de données, fichiers…), puis construisez la page. Si vous créez une vue avant un `.await`, le compilateur vous arrête avec `future cannot be sent between threads safely`. Les exemples de cette page suivent tous cet ordre.

## La balise main dans main.rs

L'élément `<main>` se crée avec `main()`. Dans un fichier qui définit la fonction `fn main` du programme (en général `src/main.rs`), ce nom est pris par votre fonction, et `main().child(...)` échoue avec une erreur du genre `no method named child found for enum Result`. Dans ce fichier, écrivez plutôt `html::main()` :

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Accueil").body(html::main().child(h1().text("Bonjour 👋")))
    });
    app.run()
}
```

Dans tous les autres fichiers (modules, `src/lib.rs`, tests), `main()` fonctionne comme prévu.

## Le thème intégré

Chaque page inclut un petit thème (environ 3,5 ko) : une typographie lisible, des boutons, des formulaires, des tableaux, et un mode clair ou sombre qui suit les réglages du système du visiteur. Du HTML tout simple rend déjà bien, et quelques classes couvrent les mises en page courantes :

| Classe ou élément | Rendu |
|---|---|
| `header`, `nav`, `footer` | une rangée d'éléments ; l'en-tête a une bordure en bas, le pied de page une bordure en haut |
| `.hero` | une grande introduction centrée, pour le haut d'une page d'accueil |
| `.card` | une boîte avec une bordure et des coins arrondis |
| `.grid` | une grille adaptative : autant de colonnes que la place le permet (au moins 15rem chacune) |
| `.row` | des éléments côte à côte, qui passent à la ligne sur les petits écrans |
| `.stack` | des enfants espacés verticalement |
| `.center` | du texte centré |
| `.muted` | du texte secondaire, en gris |
| `button`, `.button` | un bouton arrondi dans la couleur d'accent ; `.button` donne à un lien l'apparence d'un bouton |
| `.ghost` | un bouton transparent avec une bordure, pour les actions secondaires |
| `input`, `select`, `textarea`, `label` | des champs pleine largeur avec un anneau de focus |
| `table` | pleine largeur, avec des lignes entre les rangées |

Une page d'accueil qui s'en sert :

```rust
async fn home(_req: Request) -> Page {
    Page::new("Acme")
        .description("Des meubles fabriqués en Rust.")
        .body(
            section()
                .class("hero")
                .child(h1().text("Des meubles, tout simplement"))
                .child(p().text("Du chêne, de l'acier et un peu de Rust."))
                .child(
                    div()
                        .class("row")
                        .style("justify-content:center")
                        .child(a().class("button").href("/shop").text("Voir la boutique"))
                        .child(a().class("button ghost").href("/about").text("Qui sommes-nous ?")),
                ),
        )
        .body(
            div()
                .class("grid")
                .child(div().class("card").child(h3().text("Chaises")).child(p().class("muted").text("À partir de 49 €")))
                .child(div().class("card").child(h3().text("Tables")).child(p().class("muted").text("À partir de 199 €")))
                .child(div().class("card").child(h3().text("Lampes")).child(p().class("muted").text("À partir de 29 €"))),
        )
}
```

### Changer les couleurs

Le thème repose sur des variables CSS. Redéfinissez-les avec `.style(...)` :

```rust
Page::new("Accueil")
    .style(":root { --accent: #e11d48; --radius: 6px; }")
    .body(h1().text("Des boutons rouges, des coins plus nets"))
```

| Variable | Rôle |
|---|---|
| `--bg`, `--fg` | les couleurs du fond et du texte |
| `--muted` | le texte secondaire |
| `--line` | les bordures |
| `--card` | le fond des cartes, des champs et des blocs de code |
| `--accent`, `--accent-fg` | les boutons et les liens, et le texte sur les boutons |
| `--radius` | l'arrondi des coins des cartes |
| `--font`, `--mono` | les polices du texte et du code |

Le mode sombre redéfinit `--bg`, `--fg`, `--muted`, `--line`, `--card` et `--accent` dans `@media (prefers-color-scheme: dark)`.

### Votre propre CSS

- Pour quelques règles, utilisez `.style(css)` : le CSS est écrit dans la page et autorisé par sa politique.
- Pour une feuille de style commune à tout le site, placez-la dans `public/` et liez-la : `.stylesheet("/site.css")`. Les navigateurs la gardent en cache d'une page à l'autre.
- Pour partir d'une page vierge, appelez `.plain()` : le thème n'est pas inclus, seulement vos styles.

Le CSS du thème est aussi disponible dans la constante `paquets::THEME`, si vous voulez le réutiliser ailleurs.

## Balises SEO

`Page::new(titre)` et `.description(...)` produisent déjà le titre, la description, et les balises Open Graph et Twitter que lisent les moteurs de recherche et les réseaux sociaux. Ajoutez-en d'autres avec `.seo(...)` :

```rust
async fn article(_req: Request) -> Page {
    Page::new("Notre nouvelle chaise en chêne")
        .description("En chêne massif, conçue pour durer cinquante ans.")
        .seo(
            Seo::new()
                .base_url("https://acme.example")
                .canonical("/blog/oak-chair")
                .image("/images/oak-chair.jpg")
                .og_type("article")
                .site_name("Acme")
                .title_template("%s · Acme")
                .alternate("fr", "/fr/blog/chaise-chene")
                .json_ld(r#"{"@context":"https://schema.org","@type":"Article","headline":"Notre nouvelle chaise en chêne"}"#),
        )
        .body(h1().text("Notre nouvelle chaise en chêne"))
}
```

| Méthode de `Seo` | Balise |
|---|---|
| `.canonical(url)` | `<link rel="canonical">` : l'adresse officielle de la page |
| `.image(url)` | `og:image` et une grande carte Twitter |
| `.base_url(origine)` | transforme les URL relatives (canonique, image, versions alternatives) en URL absolues |
| `.site_name(nom)` | `og:site_name` |
| `.title_template("%s · Acme")` | le `<title>` final, où `%s` est remplacé par le titre de la page |
| `.noindex(true)`, `.nofollow(true)` | `<meta name="robots">`, pour tenir une page à l'écart des moteurs de recherche |
| `.alternate(langue, url)` | `<link rel="alternate" hreflang>`, une fois par langue |
| `.json_ld(json)` | des données structurées pour des résultats de recherche enrichis |
| `.og_type(...)`, `.locale(...)`, `.twitter(compte)` | les autres balises Open Graph et Twitter |
| `.meta_name(nom, contenu)`, `.meta_property(...)` | n'importe quelle autre balise `<meta>` |

Un `Seo` peut aussi se placer à l'intérieur d'une vue (`.child(Seo::new().title(...))`) : tous les `Seo` de la page sont fusionnés, la valeur la plus profonde l'emportant, sans balises en double. Une mise en page peut donc définir des valeurs pour tout le site, que chaque page complète. Voir [le guide SEO d'active](https://active-c1883a966650.herokuapp.com/fr/docs/seo/).

## La Content-Security-Policy

Chaque page est envoyée avec un en-tête `Content-Security-Policy` qui indique au navigateur ce que la page a le droit de charger et d'exécuter. Paquets le calcule à partir de la page elle-même :

| Autorisé | Bloqué |
|---|---|
| les fichiers de votre propre site (`/site.css`, `/app.js`, images, polices) | les scripts, feuilles de style et polices d'autres sites (CDN, Google Fonts) |
| les blocs `<style>` et les balises `<script>` en ligne de la page, par leur empreinte SHA-256 | tout script ou style en ligne qui n'était pas dans la page au moment du rendu |
| les attributs `style="..."` | les gestionnaires d'événements en ligne comme `onclick="..."` |
| les images de votre site, des URL `https:` et des URL `data:` | l'affichage dans un cadre (frame) sur un autre site |
| le WebAssembly de vos îlots | |

Même si un attaquant parvenait à glisser du HTML dans une page (via un champ de commentaire, par exemple), son script ne s'exécuterait pas.

Les fichiers HTML statiques de `public/` (une page d'accueil, ou un site exporté par un générateur de site statique) sont couverts eux aussi : au démarrage, le bouclier par défaut les lit, et sa politique autorise leurs scripts et styles en ligne (par empreinte), les feuilles de style et scripts externes qu'ils lient (Google Fonts compris, avec ses fichiers de polices), le WebAssembly et les images `https:`. Cette politique automatique appartient au bouclier par défaut : si vous remplacez le bouclier avec `app.security(...)`, définissez la politique vous-même (voir [Sécurité](security.md#les-en-têtes-de-sécurité-et-la-content-security-policy)).

Le plus simple pour utiliser une police ou une bibliothèque d'un autre site est de **la télécharger dans `public/`** et de la lier depuis là : c'est aussi plus rapide et plus respectueux de la vie privée de vos visiteurs. Si vous avez vraiment besoin d'une source externe, ajoutez-la à la politique de cette réponse :

```rust
app.get("/", |_| async {
    let res = Page::new("Accueil")
        .stylesheet("https://fonts.googleapis.com/css2?family=Inter")
        .body(h1().text("Bonjour"))
        .into_response();
    // Autoriser Google Fonts dans la politique de cette page.
    let csp = res
        .get_header("content-security-policy")
        .unwrap_or_default()
        .replace("style-src 'self'", "style-src 'self' https://fonts.googleapis.com")
        .replace("font-src 'self'", "font-src 'self' https://fonts.gstatic.com");
    res.header("content-security-policy", csp)
});
```

## Formulaires

Un formulaire est une vue comme une autre. Faites pointer son `action` vers une route qui le lit avec `req.form()` :

```rust
fn contact_form() -> Element {
    form()
        .attr("method", "post")
        .attr("action", "/contact")
        .child(label().attr("for", "email").text("Votre e-mail"))
        .child(input().attr("type", "email").attr("name", "email").id("email"))
        .child(label().attr("for", "message").text("Message"))
        .child(textarea().attr("name", "message").id("message"))
        .child(button().text("Envoyer"))
}
```

Aucun jeton CSRF n'est nécessaire : le bouclier refuse les formulaires envoyés depuis d'autres sites. Le côté handler se trouve dans [Requêtes et réponses](requests-responses.md#formulaires).

## Parties interactives

Une page, c'est du HTML statique. Pour qu'une partie réagisse dans le navigateur (un compteur, un champ de recherche, un panier), faites-en un **îlot** : `island("counter", "0", counter)`. Seules les pages qui contiennent des îlots chargent le bundle du navigateur. Voir [Îlots](islands.md).

## Application installable (PWA)

Une ligne rend l'application installable sur les téléphones et les ordinateurs, et utilisable hors ligne :

```rust
app.pwa(
    Pwa::new("Mes tâches")
        .short_name("Tâches")
        .theme_color("#6d4aff")
        .background_color("#0b0b12"),
);
```

- L'application sert le manifeste d'application web à `/manifest.webmanifest` et un service worker à `/sw.js` ; chaque `Page` les référence (avec la couleur du thème, le titre sur l'écran d'accueil et l'icône Apple), et la politique de la page autorise le script d'enregistrement.
- Hors ligne : la page de départ, le bundle des îlots et les icônes sont téléchargés à l'installation. Les pages viennent d'abord du réseau, puis du cache quand le réseau est coupé ; les API REST de `app.api` viennent toujours du réseau.
- Sans icônes, c'est la boîte de Paquets qui est utilisée. Vos propres icônes (au moins un PNG de 192×192 et un de 512×512 dans `public/`) : `.icon("/icon-192.png", "192x192", "image/png").icon("/icon-512.png", "512x512", "image/png").maskable_icon("/icon-maskable.png", "512x512", "image/png")`.
- `.start_url("/{lang}/")` donne à un site en plusieurs langues un manifeste par langue (`/fr/manifest.webmanifest`, lié par les pages dont `Page::lang` vaut `fr`) : l'application installée s'ouvre dans la langue de la page depuis laquelle elle a été installée.
- `.start_url("/demo")` ouvre une autre page ; `.description(...)`, `.lang("fr")`, `.display("fullscreen")`… complètent le manifeste (c'est [le module `pwa` d'active](https://docs.rs/active/latest/active/pwa/index.html) qui le construit).
- Les navigateurs n'installent une PWA que servie en HTTPS (ou sur `localhost`) : Heroku donne le HTTPS à chaque application. Sur un téléphone, ouvrez le site, puis *Sur l'écran d'accueil* (menu de partage de Safari) ou *Installer l'application* (menu de Chrome).

## Pour aller plus loin

Les vues viennent d'[Active](https://active-c1883a966650.herokuapp.com/). Sa documentation approfondit [les composants et les vues](https://active-c1883a966650.herokuapp.com/fr/docs/components/), [le SEO](https://active-c1883a966650.herokuapp.com/fr/docs/seo/) et [les données structurées](https://active-c1883a966650.herokuapp.com/fr/docs/structured-data/).

## Étapes suivantes

- [Base de données](database.md) : remplir vos pages de données.
- [Îlots](islands.md) : des parties interactives en Rust.
- [Sécurité](security.md) : le bouclier et les en-têtes qu'il ajoute.
