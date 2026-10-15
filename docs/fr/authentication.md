# Authentification

`app.auth(Auth::new())` ajoute des comptes à une application en une ligne, comme Laravel Breeze : inscription, connexion, déconnexion, mot de passe oublié, codes à deux facteurs, et les pages qui vont avec. Tout repose sur [Securite](https://securite-2db4c1994be7.herokuapp.com/) : hachage Argon2id, limitation des tentatives de connexion, cookies et liens signés, codes TOTP.

## Une ligne

```rust
use paquets::prelude::*;
use paquets::auth::{self, Auth};

async fn dashboard(req: Request) -> String {
    let user = req.user().expect("checked by login_required");
    format!("Hello {}", user.name)
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.auth(Auth::new().name("Mon appli").admins(["moi@example.com"]));
    app.get("/dashboard", dashboard.with(auth::login_required));
    app.run()
}
```

Ou `paquets make:auth` dans un projet (voir [Générateurs](generators.md)).

## Les pages

| Route | Page |
|---|---|
| `GET/POST /register` | crée un compte (nom, e-mail, mot de passe deux fois), puis connecte |
| `GET/POST /login` | connecte, puis va à `?next=` (un chemin local seulement) ou à l'accueil |
| `POST /logout` | déconnecte |
| `GET/POST /forgot-password` | envoie un lien de réinitialisation |
| `GET/POST /reset-password?token=…` | choisit un nouveau mot de passe |
| `GET /account` | le compte : codes à deux facteurs, déconnexion |
| `GET/POST /two-factor` | le code demandé après le mot de passe, une fois les codes activés |

Les pages sont rendues sur le serveur avec le thème par défaut, leur propre Content-Security-Policy et `noindex`. Leurs champs sont vérifiés avec les mêmes règles que les formulaires d'Active (`required`, `email`, `min_length`), et un formulaire invalide revient avec une erreur sous chaque champ (statut `422`).

## Dans les handlers

`req.user()` (le trait `AuthExt` du prélude) est l'[`User`](https://docs.rs/paquets/latest/paquets/auth/struct.User.html) connecté, ou `None` :

```rust
use paquets::prelude::*;

async fn hello(req: Request) -> String {
    match req.user() {
        Some(user) => format!("Bonjour {}", user.name),
        None => "Bonjour inconnu".into(),
    }
}
```

Deux middlewares protègent les routes : `auth::login_required` (les visiteurs vont à la page de connexion, puis reviennent ; les clients d'API reçoivent un `401`) et `auth::admin_required` (`403` pour les utilisateurs qui ne sont pas administrateurs). Les utilisateurs sont les lignes de la table `users` (`id`, `email`, `name`, `admin`, `created_at`…), créée au démarrage : interrogez-la comme n'importe quel [modèle](database.md). Le hachage du mot de passe, le secret à deux facteurs et la version de session n'apparaissent jamais en JSON.

## Réglages

| Méthode | Par défaut |
|---|---|
| `.prefix("/auth")` | les routes à la racine (`/login`) |
| `.home("/dashboard")` | `/`, après une connexion sans `?next=` |
| `.name("Mon appli")` | le titre des pages et des applications d'authentification |
| `.url("https://example.com")` | `APP_URL`, puis le `Host` de la requête : l'adresse des liens envoyés par e-mail |
| `.secret(…)` | `APP_KEY` (32 octets ou plus), ou une clé aléatoire au démarrage |
| `.previous_secret(…)` | aucun : les anciennes `APP_KEY` restent acceptées après une rotation |
| `.registration(false)` | les visiteurs peuvent s'inscrire |
| `.two_factor(false)` | les utilisateurs peuvent activer les codes à deux facteurs |
| `.admins([…])` | les adresses e-mail des administrateurs |
| `.session_ttl(durée)` | 30 jours |
| `.hasher(PasswordHasher…)` | Argon2id, paramètres OWASP |
| `.throttle(LoginThrottle…)` | 5 échecs par compte et IP, puis 15 minutes |
| `.mailer(|link| async {…})` | affiche le lien dans le terminal (builds de debug) |

En production, définissez `APP_KEY` (`openssl rand -base64 48`) : sans elle, une clé aléatoire est tirée au démarrage (un avertissement le signale, dans tous les builds), donc les sessions et les codes à deux facteurs cessent de fonctionner à chaque redémarrage. Pour la changer, passez l'ancienne à `.previous_secret(…)` pendant un temps. Définissez aussi `APP_URL`, pour que les liens de réinitialisation ne se fient jamais au `Host` d'une requête.

### Envoyer l'e-mail de réinitialisation

```rust
use paquets::auth::{Auth, ResetLink};

let auth = Auth::new().mailer(|link: ResetLink| async move {
    // Envoyez `link.url` à `link.user.email` avec la crate d'e-mail de votre choix.
    println!("à {} : {}", link.user.email, link.url);
    Ok(())
});
```

La page « mot de passe oublié » répond la même chose que l'adresse ait un compte ou non.

## Comment elle vous protège

- **Mots de passe** hachés avec Argon2id, sur les threads bloquants de tokio ; une adresse inconnue prend autant de temps qu'une connue (`verify_dummy`) : les temps de réponse ne révèlent rien.
- **Force brute** : les connexions ratées, demandes de réinitialisation et codes à deux facteurs sont comptés par compte et IP ; le sixième échec répond `429` avec `Retry-After`.
- **Sessions** : cookies signés en HMAC-SHA256, `HttpOnly`, `SameSite=Lax`, `Secure` en HTTPS. Changer de mot de passe met fin à toutes les sessions.
- **Liens de réinitialisation** signés, valables une heure et une seule fois.
- **Codes à deux facteurs** TOTP (RFC 6238) : n'importe quelle application d'authentification, avec la clé affichée sur la page du compte.
- **Secrets à deux facteurs chiffrés au repos** avec XChaCha20-Poly1305 (l'`Encrypter` de Securite), avec une clé dérivée d'`APP_KEY` (HMAC-SHA256, distincte de la clé de signature) et liée à la ligne de l'utilisateur. Un secret stocké en clair par une version précédente est toujours lu, puis chiffré à la connexion à deux facteurs suivante.
- **Journaux** : le `Debug` de `User` affiche `<redacted>` au lieu du hachage du mot de passe et du secret à deux facteurs.
- **CSRF** : le bouclier refuse les formulaires envoyés depuis un autre site (Fetch Metadata).

## Dans les tests

Argon2 est lent exprès, et plus encore sans optimisations : donnez aux tests un hacheur rapide.

```rust
use paquets::prelude::*;
use paquets::auth::{Auth, PasswordHasher};

fn app() -> App {
    let mut app = App::new();
    app.auth(
        Auth::new()
            .secret("a test secret that is long enough to sign cookies")
            .hasher(PasswordHasher::insecure_fast_for_tests()),
    );
    app
}
```

`paquets make:auth` compile aussi Argon2 avec optimisations en debug (`[profile.dev.package.argon2]` dans `Cargo.toml`).

## Étapes suivantes

- [Panneau d'administration](admin.md) : les administrateurs gèrent les modèles dans le navigateur.
- [Sécurité](security.md) : le bouclier devant chaque requête.
