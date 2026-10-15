# Documentation de Paquets

Tout ce qu'il faut pour créer des applications web full-stack avec Paquets 📦, le framework full-stack le plus simple pour Rust : de votre première page jusqu'à un déploiement fait depuis votre téléphone. Vous pouvez aussi lire ces pages sur le site : https://paquets-90dfc06b6fea.herokuapp.com/

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
- [Authentification](authentication.md) : inscription, connexion, mot de passe oublié et codes à deux facteurs en une ligne.
- [Panneau d'administration](admin.md) : lister, chercher, créer, modifier et supprimer les lignes de vos modèles dans le navigateur.
- [Générateurs](generators.md) : `paquets make:model`, `make:api`, `make:page`… écrivent le code et le branchent.
- [Exploitation](operations.md) : OpenAPI et Swagger UI, liste des routes, migrations et données d'exemple, santé, métriques, rechargement automatique.

## Déployer

- [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md) : le bouton Deploy, une base de données, les déploiements automatiques, les journaux et les domaines, le tout sans ordinateur.
- [Déployer avec Docker](deploy-docker.md) : l'image de production, les volumes, docker compose et les autres plateformes.
- [Publier sur crates.io](publish-crates-io.md) : l'ordre de publication des paquets, et les releases depuis un téléphone.
- [Automatisation](automation.md) : les tests à chaque push, la fusion automatique dans `master`, les déploiements, et comment les désactiver.

## Référence

- [Ligne de commande](cli.md) : `paquets new`, `dev`, `build`, `doctor`, `routes`, `db:*` et les générateurs.
- [Aide-mémoire](cheatsheet.md) : toute l'API sur une seule page.
- [Les quatre paquets](packages.md) : Vitesse, Securite, memoire-orm et Active, et comment Paquets s'en sert.
- [FAQ](faq.md) : les questions fréquentes et leurs réponses.

## Autres ressources

- Référence de l'API générée à partir du code source : https://docs.rs/paquets
- Code source, issues et exemples : https://paquets-90dfc06b6fea.herokuapp.com/
- Les quatre paquets : [Vitesse](https://vitesse-f16135654c3d.herokuapp.com/), [Securite](https://securite-2db4c1994be7.herokuapp.com/), [memoire-orm](https://memoire-7f4525f1fdaf.herokuapp.com/), [Active](https://active-c1883a966650.herokuapp.com/)
- Aussi disponible en [anglais](../en/README.md) et en [espagnol](../es/README.md).
