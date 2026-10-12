# Publier sur crates.io

[crates.io](https://crates.io) est le registre de paquets de Rust : une fois Paquets publié dessus, tout le monde peut installer la commande avec `cargo install --git https://github.com/maxlestage/Paquets paquets` et utiliser la bibliothèque avec `paquets = { git = "https://github.com/maxlestage/Paquets" }`. Cette page s'adresse aux mainteneurs de Paquets (ou d'un fork sous un autre nom). Elle explique pourquoi les quatre paquets doivent être publiés d'abord, puis comment tout publier depuis un téléphone, avec les releases GitHub.

## Pourquoi l'ordre compte

crates.io n'accepte que les paquets dont les dépendances sont **elles aussi sur crates.io** : un paquet publié ne peut pas dépendre d'un dépôt Git. Paquets dépend de ses quatre paquets : ils doivent donc être publiés avant lui.

C'est pourquoi, tant qu'ils ne sont pas tous publiés, le `Cargo.toml` de Paquets donne à chaque paquet à la fois une `version` et la source `git` de son dépôt, par exemple :

```toml
[dependencies]
active = { git = "https://github.com/maxlestage/Active" }
memoire-orm = { git = "https://github.com/maxlestage/Memoire", default-features = false, features = ["memory"] }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
vitesse = { git = "https://github.com/maxlestage/Vitesse" }
securite = { git = "https://github.com/maxlestage/Securite", default-features = false }
```

- Pendant le développement, Cargo utilise la source `git` : le code le plus récent de chaque dépôt.
- À la publication, `cargo publish` retire la source `git` et garde la `version` : le `paquets` publié dépend de `vitesse 0.1` **venant de crates.io**. Si cette version n'y est pas, crates.io refuse l'envoi.

## L'ordre de publication

Sept crates sont publiées depuis cinq dépôts. Chaque dépôt a son propre workflow de publication, qui se lance de la même façon : en publiant une release GitHub.

| Ordre | Crate | Dépôt | Workflow (onglet Actions) |
|---|---|---|---|
| 1 | `memoire-orm-derive` | [Memoire](https://maxlestage.github.io/Memoire/) | **Publish to crates.io** (publie les deux crates de Memoire, la derive en premier) |
| 2 | `memoire-orm` | [Memoire](https://maxlestage.github.io/Memoire/) | la même exécution |
| 3 | `securite` | [Securite](https://maxlestage.github.io/Securite/) | **Publish crate** |
| 4 | `vitesse` | [Vitesse](https://maxlestage.github.io/Vitesse/) | **Release** |
| 5 | `active` | [Active](https://maxlestage.github.io/Active/) | **Release** |
| 6 | `paquets-macros`, puis `paquets` | [Paquets](https://maxlestage.github.io/Paquets/) | **Release** |

`memoire-orm` a besoin de `memoire-orm-derive`, et le workflow de Memoire les publie dans cet ordre. Les quatre dépôts des étapes 1 à 5 ne dépendent pas les uns des autres : vous pouvez les publier dans n'importe quel ordre, du moment qu'ils sont tous sur crates.io avant l'étape 6. Le workflow de Paquets le vérifie, et indique les crates qui manquent.

Chaque workflow saute une crate dont la version est déjà sur crates.io : relancer un workflow est donc sans risque.

## À faire une fois : le jeton crates.io

Les workflows publient avec un jeton d'API crates.io, enregistré comme secret GitHub nommé `CARGO_REGISTRY_TOKEN` dans **chacun** des cinq dépôts.

### Créer le jeton

1. Dans le navigateur de votre téléphone, ouvrez https://crates.io et touchez **Log in with GitHub**.
2. Ouvrez **Account Settings** et vérifiez que votre adresse e-mail est confirmée : sinon, crates.io refuse de publier.
3. Ouvrez **API Tokens** et touchez **New Token**.
4. Donnez-lui un nom, par exemple « GitHub Actions », et éventuellement une date d'expiration.
5. Sélectionnez les droits **publish-new** (nécessaire la première fois qu'un nom de crate est publié) et **publish-update** (nécessaire pour les nouvelles versions).
6. Touchez **Generate Token** et copiez-le : crates.io ne l'affiche qu'une seule fois.

### L'enregistrer dans chaque dépôt

Pour chaque dépôt (Memoire, Securite, Vitesse, Active, Paquets) :

1. Sur le site de GitHub, ouvrez le dépôt, puis **Settings** → **Secrets and variables** → **Actions**.
2. Touchez **New repository secret**.
3. **Name** : `CARGO_REGISTRY_TOKEN`. **Secret** : le jeton. Touchez **Add secret**.

> [!WARNING]
> Ce jeton peut publier des crates en votre nom. Ne le gardez que dans les secrets de GitHub : jamais dans un fichier, un commit ou un message. S'il fuit, révoquez-le sur la page **API Tokens** de crates.io et créez-en un nouveau.

## Publier un paquet depuis votre téléphone

Les étapes sont les mêmes dans tous les dépôts.

1. Vérifiez que la version de `Cargo.toml` est bien celle que vous voulez publier (`0.1.0` pour une première release), et que la CI est verte sur `master`.
2. Ouvrez la page **Releases** du dépôt (`https://github.com/<propriétaire>/<dépôt>/releases`) et touchez **Draft a new release**.
3. Touchez **Choose a tag**, tapez la version précédée d'un `v`, par exemple `v0.1.0`, et sélectionnez **Create new tag: v0.1.0 on publish**. Gardez `master` comme cible.
4. Donnez un titre à la release (par exemple `v0.1.0`) et des notes ; **Generate release notes** les remplit à partir des modifications fusionnées.
5. Touchez **Publish release**.

Le workflow démarre en quelques secondes. Suivez-le dans l'onglet **Actions** (l'application GitHub fonctionne). Quand il passe au vert, la crate est sur crates.io, et docs.rs construit sa documentation quelques minutes plus tard.

> [!NOTE]
> Le tag doit correspondre à la version de `Cargo.toml` : le tag `v0.1.0` pour la version `0.1.0`. Sinon, les workflows de Paquets, Vitesse, Securite et Memoire s'arrêtent avec une erreur ; celui d'Active prend la version dans le tag.

## Publier Paquets

Une fois les quatre paquets sur crates.io, publiez Paquets de la même façon (tag `v0.1.1`). Son workflow **Release** :

1. vérifie que le tag correspond à la version de `Cargo.toml` ;
2. vérifie que `memoire-orm-derive`, `memoire-orm`, `securite`, `vitesse` et `active` sont sur crates.io, et sinon s'arrête avec la liste de ceux qui manquent ;
3. lance les tests ;
4. publie `paquets-macros` (sauf si cette version y est déjà), puis `paquets` ;
5. écrit un lien vers la crate dans le résumé de l'exécution.

Pour tout vérifier sans publier, lancez-le à la main : **Actions** → **Release** → **Run workflow**, en laissant **Only check** coché. Il fait alors un essai à blanc (dry run) au lieu de la publication.

Ensuite :

- `cargo install --git https://github.com/maxlestage/Paquets paquets --locked` installe la commande ;
- `paquets = { git = "https://github.com/maxlestage/Paquets" }` fonctionne dans n'importe quel projet. Les projets créés avec `paquets new` continuent d'utiliser le dépôt GitHub (`version = "0.1", git = ...`) jusqu'à ce que vous simplifiiez leur `Cargo.toml` en `paquets = { git = "https://github.com/maxlestage/Paquets" }`.

## Une nouvelle version

crates.io n'accepte jamais deux fois la même version : chaque release demande un nouveau numéro de version. Suivez le [versionnage sémantique](https://semver.org) : `0.1.0` → `0.1.1` pour les corrections, `0.1.1` → `0.1.1` pour les nouvelles fonctionnalités ou, avant la 1.0, les changements incompatibles.

Dans le dépôt de Paquets, quatre endroits changent :

| Fichier | Modification |
|---|---|
| `Cargo.toml` | `version = "0.1.1"` dans `[package]` |
| `macros/Cargo.toml` | la même `version = "0.1.1"` |
| `Cargo.toml` | la version épinglée des macros : `paquets-macros = { version = "=0.1.1", path = "macros" }` |
| `CHANGELOG.md` | une section pour la nouvelle version |

Depuis un téléphone, modifiez chaque fichier sur le site de GitHub (icône en forme de crayon), sur une nouvelle branche : la CI la vérifie et la fusionne dans `master`. Publiez ensuite une release avec le nouveau tag (`v0.1.1`). Si la nouvelle version de Paquets a besoin d'une nouvelle version de l'un des paquets, publiez d'abord ce paquet.

## En cas de problème

| Message | Cause | Solution |
|---|---|---|
| `Publish these packages on crates.io first…` | l'un des quatre paquets n'est pas encore publié | publiez-le depuis son dépôt, puis relancez la release de Paquets |
| `no matching package named 'vitesse' found` (ou un autre paquet) | la même chose, vue par `cargo publish` | la même solution |
| `The tag (…) does not match the version of Cargo.toml` | le tag et la version diffèrent | supprimez la release, corrigez la version ou le tag, publiez à nouveau |
| `Add the CARGO_REGISTRY_TOKEN secret` ou `401`/`403` de la part de crates.io | le jeton est absent, faux, expiré, ou il lui manque un droit | créez un nouveau jeton avec publish-new et publish-update, et mettez à jour le secret |
| `crate version … is already uploaded` | cette version est déjà sur crates.io | augmentez le numéro de version |
| `the remote server responded with an error: … verified email` | votre e-mail crates.io n'est pas confirmé | confirmez-le dans les Account Settings de crates.io |
| `crate name is already taken` | quelqu'un d'autre possède ce nom sur crates.io | dans un fork, publiez sous un autre nom |

> [!WARNING]
> Une version publiée ne peut jamais être modifiée ni supprimée. Si une release est cassée, publiez une version corrigée, puis retirez (*yank*) la version cassée avec `cargo yank paquets --version 0.1.1` (depuis un ordinateur) : les projets qui l'utilisent déjà continuent de fonctionner, les nouveaux projets ne la sélectionneront pas.

## Étapes suivantes

- [Automatisation](automation.md) : la CI qui vérifie les paquets avant chaque release.
- [Les quatre paquets](packages.md) : ce que fait chaque paquet.
- [Installation](installation.md) : ce que font les utilisateurs une fois Paquets sur crates.io.
