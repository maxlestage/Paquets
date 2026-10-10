# Automatisation

Le dépôt de Paquets, comme chaque projet créé avec `paquets new`, arrive avec des workflows GitHub qui font le travail répétitif : ils testent chaque push, **fusionnent automatiquement chaque branche verte dans `master`**, puis déploient `master` sur Heroku (et, pour le dépôt de Paquets, publient le site sur GitHub Pages). Cette page explique chaque workflow, les règles de la fusion automatique, et comment désactiver tout ou partie de cette mécanique.

## Vue d'ensemble

```text
push d'une branche ──▶ CI : formatage, Clippy, tests…
                         │
                         ├── rouge ──▶ rien d'autre ne se passe : corrigez la branche et poussez à nouveau
                         │
                         └── vert  ──▶ la branche est fusionnée dans master (sauf si elle s'appelle wip/…)
                                         │
                                         ├──▶ Deploy to Heroku   (si HEROKU_API_KEY et HEROKU_APP_NAME sont définis)
                                         └──▶ Pages              (dépôt de Paquets : le site)
```

Rien n'atteint `master`, ni la production, sans passer les tests. Et vous n'avez jamais à ouvrir, relire ou fusionner vous-même une pull request : c'est ce qui rend tout le cycle possible depuis un téléphone.

## Les workflows

| Fichier | Nom dans l'onglet Actions | Se lance | Fait |
|---|---|---|---|
| `.github/workflows/ci.yml` | **CI** | à chaque push et à chaque pull request, et à la main | vérifie tout, puis fusionne la branche dans `master` |
| `.github/workflows/heroku.yml` | **Deploy to Heroku** | après chaque fusion automatique, à chaque push sur `master`, à la main | déploie `master` sur Heroku (voir [Heroku](deploy-heroku-mobile.md#4-déploiements-automatiques-avec-github-actions)) |
| `.github/workflows/pages.yml` | **Pages** | après chaque fusion automatique, à chaque push sur `master` qui modifie le site ou la documentation, à la main | publie le site sur GitHub Pages (dépôt de Paquets uniquement) |
| `.github/workflows/release.yml` | **Release** | à la publication d'une release GitHub, à la main | publie les crates sur crates.io (dépôt de Paquets uniquement, voir [Publier sur crates.io](publish-crates-io.md)) |

Un projet créé avec `paquets new` a les deux premiers : `ci.yml` et `heroku.yml`.

## CI : ce qui est vérifié

### Dans un projet créé avec paquets new

Un job `test`, puis la fusion :

| Étape | Commande |
|---|---|
| formatage | `cargo fmt --check` |
| lints, serveur | `cargo clippy --all-targets -- -D warnings` |
| lints, îlots (si `src/lib.rs` existe) | `cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings` |
| tests | `cargo test` |

### Dans le dépôt de Paquets

Quatre jobs tournent en parallèle, et la fusion les attend tous :

| Job | Vérifie |
|---|---|
| **Crate** | le formatage ; Clippy sur le serveur avec les features par défaut et avec toutes les features ; Clippy côté navigateur (`wasm32`, feature `client`) ; les tests avec les features par défaut et avec toutes les features ; la documentation de l'API ; les paquets crates.io (`paquets-macros` en entier, et la liste des fichiers de `paquets`) |
| **MSRV** | la crate compile avec Rust 1.88, la plus ancienne version prise en charge |
| **paquets new** | génère un projet complet et un projet minimal avec la ligne de commande, et lance leurs propres vérifications (formatage, Clippy, tests, et le Clippy des îlots pour wasm32) |
| **Website** | le site (une application Paquets dans `site/`) : formatage, Clippy (serveur et WebAssembly), ses tests (y compris chaque page de la documentation, dans les trois langues, avec leurs liens), la compilation de ses îlots, et l'export statique pour GitHub Pages |

La moindre vérification au rouge arrête tout : la branche n'est pas fusionnée.

Pour lancer les principales vérifications sur votre ordinateur avant de pousser :

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings   # projets avec des îlots
cargo test
```

## La fusion automatique

Quand toutes les vérifications d'un push sont vertes, le job **Merge into master** fusionne la branche dans `master` :

- uniquement pour les **push** (pas pour les pull requests venant de forks), sur n'importe quelle branche sauf `master` elle-même ;
- **sauf pour les branches dont le nom commence par `wip/`** (`wip/nouveau-design`, `wip/experience`) : elles sont testées, jamais fusionnées automatiquement. Utilisez-les pour le travail en cours ;
- avec un commit de fusion, `Merge branch '<branche>' into master (auto-merge)`, créé par `github-actions[bot]` ;
- si la branche est déjà dans `master`, rien ne se passe ;
- les fusions ne s'exécutent jamais en même temps : elles s'attendent les unes les autres.

Après la fusion, le job lance lui-même les workflows de déploiement (`gh workflow run heroku.yml`, et `pages.yml` dans le dépôt de Paquets). C'est nécessaire, car GitHub ne lance pas de workflows pour un push effectué par un workflow.

### Quand la fusion échoue

Si `master` a changé d'une façon qui entre en conflit avec votre branche, le job de fusion échoue et `master` reste intacte. Récupérez `master` dans votre branche, résolvez les conflits, et poussez : la CI se relance et fusionne. Depuis un téléphone, le site de GitHub sait le faire dans une pull request (**Update branch**, puis **Resolve conflicts**).

Si le push sur `master` est refusé (`403` ou « protected branch »), voir [Protection de branche](#protection-de-branche) plus bas.

## GitHub Pages : le site

Le site de Paquets (dans `site/`) est une application Paquets : des pages rendues par le serveur avec active, des îlots en WebAssembly, et une démo en direct. Pour GitHub Pages, qui ne sert que des fichiers, le workflow **Pages** compile les îlots, puis exporte chaque page en fichiers statiques avec le propre code serveur du site :

```sh
cd site
BASE_PATH=/Paquets cargo run --release -- export dist
```

`BASE_PATH` préfixe chaque lien par le chemin du dépôt, car GitHub Pages sert le site à l'adresse `https://<propriétaire>.github.io/<dépôt>/`. Le workflow envoie `site/dist` et le publie.

À faire une fois, possible depuis un téléphone : **Settings** → **Pages** → **Build and deployment** → **Source** : **GitHub Actions**. D'ici là, le workflow est sauté avec un avertissement au lieu d'échouer. Pour publier tout de suite : **Actions** → **Pages** → **Run workflow**.

Ce même site, avec sa démo en direct et son API REST, est ce que déploie le bouton **Deploy to Heroku** (voir [Heroku](deploy-heroku-mobile.md)).

## Pas de Dependabot

Les dépôts n'ont pas de configuration Dependabot : aucun robot n'ouvre de pull requests pour mettre à jour les dépendances. Comme chaque branche verte est fusionnée et déployée automatiquement, la mise à jour d'un robot atteindrait la production sans que personne ne l'ait décidé. Les dépendances ne changent que lorsque quelqu'un lance `cargo update` et pousse le nouveau `Cargo.lock` sur une branche, que la CI teste comme n'importe quelle autre modification.

Pour repérer de temps en temps les dépendances obsolètes, lancez `cargo update --dry-run` (ou [`cargo outdated`](https://crates.io/crates/cargo-outdated)) sur votre ordinateur.

## Permissions

Le job de fusion demande deux permissions dans le fichier du workflow : `contents: write` (pour pousser sur `master`) et `actions: write` (pour lancer les déploiements). Elles fonctionnent avec les réglages par défaut de GitHub. Si votre organisation restreint les permissions des workflows, vérifiez **Settings** → **Actions** → **General** → **Workflow permissions**.

### Protection de branche

Si `master` est protégée par une règle qui exige des pull requests ou des relectures, le push du robot est refusé. Autorisez GitHub Actions à contourner la règle, ou désactivez la fusion automatique (voir plus bas) et fusionnez vous-même les pull requests.

## Désactiver tout ou partie

| Pour arrêter… | Faites ceci |
|---|---|
| la fusion d'une branche | nommez-la `wip/...` |
| toute la fusion automatique | supprimez le job `merge:` à la fin de `.github/workflows/ci.yml` ; les tests continuent de tourner |
| un workflow entier | **Actions** → le workflow → **⋯** → **Disable workflow** (site de GitHub) |
| les déploiements sur Heroku | supprimez le secret `HEROKU_API_KEY` ou la variable `HEROKU_APP_NAME` : le workflow est alors sauté avec un avertissement |
| le site GitHub Pages | **Settings** → **Pages**, dépubliez le site, ou désactivez le workflow **Pages** |
| la publication sur crates.io | ne publiez pas de releases, ou supprimez le secret `CARGO_REGISTRY_TOKEN` |

Les workflows désactivés se réactivent depuis le même menu.

## Étapes suivantes

- [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md) : le secret et la variable des déploiements.
- [Publier sur crates.io](publish-crates-io.md) : le workflow de publication.
- [Tests](testing.md) : écrire les tests que lance la CI.
