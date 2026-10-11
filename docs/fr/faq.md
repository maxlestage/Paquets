# FAQ

Des réponses courtes aux questions fréquentes, avec des liens vers les pages qui vont plus loin.

## À propos de Paquets

### Qu'est-ce que Paquets, en une phrase ?

Un framework Rust qui vous donne, avec une seule crate et une seule `App`, un serveur rapide, une protection devant chaque requête, une base de données avec des requêtes typées, des API REST, des pages rendues par le serveur avec toutes leurs balises SEO, et des îlots WebAssembly interactifs. Voir l'[Introduction](introduction.md).

### Est-ce prêt pour la production ?

Paquets est jeune (version 0.1). Il repose sur quatre paquets testés chacun de leur côté (Vitesse, Securite, memoire-orm, Active), et ses réglages par défaut sont pensés pour la production : bouclier de sécurité, arrêt propre, base de données issue de `DATABASE_URL`, compilations release dans le `Dockerfile`. Mais son API peut encore changer d'une version 0.x à l'autre, et il n'a pas des années d'utilisation derrière lui. Essayez-le sur un projet où vous pouvez suivre son évolution, et lisez les limites dans l'[Introduction](introduction.md#ce-que-paquets-ne-fait-pas-encore).

### Pourquoi pas axum, actix-web ou Leptos ?

Ce sont d'excellents projets, plus gros. Paquets fait d'autres compromis :

- **Tout au même endroit.** Serveur, sécurité, base de données, pages et îlots fonctionnent ensemble, sans avoir à choisir et câbler une douzaine de crates.
- **Sécurisé par défaut.** Pare-feu, limitation de débit, protection CSRF et en-têtes de sécurité sont actifs avant que vous n'écriviez la moindre ligne.
- **Pas de macros dans les vues, pas d'extracteurs dans les handlers.** Un handler prend une `Request` et renvoie une réponse ; une vue est une chaîne d'appels de fonctions. rust-analyzer et le compilateur comprennent tout.
- **Rapide.** Son serveur dépasse actix-web et axum dans ses benchmarks (voir [Performances](performance.md)).

Choisissez un framework plus gros si vous avez besoin de son écosystème, de fonctions serveur, ou si votre équipe le connaît déjà.

### Faut-il bien connaître Rust ?

Non. La plupart des handlers tiennent en quelques lignes, et cette documentation explique les subtilités propres à Rust au fur et à mesure (`async move`, `?`, `Option`). Connaître les bases du langage (les premiers chapitres du [Rust book](https://doc.rust-lang.org/book/)) aide beaucoup.

### Faut-il écrire du JavaScript ?

Non pour les pages, les API et les îlots : tout est en Rust, y compris les parties interactives. Le navigateur reçoit un petit chargeur JavaScript généré par `wasm-bindgen`, que vous n'écrivez ni ne modifiez jamais. Pour un client WebSocket dans le navigateur, un petit script reste l'option la plus simple (voir [WebSocket](websocket.md#se-connecter-depuis-une-page)).

### Est-ce gratuit ?

Paquets s'utilise librement comme dépendance, y compris dans des projets commerciaux (c'est un logiciel propriétaire : voir [Quelle licence ?](#quelle-licence-)). L'hébergement, c'est une autre histoire : Heroku n'a pas d'offre gratuite (un dyno Eco coûte environ 5 $ par mois, voir [Déployer sur Heroku](deploy-heroku-mobile.md)). L'image Docker tourne chez n'importe quel hébergeur, y compris sur votre propre serveur.

## Écrire du code

### Pourquoi `main().child(...)` échoue-t-il avec « no method named child found for enum Result » ?

Dans un fichier qui définit `fn main` (en général `src/main.rs`), le nom `main` désigne la fonction de votre programme, qui masque la balise HTML `main()`. Écrivez `html::main()` dans ce fichier. Partout ailleurs, `main()` fonctionne. Voir [Pages](pages.md#la-balise-main-dans-mainrs).

### Pourquoi le compilateur dit-il « future cannot be sent between threads safely » ?

La future d'un handler doit pouvoir passer d'un thread à l'autre, et deux choses l'en empêchent :

- une **vue** (`div()`, `Page`…) créée avant un `.await` : construisez la page après le dernier `.await` (voir [Pages](pages.md#construire-la-page-à-la-fin-du-handler)) ;
- un verrou de `std::sync::Mutex` ou de `RwLock` conservé de part et d'autre d'un `.await` : lisez d'abord le corps ou la base de données, puis prenez le verrou, ou utilisez `tokio::sync::Mutex`.

### Comment changer le format de toutes les erreurs ?

Avec `app.on_error(|err: Error| ...)` : la fonction reçoit une `paquets::Error` et renvoie n'importe quelle réponse. Définissez vous-même le statut avec `res::status(err.status())`. Voir [Erreurs](errors.md#changer-le-format-de-toutes-les-erreurs).

### Comment partager des données entre les requêtes ?

Pour des données qui doivent durer, utilisez la [base de données](database.md). Pour des réglages, des caches et des compteurs qui vivent aussi longtemps que le serveur, utilisez `app.state(valeur)` et `req.state::<T>()` (voir [Configuration](configuration.md#état-partagé)).

### Comment obtenir l'adresse IP du client ?

`req.extensions().get::<paquets::securite::ClientIp>()` renvoie l'adresse trouvée par le bouclier, qui est correcte derrière le routeur de Heroku. Voir [Requêtes et réponses](requests-responses.md#ladresse-ip-du-client).

### Comment connecter les utilisateurs ?

Paquets n'inclut pas de comptes utilisateurs. Selon vos besoins : un jeton partagé pour une API, le guard `Auth` de Securite pour une section d'administration, ou des sessions enregistrées dans votre base de données. Voir [Sécurité](security.md#authentification).

### Les composants peuvent-ils être asynchrones ?

Non : les composants sont de simples fonctions qui construisent une vue. Faites d'abord le travail asynchrone (base de données, appels HTTP) dans le handler, puis construisez la page avec les résultats.

### Un îlot peut-il appeler mon API ?

Oui, avec le `fetch` du navigateur, via les crates `web-sys` et `wasm-bindgen-futures` (voir [le guide des événements d'active](https://maxlestage.github.io/Active/fr/docs/events/)). Un îlot peut aussi contenir un simple formulaire HTML qui envoie ses données à l'une de vos routes.

### Y a-t-il du rechargement à chaud ?

Pas intégré. `paquets dev` compile et lance l'application une fois ; après une modification, arrêtez-la et relancez-la. Un outil de surveillance de fichiers automatise cela : `cargo watch -s "paquets dev"` (installez-le avec `cargo install cargo-watch`).

## Base de données

### Quelle base de données choisir ?

En mémoire pendant l'apprentissage et dans les tests, SQLite pour un seul serveur avec un disque persistant, PostgreSQL sur Heroku ou avec plusieurs serveurs. Voir [Base de données](database.md#quelle-base-de-données-choisir-).

### Pourquoi mes données ont-elles disparu ?

Sans `DATABASE_URL`, la base de données est en mémoire et se vide à l'arrêt du serveur. Sur Heroku, les fichiers SQLite sont eux aussi effacés à chaque redémarrage. Utilisez PostgreSQL (l'add-on Heroku Postgres) pour les données qui doivent durer.

### J'ai ajouté un champ à un modèle, et j'obtiens maintenant des erreurs 500.

Les tables sont créées quand elles manquent, jamais modifiées : la table existante n'a pas la nouvelle colonne. Ajoutez une migration. Voir [Base de données](database.md#modifier-une-table-plus-tard).

### Comment voir le SQL d'une requête ?

`Todo::query().filter(Todo::DONE.eq(false)).to_sql(paquets::orm::Dialect::Postgres)` le renvoie sous forme de chaîne, sans exécuter la requête.

## Sécurité

### Pourquoi est-ce que j'obtiens 403 Forbidden ?

Le bouclier a bloqué la requête : le champ `code` de la réponse dit pourquoi (`waf_blocked`, `csrf_rejected`, `bot_blocked`…). Un formulaire venu d'un autre site, un motif d'attaque dans l'URL ou le user agent d'un outil d'attaque sont refusés avant que votre code ne s'exécute. Voir [Sécurité](security.md#à-quoi-ressemble-une-requête-bloquée).

### Pourquoi est-ce que j'obtiens 429 Too Many Requests ?

La limitation de débit autorise 20 requêtes par seconde et par adresse IP, avec des pointes de 100. Trois cas fréquents :

- **dans les tests**, toutes les requêtes viennent du même client : retirez la limitation de débit dans les tests qui envoient beaucoup de requêtes (voir [Tests](testing.md#tester-la-sécurité)) ;
- **dans un test de charge** : même chose, voir [Performances](performance.md#mesurer) ;
- **derrière un proxy** autre que celui de Heroku, tous les visiteurs partagent l'adresse du proxy : indiquez au bouclier où se trouve la véritable adresse (voir [Sécurité](security.md#lip-du-client-derrière-un-proxy)).

### Ma page ne peut pas charger une police ou un script d'un autre site.

La Content-Security-Policy de la page n'autorise que vos propres fichiers. Téléchargez la police ou le script dans `public/` (c'est mieux pour la vitesse et la vie privée), ou ajoutez la source à la politique de la page. Voir [Pages](pages.md#la-content-security-policy).

### HTTPS est-il pris en charge ? Et HTTP/2 ?

Oui. Sur Heroku, HTTPS vient du routeur placé devant l'application, et rien n'a besoin de changer. Pour servir HTTPS depuis l'application elle-même, sur votre propre serveur, activez la feature `tls` et donnez-lui un certificat avec `app.tls(...)` ; ajoutez `http2` pour HTTP/2, et `http3` pour HTTP/3 sur QUIC (voir [Configuration](configuration.md#https-dans-le-serveur)). Le bouclier maintient ensuite les navigateurs sur HTTPS grâce à l'en-tête `Strict-Transport-Security`.

### Puis-je désactiver la sécurité ?

Oui, avec `app.security(None)`, mais ne le faites que si une autre couche protège l'application. Pour assouplir une seule règle, modifiez plutôt le bouclier : `paquets::shield().without("bot_blocker").build()`. Voir [Sécurité](security.md#modifier-le-bouclier).

## Déploiement

### Peut-on vraiment déployer depuis un téléphone ?

Oui : le bouton Deploy to Heroku, l'add-on de base de données, les déploiements automatiques, les modifications du code, les journaux et les domaines personnalisés fonctionnent tous depuis le navigateur d'un téléphone et l'application GitHub. Seule la création d'un nouveau projet avec `paquets new` demande Rust, une seule fois, sur un ordinateur ou dans un environnement de développement dans le cloud. Voir [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md).

### Est-ce que ça fonctionne chez d'autres hébergeurs que Heroku ?

Oui : le `Dockerfile` tourne sur Render, Fly.io, Google Cloud Run, Railway ou votre propre serveur. Vérifiez le réglage de l'IP du client pour votre plateforme. Voir [Docker](deploy-docker.md#autres-plateformes).

### Pourquoi la première compilation est-elle si lente ?

Rust compile chaque dépendance une fois, avec toutes les optimisations en mode release, et la première compilation d'un projet avec des îlots installe aussi `wasm-bindgen-cli`. Les compilations suivantes ne recompilent que votre code. Sur Heroku, la première construction prend souvent 10 à 20 minutes.

## Le projet

### Comment mettre Paquets à jour ?

Dans un projet : `cargo update -p paquets` (les quatre paquets suivent). Pour la ligne de commande : `cargo install --git https://github.com/maxlestage/Paquets paquets --locked --force` (ou la même commande avec `--git`). Lisez d'abord le journal des modifications : avant la 1.0, les versions mineures peuvent changer l'API.

### Comment signaler un bug ou contribuer ?

Ouvrez une issue sur GitHub. Les pull requests sont les bienvenues ; lisez d'abord `CONTRIBUTING.md`. Un bug dans l'un des quatre paquets se signale de préférence dans son propre dépôt (voir [Les quatre paquets](packages.md)).

### Quelle licence ?

Paquets, Vitesse, Securite, memoire-orm et Active sont des logiciels propriétaires : © Maxime Nathan Lestage, tous droits réservés. Leur code source est publié sur crates.io et GitHub pour que vous puissiez le lire, l'auditer et le compiler, et vous pouvez les utiliser librement comme dépendances de vos propres programmes, y compris commerciaux. Les modifier ou redistribuer leur code source demande l'accord écrit de l'auteur. Les conditions complètes sont dans le fichier [LICENSE](https://maxlestage.github.io/Paquets/LICENSE.txt) de chaque dépôt. Ce que vous construisez avec eux, y compris les projets créés par `paquets new`, vous appartient.

## Étapes suivantes

- [Démarrage rapide](quick-start.md) : construire votre première application.
- [Aide-mémoire](cheatsheet.md) : toute l'API sur une seule page.
