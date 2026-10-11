# Îlots

Un îlot est une partie interactive d'une page rendue par le serveur : un compteur, un champ de recherche, un sélecteur de quantité. Le serveur le rend en HTML comme le reste de la page. Dans le navigateur, un petit bundle WebAssembly, compilé à partir de la même fonction Rust, rend interactive cette partie, et elle seule. Le reste de la page reste du HTML simple et ne coûte rien. Cette page montre comment écrire des îlots, les placer dans des pages, les compiler et les déployer.

## Comment ça marche

```text
src/lib.rs ── fn counter(props) ──┬── sur le serveur : rendu en HTML dans la page
                                  │
                                  └── paquets build ──▶ pkg/mon_app.js + pkg/mon_app_bg.wasm
                                                        chargés par le navigateur, qui rend
                                                        interactif le compteur de la page
```

1. Le serveur appelle la fonction de l'îlot et écrit son HTML dans la page, à l'intérieur d'un élément `<active-island>` qui enregistre son nom et ses props.
2. Les visiteurs voient le contenu immédiatement, avant même qu'un script ne s'exécute. Les moteurs de recherche le voient aussi.
3. Le navigateur charge le bundle, appelle la même fonction avec les mêmes props, reprend le HTML déjà présent (sans le redessiner) et y attache les gestionnaires d'événements.

Seules les pages qui contiennent au moins un îlot chargent le bundle. Un îlot compteur pèse environ **36,8 ko** compressé, WebAssembly et chargeur compris : le plus léger parmi les frameworks front-end en Rust.

## Prérequis

Les îlots sont compilés en WebAssembly, ce qui demande deux outils :

- la cible `wasm32-unknown-unknown` de Rust ;
- `wasm-bindgen-cli`, exactement à la version de la crate `wasm-bindgen` de votre `Cargo.lock`.

Le premier `paquets dev` dans un projet avec des îlots installe ce qui manque (cela prend quelques minutes, une seule fois). `paquets doctor` affiche ce qui est installé. Voir [Installation](installation.md#3-les-outils-pour-les-îlots-facultatif).

## Votre premier îlot

Un projet créé avec `paquets new` (sans `--minimal`) a déjà un îlot, `counter`, dans `src/lib.rs`. Voici le cycle complet, avec un compteur plus simple.

### 1. Écrire l'îlot dans src/lib.rs

Un îlot est une fonction qui reçoit ses props sous forme de chaîne et renvoie un `Node` :

```rust
// src/lib.rs
use paquets::prelude::*;

/// Un compteur : les props sont la valeur de départ.
pub fn counter(props: &str) -> Node {
    let count = use_state(props.parse::<i32>().unwrap_or(0));
    button()
        .on_click(move |_| count.update(|n| *n += 1))
        .text_dyn(move || format!("Cliqué {} fois", count.get()))
        .into()
}
```

- `use_state(initial)` crée un état : une valeur qui peut changer dans le navigateur.
- `.on_click(...)` exécute du code quand on clique sur le bouton. Ici, il ajoute 1 à l'état.
- `.text_dyn(...)` affiche un texte calculé à partir de l'état. Quand l'état change, seul ce texte est mis à jour dans la page.

L'îlot doit se trouver dans `src/lib.rs` (ou dans un module de la bibliothèque), car le bundle du navigateur est compilé à partir de la bibliothèque.

### 2. L'enregistrer pour le navigateur

À la fin de `src/lib.rs`, le point d'entrée du navigateur liste tous les îlots, par nom :

```rust
/// Le point d'entrée du navigateur : rend interactifs tous les îlots de la page.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&[("counter", counter)]);
}
```

`#[wasm_bindgen(start)]` exécute `start` dès que le bundle est chargé. `#[cfg(target_arch = "wasm32")]` tient cette fonction à l'écart de la compilation du serveur.

### 3. Le placer dans une page

Dans `src/main.rs`, importez l'îlot et placez-le avec `island(nom, props, fonction)` :

```rust
// src/main.rs
use paquets::prelude::*;

use mon_app::counter; // `mon_app` est le nom de votre paquet, avec `_` à la place de `-`

async fn home(_req: Request) -> Page {
    Page::new("Accueil")
        .body(h1().text("Un compteur"))
        .body(island("counter", "0", counter))
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", home);
    app.run()
}
```

Le nom (`"counter"`) doit être le même ici et dans `start()`. Les props (`"0"`) sont la valeur de départ.

### 4. Le lancer

```sh
paquets dev
```

`paquets dev` compile `src/lib.rs` en WebAssembly (en mode debug), écrit le bundle dans `pkg/`, puis lance le serveur. La ligne de démarrage affiche le bundle trouvé :

```text
📦 Paquets · http://localhost:3000
   database: memory · security: on (10 guards) · islands: /pkg/mon_app.js
```

Ouvrez la page et cliquez sur le bouton. Après avoir modifié un îlot, relancez `paquets dev` : `cargo run` seul recompile le serveur, pas le bundle.

## État, événements et vues dynamiques

### État

| Méthode | Pour |
|---|---|
| `use_state(initial)` | créer un état ; la poignée est `Copy`, déplacez-la dans autant de closures que nécessaire |
| `state.get()` | lire une copie de la valeur |
| `state.with(\|v\| ...)` | lire à travers une référence, sans copie (`text.with(String::len)`) |
| `state.set(valeur)` | remplacer la valeur |
| `state.update(\|v\| ...)` | modifier la valeur sur place (`*n += 1`, `list.push(...)`) |
| `memo(\|\| ...)` | une valeur dérivée d'autres états, recalculée seulement quand ils changent, et qui ne notifie que quand son résultat change (`memo(move \|\| items.with(Vec::len))`) |
| `untrack(\|\| ...)` | lire des états sans les suivre |
| `on_cleanup(\|\| ...)` | exécuter du code quand la partie de la page qui l'a créé disparaît (arrêter un minuteur, retirer un écouteur) |

### Événements

| Méthode | Événement |
|---|---|
| `.on_click(\|e\| ...)` | un clic |
| `.on_input(\|e\| ...)` | la valeur d'un champ change, à chaque frappe |
| `.on("keydown", \|e\| ...)` | n'importe quel événement du DOM, par son nom : `"change"`, `"submit"`, `"keydown"`, `"focus"`… |

L'événement `e` fournit `e.value()` (la valeur du champ), `e.key()` (la touche d'un événement clavier) et `e.prevent_default()`.

### Vues dynamiques

Les méthodes qui prennent une closure au lieu d'une valeur suivent l'état : la closure s'exécute à nouveau quand un état qu'elle lit change, et seule cette partie de la page est mise à jour.

| Méthode | Met à jour |
|---|---|
| `.text_dyn(f)` | un texte |
| `.attr_dyn(nom, f)` | un attribut (`value` met aussi à jour un champ) |
| `.class_if(classe, f)` | ajoute une classe tant que `f()` vaut `true` |
| `.bool_attr(nom, f)` | un attribut booléen : `disabled`, `hidden`, `checked`… |
| `.children_dyn(f)` | une liste d'enfants, reconstruite quand elle change |

Un champ de texte qui compte ses caractères :

```rust
/// Un champ de texte qui compte ses caractères. Les props sont le maximum.
pub fn limited_text(props: &str) -> Node {
    let max: usize = props.parse().unwrap_or(280);
    let text = use_state(String::new());
    let count = move || text.with(|t| t.chars().count());
    div()
        .child(textarea().attr("name", "message").on_input(move |e| text.set(e.value())))
        .child(
            p().class("muted")
                .class_if("too-long", move || count() > max)
                .text_dyn(move || format!("{} / {max}", count())),
        )
        .into()
}
```

Placez-le avec `island("limited-text", "140", limited_text)` et ajoutez `("limited-text", limited_text)` à la liste de `start()`.

## Props

Les props voyagent dans le HTML de la page, sous forme de chaîne. Utilisez un nombre ou un mot dans les cas simples, et du JSON pour des données structurées :

```rust
// src/lib.rs
/// Les props de `add_to_cart`, envoyées en JSON.
#[data]
#[derive(Default)]
pub struct CartProps {
    pub product: String,
    pub price_cents: u32,
}

/// Un champ de quantité et le prix total.
pub fn add_to_cart(props: &str) -> Node {
    let props: CartProps = paquets::serde_json::from_str(props).unwrap_or_default();
    let price = props.price_cents;
    let quantity = use_state(1u32);
    div()
        .class("row")
        .child(strong().text(props.product))
        .child(
            input()
                .attr("type", "number")
                .attr("min", "1")
                .attr_dyn("value", move || quantity.get().to_string())
                .on_input(move |e| quantity.set(e.value().parse().unwrap_or(1))),
        )
        .child(span().text_dyn(move || {
            let total = quantity.get() * price;
            format!("{},{:02} €", total / 100, total % 100)
        }))
        .into()
}
```

```rust
// src/main.rs
use mon_app::{CartProps, add_to_cart};

async fn product(_req: Request) -> Page {
    let props = CartProps { product: "Chaise en chêne".into(), price_cents: 24900 };
    Page::new("Chaise en chêne")
        .body(h1().text("Chaise en chêne"))
        .body(island("add-to-cart", paquets::serde_json::to_string(&props).unwrap(), add_to_cart))
}
```

Les props font partie du HTML : gardez-les légères, et n'y mettez jamais de secrets.

## Les règles des îlots

- **Tout vient des props.** Un îlot est une simple fonction (`fn(&str) -> Node`) qui ne capture rien. Dans le navigateur, il n'a accès ni à votre base de données ni à l'état de votre serveur : donnez-lui ce dont il a besoin par ses props. Une vue qui dépend de quelque chose que seul le serveur possède (l'heure, un nombre aléatoire, une valeur de la base de données qui ne passe pas par les props) produit un HTML différent dans le navigateur, que celui-ci redessine alors, avec un avertissement dans sa console : `active: re-rendering island (tag mismatch)`.
- **Les îlots sont indépendants.** Chaque îlot a ses propres états. Si deux parties d'une page doivent réagir l'une à l'autre (un filtre et une liste), faites-en un seul îlot, ou partagez un store (voir [Garder des données dans le navigateur](#garder-des-données-dans-le-navigateur)).
- **Le même balisage des deux côtés.** N'utilisez `#[cfg(target_arch = "wasm32")]` que pour le comportement, jamais pour changer ce qui est rendu.
- **Les noms doivent correspondre.** Un îlot dont le nom n'est pas dans `start()` reste tout simplement statique, sans la moindre erreur.
- **Les données du serveur.** Pour lire ou envoyer des données une fois la page chargée, appelez votre API avec le `fetch` du navigateur via `web-sys` et `wasm-bindgen-futures` (voir [le guide des événements d'active](https://maxlestage.github.io/Active/fr/docs/events/)), ou utilisez un simple formulaire HTML qui envoie ses données à une route.

## Garder des données dans le navigateur

Avec la feature `browser-storage`, les îlots peuvent garder des données dans le navigateur du visiteur, grâce au module `web` de Memoire (`paquets::orm::web`) :

- **`Store<S>`**, un état d'application dans l'esprit de Pinia : `Store::new("cart")` renvoie le même store dans chaque îlot de la page, enregistré dans `localStorage` à chaque changement, restauré à la visite suivante et synchronisé entre les onglets ;
- **`LocalStorageDb`**, une base Memoire dans `localStorage` : modèles, requêtes et transactions, comme sur le serveur.

Les îlots de `src/lib.rs` sont compilés pour les deux côtés (rendus par le serveur, puis hydratés dans le navigateur) : activez donc la feature dans les deux dépendances :

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", features = ["browser-storage"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client", "browser-storage"] }
```

Deux îlots qui partagent un panier, le badge du menu et le bouton d'une page produit :

```rust
use paquets::orm::web::Store;
use paquets::prelude::*;

#[data]
#[derive(Default)]
pub struct Cart {
    pub items: Vec<String>,
}

/// Le nombre d'articles du panier, tenu à jour dans chaque îlot et dans chaque onglet.
pub fn cart_badge(_props: &str) -> Node {
    let count = use_state(0);
    // Les effets ne s'exécutent que dans le navigateur : le serveur rend un panier vide.
    effect(move || {
        let cart = Store::<Cart>::new("cart");
        count.set(cart.get(|c| c.items.len()));
        let follow = cart.subscribe_local(move |c| count.set(c.items.len()));
        on_cleanup(move || drop(follow));
    });
    span().class("badge").text_dyn(move || count.get().to_string()).into()
}

/// Le bouton d'une page produit. Les props sont le nom du produit.
pub fn add_to_cart(product: &str) -> Node {
    let product = product.to_owned();
    button()
        .text("Ajouter au panier")
        .on_click(move |_| {
            Store::<Cart>::new("cart").update(|c| c.items.push(product.clone()));
        })
        .into()
}
```

Les données restent dans le navigateur : le serveur ne les voit jamais, sauf si un îlot les envoie (à votre API, avec `fetch`). Voir [le guide du stockage dans le navigateur](https://maxlestage.github.io/Memoire/fr/docs/browser-storage/) de Memoire pour les versions et les migrations, `SessionStorage` et `LocalStorageDb`.

## Compiler pour la production

```sh
paquets build
```

`paquets build` compile les îlots en mode release, avec le profil `wasm-release` optimisé pour la taille, dans `pkg/`, puis compile le serveur avec `cargo build --release`. Déployez `pkg/` (et `public/`) à côté du programme : le `Dockerfile` d'un projet généré le fait déjà (voir [Docker](deploy-docker.md)).

Au démarrage, l'application cherche `pkg/<nom>.js` et `pkg/<nom>_bg.wasm` dans le dossier courant et les sert sous `/pkg/`. Les pages qui contiennent des îlots reçoivent alors trois balises dans leur `<head>` : un préchargement du module, un préchargement du fichier WebAssembly (téléchargé en parallèle de la page), et la ligne qui le démarre. Leur Content-Security-Policy autorise le WebAssembly. Les pages sans îlots ne reçoivent rien de tout cela.

## Préparer un projet à la main

Un projet créé avec `paquets new --minimal`, ou un projet existant, demande quatre modifications pour accueillir des îlots.

**1. `Cargo.toml`** : une crate, deux cibles. La bibliothèque est aussi compilée en WebAssembly (`cdylib`), et Paquets est utilisé sans son serveur ni ses bases de données dans le navigateur :

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
paquets = { git = "https://github.com/maxlestage/Paquets", default-features = false, features = ["client"] }
wasm-bindgen = "0.2"

# Le bundle du navigateur : le plus petit possible.
[profile.wasm-release]
inherits = "release"
opt-level = "s"
lto = true
panic = "abort"
```

Retirez `paquets` de la section `[dependencies]` ordinaire : gardez-le uniquement dans les deux sections par cible.

**2. `src/lib.rs`** : les îlots et la fonction `start()` présentée plus haut.

**3. `src/main.rs`** : `use mon_app::counter;` et `island(...)` dans une page.

**4. `.gitignore`** : ajoutez `/pkg`.

Ensuite, `paquets dev` compile tout. Sans l'outil `paquets`, voici les commandes qu'il lance :

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # la version du Cargo.lock
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --out-dir pkg \
  target/wasm32-unknown-unknown/wasm-release/mon_app.wasm
cargo run --release
```

Le fichier `.wasm` porte le nom du paquet, avec `-` remplacé par `_`.

## Dépannage

| Problème | Solution |
|---|---|
| La page affiche l'îlot, mais cliquer ne fait rien | Regardez la ligne de démarrage : `islands: none` signifie que `pkg/` ne contient pas de bundle ; lancez `paquets dev`. Vérifiez que le nom passé à `island(...)` figure dans la liste de `start()`. Ouvrez la console du navigateur pour voir les erreurs. |
| `active: re-rendering island (… mismatch)` dans la console | Le HTML du serveur et la vue du navigateur diffèrent : faites passer toutes les entrées par les props (voir les règles plus haut). |
| `wasm-bindgen` signale une différence de version ou de schéma | `cargo install wasm-bindgen-cli --version <version du Cargo.lock> --locked`, ou laissez `paquets dev` s'en charger. |
| Des erreurs à propos de tokio, des sockets ou de `mio` en compilant pour wasm32 | Une dépendance propre au serveur est compilée pour le navigateur : déplacez-la sous `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. |
| `hydrate` introuvable | La feature `client` manque sur la ligne `paquets` de wasm32, ou le code n'est pas derrière `#[cfg(target_arch = "wasm32")]`. |
| Les modifications d'un îlot n'apparaissent pas | Recompilez le bundle avec `paquets dev` (pas seulement `cargo run`), puis rechargez la page. |

## Pour aller plus loin

Les îlots, les états et les événements viennent d'[Active](https://maxlestage.github.io/Active/). Sa documentation va plus loin : [îlots](https://maxlestage.github.io/Active/fr/docs/islands/), [état et effets](https://maxlestage.github.io/Active/fr/docs/state/), [événements](https://maxlestage.github.io/Active/fr/docs/events/), et [applications côté client](https://maxlestage.github.io/Active/fr/docs/client-app/) (`paquets::mount_to`, `navigate` et `location` avec la feature `client`).

## Étapes suivantes

- [Pages](pages.md) : le reste de la page, rendu par le serveur.
- [Ligne de commande](cli.md) : `paquets dev` et `paquets build` en détail.
- [Déployer avec Docker](deploy-docker.md) : comment les îlots sont compilés dans l'image.
