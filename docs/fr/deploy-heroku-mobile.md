# Déployer sur Heroku depuis votre téléphone

Pas besoin d'ordinateur pour mettre une application Paquets en ligne. Ce guide passe en revue chaque étape depuis un téléphone : un déploiement en un geste avec le bouton **Deploy to Heroku**, une base de données PostgreSQL, des déploiements automatiques à chaque modification avec GitHub Actions, puis le quotidien : redéployer, lire les journaux, ajouter un nom de domaine, régler les problèmes. Les mêmes étapes valent pour le dépôt de Paquets et pour votre propre projet créé avec `paquets new`.

> [!NOTE]
> Toutes les étapes fonctionnent dans le navigateur web d'un téléphone (Safari, Chrome, Firefox…). L'application GitHub est pratique pour les onglets **Actions** et **Releases**, mais quelques réglages (secrets, variables, modification de fichiers) ne se trouvent que sur le site de GitHub : ouvrez https://github.com dans votre navigateur pour ceux-là. Si un menu semble manquer sur github.com, passez votre navigateur en mode « Version pour ordinateur ». Les interfaces de Heroku et de GitHub sont en anglais, d'où les libellés en anglais ci-dessous, et elles renomment parfois leurs menus : si un libellé diffère légèrement, cherchez le plus proche.

## Avant de commencer

Il vous faut :

- **Un compte Heroku** : inscrivez-vous sur https://signup.heroku.com. Heroku n'a **pas d'offre gratuite** : les dynos (les conteneurs qui font tourner votre application) les moins chers sont les **Eco**, environ **5 $ par mois**, et les **Basic**, environ 7 $ par mois. Heroku demande un moyen de paiement avant de pouvoir créer une application. Les tarifs évoluent : vérifiez sur https://www.heroku.com/pricing.
- **Un compte GitHub**, pour votre propre copie du code et pour les déploiements automatiques (parties 3 et 4).
- Éventuellement, quelques dollars de plus par mois pour une base de données **Heroku Postgres** (partie 2).

### Comment ça marche

Tout ce dont Heroku a besoin se trouve déjà dans le dépôt, rien n'est donc compilé sur votre téléphone : Heroku construit l'application sur ses propres serveurs.

Dans le dépôt de Paquets, Heroku construit le site de Paquets, qui est lui-même une application Paquets (pages rendues par le serveur, îlots en WebAssembly), avec une démo en direct sur `/demo` et une API REST sur `/api/todos`. Par défaut, il utilise le [buildpack **emk/rust**](https://elements.heroku.com/buildpacks/emk/heroku-buildpack-rust) sur la pile **heroku-26** :

| Fichier | Rôle |
|---|---|
| `app.json` | décrit l'application pour le bouton Deploy (nom, description, la pile heroku-26, le buildpack emk/rust) |
| `RustConfig` | configure le buildpack : la version de Rust et, au lieu de sa propre compilation, `heroku-build.sh` |
| `heroku-build.sh` | la compilation, lancée une fois Rust installé par le buildpack : compile le serveur (`paquets-site`) en mode release, dans `site/`, puis les îlots (WebAssembly) dans `site/pkg` |
| `Procfile` | la commande qui démarre le site |
| `heroku.yml` et `Dockerfile` | l'autre voie : une image Docker, sur la pile « container » (`HEROKU_BUILD=docker`, voir la partie 4) |
| `.github/workflows/heroku.yml` | le workflow **Deploy to Heroku**, pour les déploiements automatiques |
| `.github/workflows/ci.yml` | le workflow **CI** : teste chaque push, puis fusionne la branche dans `master` |

Un projet créé avec `paquets new` est construit à partir de son `Dockerfile` : son `app.json` choisit la pile « container », et son `heroku.yml` indique à Heroku de construire l'image à partir du `Dockerfile`, qui compile votre application (`src/main.rs` et les îlots de `src/lib.rs`) en mode release et l'emballe dans une petite image.

## 1. Le plus rapide : le bouton Deploy to Heroku

1. Touchez le bouton ci-dessus (il figure aussi dans le README du dépôt), ou ouvrez cette adresse : `https://dashboard.heroku.com/new-app`. Connectez-vous à Heroku si on vous le demande.
2. Heroku affiche un formulaire de création d'application :
   - **App name** : par exemple `mes-paquets`. Les noms sont uniques sur tout Heroku, en lettres minuscules, chiffres et tirets. Vous pouvez laisser le champ vide : Heroku en choisira un.
   - **Choose a region** : États-Unis ou Europe.
3. Touchez **Deploy app**. Heroku récupère le code et construit l'application (avec le buildpack dans le dépôt de Paquets, à partir du `Dockerfile` dans un projet créé avec `paquets new`) ; le journal de construction défile sur la page. La première construction prend un moment (souvent 10 à 20 minutes) : Rust compile l'application et son bundle WebAssembly avec toutes les optimisations activées.
4. Une fois la construction terminée, touchez **View**. Le site s'ouvre à l'adresse `https://<nom-de-l-app>.herokuapp.com` (l'adresse exacte figure aussi dans l'onglet **Settings** de l'application sur Heroku).

Essayez la démo en ajoutant ces chemins à l'adresse :

| Chemin | Ce que vous voyez |
|---|---|
| `/demo` | une liste de tâches rendue par le serveur, avec un formulaire |
| `/api/todos` | les mêmes tâches, en JSON |

> [!NOTE]
> Tant que vous n'ajoutez pas de base de données, l'application garde ses données en mémoire : elles sont perdues à chaque redémarrage (chaque déploiement, chaque changement de réglage, et le redémarrage automatique qu'effectue Heroku environ une fois par jour). La partie 2 règle ce problème.

## 2. Ajouter une base de données

1. Ouvrez https://dashboard.heroku.com et touchez votre application.
2. Ouvrez l'onglet **Resources**.
3. Dans **Add-ons**, cherchez **Heroku Postgres** et sélectionnez-le.
4. Choisissez une offre (la plus petite, Essential-0, coûte environ 5 $ par mois au moment où ces lignes sont écrites) et confirmez avec **Submit Order Form**.

C'est tout. Heroku crée la base de données, définit la config var `DATABASE_URL` et redémarre l'application. Paquets lit `DATABASE_URL` au démarrage, se connecte avec TLS et crée les tables de vos modèles si elles n'existent pas encore. Pour le vérifier, ouvrez les journaux (**More** → **View logs**) : la ligne de démarrage indique maintenant `database: postgres`.

```text
📦 Paquets · http://localhost:21345
   database: postgres · security: on (10 guards) · islands: /pkg/paquets_site.js
```

(L'adresse de cette ligne est celle de l'intérieur du conteneur : votre application est joignable à l'adresse `https://<nom-de-l-app>.herokuapp.com`.)

> [!WARNING]
> N'utilisez pas SQLite sur Heroku : le système de fichiers d'un dyno est temporaire, et le fichier serait effacé à chaque redémarrage.

## 3. Votre propre copie du code

Pour modifier le code, il vous faut votre propre copie du dépôt sur GitHub :

1. Ouvrez https://maxlestage.github.io/Paquets/ (l'application GitHub fonctionne), touchez **Fork**, puis **Create fork**.
2. Déployez votre fork en tapant cette adresse dans votre navigateur, avec votre nom d'utilisateur GitHub à la place de `<vous>` :

   ```text
   https://heroku.com/deploy?template=https://github.com/<vous>/Paquets
   ```

3. Suivez les étapes de la partie 1.

> [!IMPORTANT]
> Le bouton déploie **une seule fois**. Quand vous modifiez votre fork ensuite, l'application Heroku n'est pas mise à jour d'elle-même. Configurez les déploiements automatiques (partie 4) avec le **même nom d'application** : le workflow met à jour l'application que vous venez de créer.

## 4. Déploiements automatiques avec GitHub Actions

Objectif : chaque modification fusionnée dans `master` reconstruit et redéploie l'application, sans rien faire. Le workflow **Deploy to Heroku** dialogue directement avec l'API Platform de Heroku : ni CLI Heroku, ni ordinateur. À chaque exécution, il :

1. vérifie qu'il est configuré (sinon, il s'arrête sans échouer) ;
2. crée l'application Heroku si elle n'existe pas encore, et choisit comment Heroku la construit : dans le dépôt de Paquets, la pile heroku-26 et le buildpack emk/rust (ou la pile « container » avec `HEROKU_BUILD=docker`) ; dans un projet créé avec `paquets new`, la pile « container » ;
3. envoie le code à Heroku, qui le construit (avec le buildpack, ou l'image à partir de `heroku.yml`), et affiche le journal de construction en direct dans GitHub ;
4. écrit l'adresse de l'application dans le résumé de l'exécution.

### Étape 1 : obtenir une clé d'API Heroku

1. Dans votre navigateur, ouvrez https://dashboard.heroku.com et connectez-vous.
2. Ouvrez **Account settings** (depuis le menu ou votre avatar, en haut à droite).
3. Dans la section **API Key**, touchez **Reveal**, puis copiez la clé.

Pour une clé que vous pourrez révoquer séparément, ouvrez plutôt l'onglet **Applications** des Account settings, touchez **Create authorization** sous **Authorizations**, donnez-lui une description comme « GitHub Actions », et copiez le jeton.

> [!WARNING]
> Cette clé agit sur tout votre compte Heroku. Ne la collez jamais dans un fichier, un commit, une issue ou un message : uniquement dans les secrets de GitHub, comme décrit ci-dessous.

### Étape 2 : ajouter le secret et la variable sur GitHub

1. Sur https://github.com, ouvrez **votre dépôt** (votre fork, ou votre propre projet) et touchez **Settings** (dans la barre d'onglets du dépôt ; faites-la défiler sur le côté si besoin).
2. Ouvrez **Secrets and variables**, puis **Actions**.
3. Dans l'onglet **Secrets**, touchez **New repository secret** :
   - **Name** : `HEROKU_API_KEY`
   - **Secret** : collez la clé
   - touchez **Add secret**.
4. Dans l'onglet **Variables**, touchez **New repository variable** :
   - **Name** : `HEROKU_APP_NAME`
   - **Value** : le nom de votre application Heroku (celle créée avec le bouton, ou un nouveau nom : le workflow crée l'application si besoin)
   - touchez **Add variable**.
5. Facultatif : une variable `HEROKU_REGION` avec la valeur `eu` crée l'application en Europe. Elle ne sert que lorsque le workflow crée l'application.
6. Facultatif, dans le dépôt de Paquets : une variable `HEROKU_BUILD` avec la valeur `docker` construit l'image Docker (`heroku.yml` et le `Dockerfile`, pile « container ») au lieu du buildpack. Avec `buildpack`, ou sans la variable, le workflow utilise le buildpack emk/rust. Il change lui-même la pile de l'application quand vous changez cette valeur.

| Nom | Type | Obligatoire | Valeur |
|---|---|---|---|
| `HEROKU_API_KEY` | secret | oui | votre clé d'API Heroku |
| `HEROKU_APP_NAME` | variable | oui | par exemple `mes-paquets` |
| `HEROKU_REGION` | variable | non | `us` (par défaut) ou `eu` |
| `HEROKU_BUILD` | variable | non | `buildpack` (par défaut) ou `docker` (dépôt de Paquets) |

### Étape 3 : activer Actions dans un fork

GitHub désactive les workflows dans un nouveau fork. Ouvrez l'onglet **Actions** de votre fork : si un message indique que les workflows ne tournent pas sur ce fork, touchez le bouton qui les active. (Vos propres dépôts n'en ont pas besoin.)

### Étape 4 : lancer le premier déploiement

1. Ouvrez l'onglet **Actions** (dans l'application GitHub ou sur le site) et choisissez **Deploy to Heroku** dans la liste des workflows.
2. Touchez **Run workflow**, gardez la branche `master`, et confirmez avec **Run workflow**.
3. Touchez l'exécution qui apparaît pour la suivre. La première construction prend plusieurs minutes.
4. Quand elle passe au vert, le résumé de l'exécution affiche l'adresse de l'application : `🚀 Deployed to https://mes-paquets.herokuapp.com`.

Désormais, l'application est redéployée :

- après chaque branche que le workflow **CI** fusionne dans `master` (voir la partie 5 et [Automatisation](automation.md)) ;
- après chaque commit direct sur `master` (dans le dépôt de Paquets, un commit qui ne modifie que des fichiers Markdown ne le déclenche pas) ;
- chaque fois que vous lancez le workflow à la main.

> [!NOTE]
> Tant que le secret ou la variable manque, le workflow saute le déploiement avec un avertissement (« Heroku is not configured ») au lieu d'échouer.

### Sans GitHub Actions

Pour le dépôt de Paquets, une application créée autrement (ni par le bouton, ni par le workflow) doit utiliser le buildpack :

- **Tableau de bord Heroku** : votre application → **Settings** → **Buildpacks** → **Add buildpack**, tapez `emk/rust` et enregistrez. La pile de l'application ne doit pas être « container » (cette pile construit avec `heroku.yml` à la place).
- **CLI Heroku** (sur un ordinateur, ou dans un environnement dans le cloud comme GitHub Codespaces), depuis un clone du dépôt :

  ```sh
  heroku create mes-paquets --stack heroku-26 --buildpack emk/rust
  git push heroku master
  ```

  Pour une application existante : `heroku stack:set heroku-26`, puis `heroku buildpacks:set emk/rust`, puis `git push heroku master`.

## 5. Modifier le code depuis votre téléphone

1. Sur le site de GitHub, dans votre dépôt, ouvrez le fichier à modifier : pour votre propre projet, `src/main.rs`.
2. Touchez l'icône en forme de **crayon** (✏️ « Edit this file ») et faites votre modification. Par exemple, changez le texte de la page d'accueil :

   ```rust
   .child(h1().text("Bonjour depuis mon téléphone 👋"))
   ```

3. Touchez **Commit changes…**. Choisissez **Create a new branch for this commit**, donnez-lui un nom comme `nouveau-titre`, et confirmez (**Propose changes**). Inutile d'ouvrir la pull request.
4. Le workflow **CI** démarre (suivez-le dans l'onglet **Actions**) : formatage, Clippy, tests. Quand tout est vert, il fusionne de lui-même votre branche dans `master` et lance le déploiement. Quelques minutes plus tard, la nouvelle version est en ligne.

Si la CI échoue, ouvrez l'exécution en échec : le message du compilateur donne le fichier, la ligne et la colonne. Corrigez le fichier sur la même branche : la CI se relance. Rien n'atteint `master`, ni Heroku, tant que la CI n'est pas verte.

> [!TIP]
> Vous pouvez aussi faire un commit directement sur `master` : le déploiement démarre aussitôt, mais sans attendre les tests. Une branche est plus sûre. Une branche dont le nom commence par `wip/` est testée mais jamais fusionnée automatiquement, pour le travail en cours.

Heroku ne met en ligne que les constructions réussies : si une construction échoue, la version précédente reste en ligne.

## 6. Votre propre projet, créé avec paquets new

Un projet créé avec `paquets new mon-app` contient `app.json`, `heroku.yml`, un `Dockerfile` et les deux workflows, et il est construit à partir de son `Dockerfile` (pile « container »). Chaque étape de ce guide s'y applique, sauf le buildpack et `HEROKU_BUILD`, propres au dépôt de Paquets.

La création du projet demande Rust, une seule fois : sur n'importe quel ordinateur, ou dans un environnement de développement dans le cloud qui tourne dans un navigateur, comme GitHub Codespaces. Lancez `paquets new mon-app`, puis publiez le dossier comme nouveau dépôt GitHub (par exemple `gh repo create mon-app --private --source=mon-app --push`). À partir de là, tout se fait depuis votre téléphone :

- **Premier déploiement** : le bouton Deploy avec l'adresse de votre dépôt, `https://heroku.com/deploy?template=https://github.com/<vous>/mon-app` (pour un dépôt privé, passez plutôt par la partie 4 : le bouton a besoin d'un dépôt public).
- **Déploiements automatiques** : le secret `HEROKU_API_KEY` et la variable `HEROKU_APP_NAME`, comme dans la partie 4.
- **Une base de données** : l'add-on Heroku Postgres, comme dans la partie 2.

Le `README.md` du projet reprend ces étapes.

## 7. Au quotidien

### Redéployer à la main

**Actions** → **Deploy to Heroku** → **Run workflow** → **Run workflow**. Cela fonctionne dans l'application GitHub. Pratique après avoir changé une config var qui demande une nouvelle construction, ou pour retenter un déploiement.

### Lire les journaux

Tableau de bord Heroku → votre application → **More** (en haut à droite) → **View logs**. Vous y voyez les messages de Heroku lui-même (démarrages, arrêts, plantages, une ligne par requête de la part de son routeur) et tout ce qu'affiche votre application :

```text
app[web.1]: 📦 Paquets · http://localhost:21345
app[web.1]:    database: postgres · security: on (10 guards) · islands: /pkg/mon_app.js
app[web.1]: GET /api/todos 200 0.412 ms
heroku[router]: at=info method=GET path="/api/todos" host=mon-app.herokuapp.com status=200 ...
```

Les erreurs de vos handlers qui deviennent une `500` sont journalisées avec leur cause (`[vitesse] error 500: ...`). L'onglet **Activity** liste les constructions et les mises en ligne, avec un lien vers le journal de chaque construction.

### Settings : config vars

**Settings** → **Reveal Config Vars**. Chaque config var est une variable d'environnement, lue dans votre code avec `std::env::var("NOM")`. Enregistrer une modification redémarre l'application. `DATABASE_URL` est gérée par l'add-on Postgres. Ne définissez pas `PORT` : Heroku la définit à chaque démarrage.

### Redémarrage, mise en veille, type de dyno

- **Redémarrer** : **More** → **Restart all dynos**.
- **Mise en veille** : un dyno Eco s'endort après environ 30 minutes sans visite ; la visite suivante le réveille en quelques secondes. Un dyno Basic ne dort jamais.
- **Type de dyno** : dans l'onglet **Resources**, modifiez la ligne `web` (icône en forme de crayon) pour changer de type ; le prix change en conséquence.

### Un domaine personnalisé

1. **Settings** → **Domains** → **Add domain**, et tapez votre domaine, par exemple `www.example.com`.
2. Heroku vous donne une **cible DNS** (DNS target). Chez votre registraire de domaine, créez un enregistrement `CNAME` pour `www` qui pointe vers cette cible. Un domaine racine (`example.com`) demande un registraire qui prend en charge les enregistrements `ALIAS`/`ANAME` ou l'aplatissement de CNAME (CNAME flattening).
3. Pour avoir HTTPS sur votre domaine, activez l'**Automated Certificate Management** de Heroku dans le même onglet. Il n'est pas disponible sur tous les types de dynos : voir https://devcenter.heroku.com/articles/automated-certificate-management.

### Arrêter de payer

Supprimez les add-ons dans **Resources**, et l'application elle-même dans **Settings** → **Delete app**. Supprimer le secret `HEROKU_API_KEY` arrête les déploiements automatiques.

## 8. Dépannage

### Le workflow échoue avant la construction

| Message | Solution |
|---|---|
| `Heroku is not configured` (un avertissement) | Ajoutez le secret `HEROKU_API_KEY` et la variable `HEROKU_APP_NAME` (partie 4, étape 2). |
| `Heroku refused the API key (HEROKU_API_KEY).` | La clé est fausse, expirée ou révoquée : copiez-en une nouvelle et mettez à jour le secret. |
| `This Heroku account cannot use the app "…" (the name may be taken).` | Le nom appartient à un autre compte Heroku : choisissez un autre nom dans `HEROKU_APP_NAME`. |
| la création de l'application échoue | Vérifiez que votre compte Heroku a un moyen de paiement, ou créez d'abord l'application avec le bouton et mettez son nom dans `HEROKU_APP_NAME`. |

### La construction échoue

Ouvrez le journal de construction, dans GitHub (**Actions**, l'exécution en échec) ou sur Heroku (**Activity** → journal de construction), et cherchez la première ligne qui commence par `error` :

| Message | Cause | Solution |
|---|---|---|
| `error[E…]` avec un fichier et une ligne | une erreur de compilation Rust | corrigez le code à cette ligne et refaites un commit |
| `no matching package named …` | un nom de crate mal orthographié dans `Cargo.toml` | vérifiez le nom sur https://crates.io |
| `The Heroku build ended with status "failed".` | la construction a échoué sur Heroku | lisez le journal de construction juste au-dessus, dans la même exécution |

### « Application error » dans le navigateur

La construction a réussi, mais l'application ne répond pas. Ouvrez les journaux et cherchez un code d'erreur :

| Code | Signification | Que faire |
|---|---|---|
| `H10` App crashed | le programme s'est arrêté | lisez les lignes juste au-dessus : `Error: … database: …` signifie que `DATABASE_URL` est fausse ou que la base de données est injoignable ; `startup: …` signifie qu'un hook `on_start` a échoué ; `duplicate route` signifie que deux routes entrent en conflit |
| `R10` Boot timeout | l'application n'a pas écouté sur `$PORT` dans les 60 secondes | Paquets écoute sur `$PORT` de lui-même : n'appelez pas `app.port(...)` avec un port fixe en production |
| `H14` No web dynos running | aucun dyno ne fait tourner le processus `web` | dans **Resources**, vérifiez que le dyno `web` est allumé |
| `H12` Request timeout | une requête a pris plus de 30 secondes | accélérez le handler, ou sortez le travail de la requête |

Tous les codes sont listés sur https://devcenter.heroku.com/articles/error-codes.

### Autres surprises

- **Tous les visiteurs reçoivent `429 Too Many Requests`** : le bouclier compte les requêtes par adresse IP. Le bouclier par défaut lit la véritable adresse derrière le routeur de Heroku ; si vous l'avez remplacé avec `app.security(...)`, construisez-le à partir de `paquets::shield()`, qui conserve ce réglage (voir [Sécurité](security.md#lip-du-client-derrière-un-proxy)).
- **La première visite est lente** : un dyno Eco dormait.
- **Mes données ont disparu** : l'application n'a pas d'add-on de base de données (données en mémoire), ou utilise SQLite. Ajoutez Heroku Postgres (partie 2).
- **Un nouveau champ d'un modèle provoque des erreurs `500`** : la table existante n'a pas été modifiée. Voir [Base de données](database.md#modifier-une-table-plus-tard).
- **Les connexions WebSocket se ferment au bout d'une minute** : Heroku ferme les connexions qui restent silencieuses pendant environ 55 secondes. Envoyez régulièrement un ping (voir [WebSocket](websocket.md#en-production)).
- **HTTP/3 ne fonctionne pas** : le routeur de Heroku ne transmet pas l'UDP. Laissez la feature `http3` désactivée sur Heroku.

## Étapes suivantes

- [Automatisation](automation.md) : la CI, la fusion automatique dans `master`, et comment les désactiver.
- [Docker](deploy-docker.md) : la même image sur votre ordinateur ou sur d'autres plateformes.
- [Configuration](configuration.md) : variables d'environnement et réglages.
