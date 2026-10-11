# Documentation de Paquets

Tout ce qu'il faut pour créer des applications web full-stack avec Paquets 📦, le framework full-stack le plus simple pour Rust : de votre première page jusqu'à un déploiement fait depuis votre téléphone. Vous pouvez aussi lire ces pages sur le site : https://maxlestage.github.io/Paquets/

## Démarrer

- [Introduction](introduction.md) : ce qu'est Paquets, ce qu'il fait pour vous, et ses limites.
- [Installation](installation.md) : Rust, la commande `paquets` et les outils WebAssembly facultatifs.
- [Démarrage rapide](quick-start.md) : construire pas à pas une petite application avec des pages, un formulaire, une base de données et un test.
- [Structure d'un projet](project-structure.md) : chaque fichier créé par `paquets new`, et comment organiser un projet qui grandit.

## L'essentiel

- [Routes](routing.md) : routes, paramètres, routeurs, middlewares, la page 404 et les fichiers statiques.
- [Requêtes et réponses](requests-responses.md) : query string, en-têtes, JSON et formulaires ; texte, JSON, redirections, cookies et fichiers.
- [Pages](pages.md) : vues, mises en page, le thème intégré, les balises SEO et la Content-Security-Policy.
- [Base de données](database.md) : modèles, choix de la base, requêtes, transactions, migrations et SQL brut.
- [API REST](rest-api.md) : une API JSON complète pour un modèle en une ligne, et comment la protéger ou l'étendre.
- [Sécurité](security.md) : le bouclier devant chaque requête, comment le modifier, et l'authentification.

## Aller plus loin

- [Îlots](islands.md) : les parties interactives d'une page, écrites en Rust et exécutées dans le navigateur en WebAssembly.
- [WebSocket](websocket.md) : des connexions bidirectionnelles, du serveur d'écho au salon de discussion.
- [Tests](testing.md) : tester toute l'application en mémoire avec `app.test()`, sans ouvrir de port.
- [Erreurs](errors.md) : `Result`, l'opérateur `?`, le statut de chaque erreur, et les formats d'erreur sur mesure.
- [Configuration](configuration.md) : variables d'environnement, réglages de l'application, état partagé et fonctionnement du serveur.
- [Performances](performance.md) : les benchmarks, la façon dont le serveur utilise votre processeur, et des conseils pour rester rapide.

## Déployer

- [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md) : le bouton Deploy, une base de données, les déploiements automatiques, les journaux et les domaines, le tout sans ordinateur.
- [Déployer avec Docker](deploy-docker.md) : l'image de production, les volumes, docker compose et les autres plateformes.
- [Publier sur crates.io](publish-crates-io.md) : l'ordre de publication des paquets, et les releases depuis un téléphone.
- [Automatisation](automation.md) : les tests à chaque push, la fusion automatique dans `master`, les déploiements, et comment les désactiver.

## Référence

- [Ligne de commande](cli.md) : `paquets new`, `dev`, `build` et `doctor`.
- [Aide-mémoire](cheatsheet.md) : toute l'API sur une seule page.
- [Les quatre paquets](packages.md) : Vitesse, Securite, memoire-orm et Active, et comment Paquets s'en sert.
- [FAQ](faq.md) : les questions fréquentes et leurs réponses.

## Autres ressources

- Référence de l'API générée à partir du code source : https://docs.rs/paquets
- Code source, issues et exemples : https://maxlestage.github.io/Paquets/
- Les quatre paquets : [Vitesse](https://maxlestage.github.io/Vitesse/), [Securite](https://maxlestage.github.io/Securite/), [memoire-orm](https://maxlestage.github.io/Memoire/), [Active](https://maxlestage.github.io/Active/)
- Aussi disponible en [anglais](../en/README.md) et en [espagnol](../es/README.md).
