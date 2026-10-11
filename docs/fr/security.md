# Sécurité

Chaque application Paquets est protégée dès sa première ligne : `App::new()` place le bouclier recommandé de [securite](https://maxlestage.github.io/Securite/) devant chaque requête. Les attaques sont bloquées avant que votre code ne s'exécute, et chaque réponse reçoit les en-têtes de sécurité dont les navigateurs ont besoin. Cette page explique ce que fait le bouclier, comment le modifier, et ce qui reste à votre charge, comme la connexion des utilisateurs.

## Ce que fait le bouclier

Le bouclier par défaut exécute dix guards, dans cet ordre, pour chaque requête (environ 6 µs au total) :

| Guard | Nom | Protège contre | Quand il bloque |
|---|---|---|---|
| Identifiant de requête | `request_id` | — | jamais : il donne à chaque requête un `X-Request-Id`, renvoyé aussi dans la réponse |
| Hygiène | `hygiene` | les requêtes malformées ou démesurées | URI de plus de 8 Kio, plus de 100 en-têtes, segments `..`, mauvais encodage : `400` |
| Filtre de méthodes | `method` | les méthodes HTTP inhabituelles, les astuces de substitution de méthode | les méthodes autres que `GET HEAD POST PUT PATCH DELETE OPTIONS` : `405` |
| Limite de corps | `body_limit` | les envois énormes | un corps annoncé de plus de 2 Mio : `413` |
| Limitation de débit | `rate_limit` | les inondations de requêtes, la force brute, le scraping | plus de 20 requêtes par seconde et par IP, avec des pointes jusqu'à 100 : `429` |
| Blocage des robots | `bot_blocker` | les outils d'attaque | les scanners connus (sqlmap, nikto, nuclei…) : `403` |
| Pare-feu (WAF) | `waf` | l'injection SQL, le XSS, l'injection de commandes, Log4Shell, le SSRF, la traversée de répertoires… | un motif d'attaque dans l'URL, la query string ou un en-tête : `403` |
| CSRF | `csrf` | la falsification de requête intersite | un `POST`, `PUT`, `PATCH` ou `DELETE` envoyé par un navigateur depuis un autre site : `403` |
| En-têtes de sécurité | `security_headers` | le XSS, le clickjacking, le MIME sniffing, le retour forcé à HTTP | jamais : il ajoute des en-têtes à chaque réponse |
| Durcissement des cookies | `cookies` | le vol des cookies de session | jamais : il ajoute `Secure`, `HttpOnly` et `SameSite=Lax` à chaque cookie |

La ligne de démarrage le confirme : `security: on (10 guards)`.

### À quoi ressemble une requête bloquée

Une requête bloquée n'atteint jamais vos handlers. Le client reçoit un message court et générique (aucun détail qui aiderait un attaquant), au format JSON standard « problem details » :

```http
HTTP/1.1 429 Too Many Requests
content-type: application/problem+json
retry-after: 1
ratelimit-limit: 100
ratelimit-remaining: 0

{"type":"about:blank","title":"Too Many Requests","status":429,"code":"rate_limited","detail":"Too many requests, slow down.","request_id":"222caeb2-c3fa-4692-a3de-b6650499864c"}
```

| Code | Statut | Guard |
|---|---|---|
| `waf_blocked` | `403` | pare-feu |
| `csrf_rejected` | `403` | CSRF |
| `rate_limited` | `429` | limitation de débit |
| `bot_blocked` | `403` | blocage des robots |

Le journal des requêtes affiche le statut (`GET /?q=... 403 0.05 ms`). Le `request_id` vous permet de retrouver une requête dans vos journaux quand un utilisateur signale un problème.

### En-têtes de sécurité

Chaque réponse, erreurs et requêtes bloquées comprises, porte :

| En-tête | Valeur | Effet |
|---|---|---|
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains` | les navigateurs n'utilisent plus que HTTPS, pendant deux ans |
| `X-Content-Type-Options` | `nosniff` | pas de devinette sur les types de contenu |
| `X-Frame-Options` | `DENY` | vos pages ne peuvent pas être affichées dans un cadre (clickjacking) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | les URL complètes ne fuient pas vers d'autres sites |
| `Permissions-Policy` | caméra, micro, géolocalisation… désactivés | le code tiers ne peut pas s'en servir |
| `Cross-Origin-Opener-Policy`, `Cross-Origin-Resource-Policy` | `same-origin` | l'isolation vis-à-vis des autres sites |
| `Content-Security-Policy` | voir plus bas | les scripts, styles et ressources qui ont le droit de se charger |

## Les en-têtes de sécurité et la Content-Security-Policy

La Content-Security-Policy (CSP) indique au navigateur ce qu'une page a le droit de charger et d'exécuter. Même si un attaquant glisse du HTML dans une page, une CSP stricte empêche son script de s'exécuter. Paquets la définit de trois façons :

| Réponse | Politique |
|---|---|
| Une [`Page`](pages.md) | sa propre politique, calculée au rendu de la page : les styles et scripts en ligne de la page sont autorisés par leur empreinte SHA-256, ainsi que vos propres fichiers, les images `https:` et le WebAssembly (voir [Pages](pages.md#la-content-security-policy)) |
| Les fichiers HTML de `public/` | quand `public/` contient des fichiers `.html`, le bouclier par défaut les lit au démarrage et autorise leurs scripts et styles en ligne (par empreinte), les feuilles de style et scripts externes qu'ils lient (Google Fonts compris, avec `https://fonts.gstatic.com` pour ses polices), le WebAssembly et les images `https:` |
| Tout le reste (JSON, texte, `Html(...)`) | la politique du bouclier : uniquement les ressources de la même origine, aucun script ni style en ligne |

La politique automatique pour `public/` vient avec le bouclier par défaut. Si vous remplacez le bouclier avec `app.security(...)`, définissez vous-même une politique quand votre HTML statique en a besoin, avec le guard `SecurityHeaders` :

```rust
use paquets::securite::headers::ContentSecurityPolicy;
use paquets::securite::prelude::*;

let csp = ContentSecurityPolicy::strict()
    .add_sources("script-src", ["https://cdn.example.com"])
    .remove_directive("upgrade-insecure-requests");
app.security(
    paquets::shield()
        .guard(SecurityHeaders::new().content_security_policy(csp))
        .build(),
);
```

## L'IP du client derrière un proxy

La limitation de débit compte les requêtes **par adresse IP** : le bouclier doit donc connaître la véritable adresse de chaque visiteur. Sur Heroku, l'application se trouve derrière le routeur de Heroku, et toutes les connexions viennent du routeur : la véritable adresse est dans l'en-tête `X-Forwarded-For`. Le bouclier par défaut détecte Heroku (la variable d'environnement `DYNO`) et lit cet en-tête en partant de la droite, la méthode sûre. Ailleurs, il utilise l'adresse de la connexion.

Sur les autres plateformes, derrière un proxy, indiquez au bouclier où trouver l'adresse ; sinon tous les visiteurs partagent l'adresse du proxy, et la limitation de débit s'applique à tous à la fois :

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // Render, les load balancers AWS, un seul Nginx
        // .client_ip(ClientIpSource::fly())        // Fly.io
        // .client_ip(ClientIpSource::cloudflare()) // derrière Cloudflare
        .build(),
);
```

> [!WARNING]
> Ne faites confiance à un en-tête de transfert que si votre application n'est joignable **que** par le proxy qui le définit. Sinon, n'importe qui peut envoyer l'en-tête et choisir son adresse. Voir le [guide de l'IP du client](https://maxlestage.github.io/Securite/fr/docs/client-ip/) de securite.

Les handlers peuvent lire l'adresse trouvée par le bouclier avec `req.extensions().get::<ClientIp>()` (voir [Requêtes et réponses](requests-responses.md#ladresse-ip-du-client)).

## Modifier le bouclier

`paquets::shield()` renvoie le bouclier par défaut sous forme de builder, prêt à être modifié, avec la détection de Heroku incluse. Changez ce dont vous avez besoin, appelez `.build()`, et passez le résultat à `app.security(...)` :

```rust
use paquets::prelude::*;
use paquets::securite::prelude::*;

let mut app = App::new();
app.security(
    paquets::shield()
        .guard(RateLimit::per_minute(600)) // remplace la limitation de débit par défaut
        .without("bot_blocker")            // retire un guard, par son nom
        .build(),
);
```

| Méthode du builder | Effet |
|---|---|
| `.guard(g)` | ajoute un guard, ou remplace le guard qui porte le même nom |
| `.without(nom)` | retire un guard : `"rate_limit"`, `"waf"`, `"csrf"`… |
| `.client_ip(source)` | d'où vient l'IP du client (voir plus haut) |
| `.error_format(format)` | comment répondre aux requêtes bloquées : `ErrorFormat::ProblemJson` (par défaut), `PlainText`, `Empty` |
| `.on_event(f)` | une fonction appelée pour chaque requête bloquée |
| `.build()` | le bouclier terminé |

Importez les guards avec `use paquets::securite::prelude::*;`. `app.security(...)` peut être appelé n'importe où avant `run()` : le bouclier est choisi au démarrage de l'application.

### Recettes courantes

**Une API plus sollicitée.** Relevez la limitation de débit, par minute ou avec une pointe sur mesure :

```rust
app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
app.security(paquets::shield().guard(RateLimit::new(Quota::per_second(50).burst(200))).build());
```

**Des envois plus gros.** Relevez ensemble la limite du bouclier et la limite de lecture (voir [Requêtes et réponses](requests-responses.md#taille-du-corps)) :

```rust
app.body_limit(10 * 1024 * 1024);
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

**Un front-end sur une autre origine.** Autorisez-le dans CORS, et faites-lui confiance dans le guard CSRF (voir [API REST](rest-api.md#appeler-lapi-depuis-un-navigateur)) :

```rust
app.security(
    paquets::shield()
        .guard(Cors::new().allow_origin("https://app.example.com"))
        .guard(Csrf::new().trusted_origin("https://app.example.com"))
        .build(),
);
```

**Un formulaire renvoyé par un autre site.** Certains fournisseurs de connexion ou de paiement renvoient le visiteur sur votre site avec un `POST` de formulaire. Exemptez ce chemin du guard CSRF :

```rust
app.security(paquets::shield().guard(Csrf::new().exempt("/auth/callback")).build());
```

Les webhooks envoyés par des serveurs (GitHub, Stripe…) n'en ont pas besoin : les requêtes qui ne viennent pas d'un navigateur passent le guard CSRF.

**Voir ce qui est bloqué.** Affichez chaque requête bloquée :

```rust
app.security(
    paquets::shield()
        .on_event(|e| {
            if e.enforced {
                eprintln!("[sécurité] {} a bloqué {} {} ({})", e.guard, e.method, e.path, e.status);
            }
        })
        .build(),
);
```

```text
[sécurité] waf a bloqué GET / (403 Forbidden)
```

**Essayer un guard sans bloquer.** La plupart des guards ont un mode surveillance, qui signale via `on_event` (avec `e.enforced == false`) au lieu de bloquer : `.guard(Waf::new().monitor())`.

### Préréglages

securite propose trois préréglages. `paquets::shield()` est le préréglage recommandé, plus la détection de Heroku :

| Préréglage | Pour | Différences avec le préréglage recommandé |
|---|---|---|
| `ShieldBuilder::recommended()` | les sites et les API utilisés par des navigateurs | — |
| `ShieldBuilder::strict()` | les sites sensibles | ajoute une redirection vers HTTPS, des bannissements automatiques après des blocages répétés, une limite de connexions simultanées par IP et une vérification du type de contenu ; des limites plus basses (1 Mio, 10 requêtes par seconde) ; des cookies `SameSite=Strict` ; `Cross-Origin-Embedder-Policy: require-corp` (les images et scripts d'autres sites doivent l'autoriser) |
| `ShieldBuilder::api()` | les API JSON utilisées avec des jetons, pas des cookies | pas de guard CSRF, uniquement des corps JSON, une CSP qui n'autorise rien, `Cache-Control: no-store` |

Les préréglages ne connaissent pas Heroku : ajoutez vous-même la source de l'IP du client.

```rust
app.security(ShieldBuilder::strict().client_ip(ClientIpSource::heroku()).build());
```

`Shield::strict()` et `Shield::api()` sont les mêmes préréglages, déjà construits.

### Désactiver le bouclier

```rust
app.security(None);
```

Ne le faites que si une autre couche protège déjà l'application (un WAF au niveau de votre CDN, par exemple), ou pour un benchmark. La ligne de démarrage indique alors `security: off`.

## Ce que le bouclier ne fait pas

- **Il ne lit pas le corps des requêtes.** Le pare-feu inspecte l'URL, la query string et les en-têtes. Les corps JSON et les formulaires relèvent de vos handlers : validez-les (voir [Requêtes et réponses](requests-responses.md#corps-json)). Deux choses vous protègent déjà de ce côté : memoire envoie chaque valeur comme paramètre lié, les requêtes des modèles ne peuvent donc pas être détournées en injection SQL ; et les vues échappent tout le texte, les données affichées dans une page ne peuvent donc pas injecter de HTML.
- **Il ne connaît pas vos utilisateurs.** L'authentification et les permissions sont à votre charge (section suivante).
- **Il ne fournit pas HTTPS à lui seul.** Paquets sert du HTTP simple ; c'est Heroku, un reverse proxy ou un CDN placé devant qui fournit HTTPS. L'en-tête `Strict-Transport-Security` maintient ensuite les navigateurs sur HTTPS.

## Authentification

Paquets n'impose pas de façon de connecter les utilisateurs. Voici trois approches courantes, de la plus simple à la plus complète :

### Un jeton partagé pour une API

Pour une API utilisée par vos propres scripts ou applications, vérifiez un jeton secret dans un middleware : voir [API REST](rest-api.md#protéger-lapi).

### Un mot de passe pour une section d'administration

Le guard `Auth` de securite protège des chemins avec l'authentification HTTP Basic (le navigateur affiche une boîte de connexion), des jetons bearer ou des clés d'API :

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .guard(Auth::basic("Admin").user("admin", "correct horse battery").protect(["/admin"]))
        .build(),
);

app.get("/admin", |req: Request| async move {
    let name = req.extensions().get::<Principal>().map(|p| p.name.to_string()).unwrap_or_default();
    format!("Bonjour {name}")
});
```

Tous les chemins sous `/admin` demandent désormais le nom d'utilisateur et le mot de passe ; les autres restent publics. Lisez le mot de passe depuis une variable d'environnement (`std::env::var("ADMIN_PASSWORD")`) plutôt que de l'écrire dans le code. Le [guide de l'authentification](https://maxlestage.github.io/Securite/fr/docs/auth/) de securite couvre les jetons, les clés d'API et les vérifications sur mesure comme JWT.

### Comptes et sessions

Pour des comptes utilisateurs, le schéma habituel est le suivant :

1. enregistrez les utilisateurs dans un modèle, avec une **empreinte** du mot de passe (utilisez une crate dédiée comme `argon2`, jamais un simple hachage ni le mot de passe lui-même) ;
2. à la connexion, vérifiez le mot de passe, créez un jeton de session aléatoire (issu d'un générateur aléatoire cryptographique), enregistrez-le dans un modèle `Session`, et envoyez-le dans un cookie : `res::redirect("/").cookie(Cookie::new("session", token))` ;
3. dans un middleware, lisez `req.cookie("session")`, retrouvez la session avec `req.db()`, et attachez l'utilisateur à la requête avec `req.set(user)` ; les handlers le lisent avec `req.get::<User>()`.

Le bouclier s'occupe du reste : le cookie reçoit `Secure`, `HttpOnly` et `SameSite=Lax`, la limitation de débit ralentit les tentatives pour deviner les mots de passe, et le guard CSRF empêche d'autres sites d'envoyer des formulaires au nom de vos utilisateurs.

## Pour aller plus loin

La documentation de securite a une page par guard : [WAF](https://maxlestage.github.io/Securite/fr/docs/waf/), [limitation de débit](https://maxlestage.github.io/Securite/fr/docs/rate-limit/), [CSRF](https://maxlestage.github.io/Securite/fr/docs/csrf/), [en-têtes de sécurité](https://maxlestage.github.io/Securite/fr/docs/security-headers/), [préréglages](https://maxlestage.github.io/Securite/fr/docs/shield-presets/), et bien d'autres : filtre d'IP, filtre d'hôte, bannissements automatiques, signatures de webhooks. Dans Paquets, securite est réexporté sous `paquets::securite`.

## Étapes suivantes

- [Erreurs](errors.md) : comment les erreurs deviennent des réponses.
- [Configuration](configuration.md) : tous les réglages de l'application.
- [Déployer sur Heroku](deploy-heroku-mobile.md) : HTTPS et l'IP du client sont gérés pour vous.
