# Introduction

Paquets 📦 est **le framework full-stack le plus simple pour Rust**. Une crate, un import et une `App` vous donnent un serveur web rapide, une protection devant chaque requête, une base de données, des API REST, des pages web avec toutes leurs balises SEO, et des parties interactives qui s'exécutent dans le navigateur en WebAssembly. Cette page explique ce qu'est Paquets, ce qu'il fait pour vous, et ce qu'il ne fait pas.

## Qu'est-ce que Paquets ?

Paquets est une bibliothèque Rust pour créer des sites et des API web. Il assemble quatre paquets écrits par le même auteur, chacun utilisable seul :

| Paquet | Son rôle dans Paquets |
|---|---|
| [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/) | Le serveur HTTP, avec une API dans le style d'Express.js : `app.get(...)`, `req.param(...)`, middlewares, routeurs, WebSocket. |
| [Securite](https://securite-2db4c1994be7.herokuapp.com/) | Le bouclier de sécurité devant chaque requête : pare-feu, limitation de débit, protection CSRF, en-têtes de sécurité, et bien plus. |
| [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/) | La base de données : modèles et requêtes typés, en mémoire, SQLite, PostgreSQL, MySQL, Redis ou MongoDB. |
| [Active](https://active-c1883a966650.herokuapp.com/) | Le front-end : des vues écrites comme des fonctions Rust, le rendu serveur avec les balises SEO, et les îlots WebAssembly. |

Pas besoin d'apprendre quatre bibliothèques pour commencer. Paquets les relie, choisit de bons réglages par défaut et expose une seule petite API. Quand il vous en faut plus, chaque paquet n'est qu'à un import de distance (voir [Les quatre paquets](packages.md)).

## Une application complète

Voici un programme complet. Il sert une page d'accueil, une route qui vous salue et une API REST adossée à une base de données :

```rust
use paquets::prelude::*;

#[model]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.get("/", |_| async {
        Page::new("Accueil")
            .description("Mon site, en Rust.")
            .body(html::main().child(h1().text("Bonjour 👋")))
    });

    app.get("/hello/:name", |req: Request| async move {
        format!("Bonjour {} !", req.param("name").unwrap_or("toi"))
    });

    app.api::<Todo>("/api/todos");

    app.run()
}
```

Lancez-le, et vous obtenez :

- `http://localhost:3000/` : une page HTML mise en forme, avec son titre, sa description et ses balises Open Graph et Twitter, en mode clair ou sombre ;
- `http://localhost:3000/hello/Ada` : le texte `Bonjour Ada !` ;
- `http://localhost:3000/api/todos` : une API JSON pour lister, lire, créer, remplacer, modifier et supprimer des tâches (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`), enregistrées dans une table de la base de données créée au démarrage.

> [!NOTE]
> Dans un fichier qui contient `fn main`, la balise HTML `<main>` s'écrit `html::main()`, car le nom `main` appartient déjà à la fonction de votre programme. Partout ailleurs, `main()` fonctionne. Voir [Pages](pages.md#la-balise-main-dans-mainrs).

## Ce que vous obtenez sans rien configurer

`App::new()` est prêt pour la production. Rien dans le tableau ci-dessous ne demande la moindre ligne de configuration :

| Fonctionnalité | Comportement par défaut | Pour le changer |
|---|---|---|
| Serveur | Le port de la variable `PORT` (Heroku), sinon `3000`. `Ctrl+C` ou `SIGTERM` l'arrête proprement. | `app.port(8080)` |
| Sécurité | Le bouclier recommandé de Securite sur chaque requête : identifiants de requête, contrôles du protocole, corps limité à 2 Mio, limite de 20 requêtes par seconde et par IP, blocage des robots, pare-feu, protection CSRF, en-têtes de sécurité, cookies durcis. | `app.security(...)` |
| Base de données | D'après la variable `DATABASE_URL` ; en mémoire si elle n'est pas définie. Les tables de vos modèles sont créées au démarrage. | `app.database(...)` |
| Pages | Du HTML rendu par le serveur avec les balises SEO, un thème intégré et une Content-Security-Policy qui autorise exactement ce que contient la page. | les méthodes de `Page` |
| Fichiers statiques | Le dossier `public/` est servi à la racine du site. | `app.static_dir(...)` |
| Îlots | Le bundle du navigateur dans `pkg/` (construit par `paquets build`) est trouvé au démarrage et chargé uniquement par les pages qui contiennent des îlots. | — |
| Journaux | Une ligne par requête : `GET /hello/Ada 200 0.339 ms`. | `app.quiet()` |

## Pourquoi Paquets ?

- **La simplicité.** Une dépendance, un import (`use paquets::prelude::*;`), une `App`. Un handler est une fonction `async` qui prend une requête et renvoie une réponse. Les erreurs sont des valeurs, et `?` les transforme en bon statut HTTP.
- **Sécurisé par défaut.** Les attaques comme l'injection SQL dans l'URL, la falsification de requête intersite (CSRF), le clickjacking ou la force brute sont bloquées avant que votre code ne s'exécute. C'est la protection qu'il faut désactiver, pas l'inverse.
- **Rapide.** Vitesse traite environ 291 000 requêtes par seconde pour `GET /` sur 2 cœurs, plus qu'actix-web et axum sur la même machine, et 2,78 millions par seconde avec le pipelining. Toute la chaîne de Securite coûte environ 6 µs par requête. Le plus petit bundle d'îlot (un compteur) pèse 36,8 ko compressé. Voir [Performances](performance.md).
- **Typé de la base de données jusqu'à la page.** Une colonne mal orthographiée, une comparaison entre un nombre et une chaîne ou un champ manquant dans une réponse est une erreur de compilation, pas un bug en production.
- **Facile à déployer.** Un projet créé avec `paquets new` contient un `Dockerfile`, les fichiers pour déployer sur Heroku d'un simple geste et des workflows GitHub. Vous pouvez déployer, redéployer et lire vos journaux [depuis un téléphone](deploy-heroku-mobile.md).

## Pour qui ?

- **Les débutants en Rust** qui veulent construire un vrai site. La plupart des handlers tiennent en quelques lignes, et cette documentation explique les subtilités propres à Rust au fur et à mesure.
- **Les développeurs web** qui viennent d'Express, Rails, Laravel ou Django et veulent un seul framework pour toute la pile, sans assembler une douzaine de crates.
- **Les développeurs Rust** qui veulent un framework petit et explicite, sécurisé par défaut, et qui aiment lire le code dont ils dépendent.

## Ce que Paquets ne fait pas (encore)

Paquets est jeune (version 0.1). Gardez ces limites en tête avant de le choisir :

- **HTTPS, HTTP/2 et HTTP/3 sont des options.** Sur une plateforme comme Heroku, son routeur fournit HTTPS devant l'application. Pour le servir depuis l'application elle-même, activez la feature `tls` (et `http2`, `http3`) : voir [Configuration](configuration.md#https-dans-le-serveur).
- **Pas de modification automatique du schéma.** Les tables sont créées quand elles manquent, mais ajouter un champ à un modèle ne modifie pas une table existante : écrivez une migration (voir [Base de données](database.md#modifier-une-table-plus-tard)).
- **Les îlots demandent deux outils de plus** : la cible `wasm32-unknown-unknown` et `wasm-bindgen-cli`. `paquets dev` les installe pour vous la première fois. Les pages sans îlots n'ont besoin de rien.
- **Pas de comptes utilisateurs intégrés.** Le bouclier protège chaque requête et Vitesse tient les sessions (feature `session`), mais enregistrer les utilisateurs et vérifier leurs mots de passe reste à votre charge. Voir [Sécurité](security.md#comptes-et-sessions).
- **Pas encore sur crates.io.** Tant que les quatre paquets et Paquets ne sont pas publiés, les projets dépendent du dépôt GitHub. Voir [Installation](installation.md) et [Publier sur crates.io](publish-crates-io.md).

## Organisation de cette documentation

- **Démarrer** : l'[Installation](installation.md), un tutoriel de [Démarrage rapide](quick-start.md) et la [Structure d'un projet](project-structure.md).
- **L'essentiel** : [Routes](routing.md), [Requêtes et réponses](requests-responses.md), [Pages](pages.md), [Base de données](database.md), [API REST](rest-api.md) et [Sécurité](security.md).
- **Aller plus loin** : [Îlots](islands.md), [WebSocket](websocket.md), [Tests](testing.md), [Erreurs](errors.md), [Configuration](configuration.md) et [Performances](performance.md).
- **Déployer** : [Heroku depuis un téléphone](deploy-heroku-mobile.md), [Docker](deploy-docker.md), [Publier sur crates.io](publish-crates-io.md) et [Automatisation](automation.md).
- **Référence** : la [Ligne de commande](cli.md), l'[Aide-mémoire](cheatsheet.md), [Les quatre paquets](packages.md) et la [FAQ](faq.md).

## Étapes suivantes

- [Installation](installation.md) : installer Rust et la commande `paquets`.
- [Démarrage rapide](quick-start.md) : construire votre première application en dix minutes.
