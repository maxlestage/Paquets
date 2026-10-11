# Déployer avec Docker

Chaque projet créé avec `paquets new` a un `Dockerfile` qui construit une petite image de production : le serveur en mode release, les îlots en WebAssembly, et les fichiers statiques. Heroku l'utilise, et Render, Fly.io, Google Cloud Run, Railway ou votre propre serveur peuvent en faire autant. Cette page explique l'image, comment la construire et la lancer sur votre ordinateur, comment ajouter une base de données, et ce qu'il faut vérifier sur chaque plateforme.

## Le Dockerfile

```dockerfile
# Image de production (Heroku, et telle quelle sur Render, Fly.io, Cloud Run…).
#
#   docker build -t mon-app . && docker run -p 8080:8080 mon-app

FROM rust:1-slim-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
# Les îlots, compilés en WebAssembly avec la version de wasm-bindgen du Cargo.lock.
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && cargo install wasm-bindgen-cli --version "$version" --locked \
    && wasm-bindgen --target web --no-typescript --out-dir pkg \
        target/wasm32-unknown-unknown/wasm-release/mon_app.wasm
# Le serveur.
RUN cargo build --release --bin mon-app && cp target/release/mon-app /server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /app/pkg ./pkg
COPY public ./public
USER app
# Heroku définit PORT au démarrage ; 8080 ailleurs.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
```

Il comporte deux étapes :

1. **build**, à partir de l'image Rust officielle : elle compile les îlots en WebAssembly (avec la version de `wasm-bindgen-cli` qui correspond au `Cargo.lock`), écrit le bundle dans `pkg/`, puis compile le serveur en mode release.
2. **runtime**, à partir d'une image Debian allégée : elle ne contient que le programme, `pkg/`, `public/` et les certificats racine (nécessaires aux connexions TLS vers une base de données). Le programme tourne avec un utilisateur non root.

Le répertoire de travail est `/app`, où le serveur trouve `public/` et `pkg/` au démarrage. Le `Dockerfile` du projet minimal (`paquets new --minimal`) est identique, sans les étapes WebAssembly.

## Construire et lancer l'image sur votre ordinateur

Il vous faut [Docker](https://docs.docker.com/get-docker/) (`paquets doctor` vous indique s'il est installé).

```sh
docker build -t mon-app .
docker run --rm -p 8080:8080 mon-app
```

Ouvrez http://localhost:8080. La première construction prend plusieurs minutes : elle compile toutes les dépendances, ainsi que `wasm-bindgen-cli`, en mode release. Arrêtez le conteneur avec `Ctrl+C` : Paquets reçoit le signal et s'arrête proprement.

### Le .dockerignore

`COPY . .` envoie tout le dossier du projet à Docker. Sans précaution, cela inclut `target/`, qui peut peser plusieurs gigaoctets. `paquets new` crée à côté du `Dockerfile` un `.dockerignore` qui l'exclut (ajoutez-en un avec ce contenu à un projet plus ancien) :

```text
target
pkg
.git
*.db
```

Les constructions démarrent plus vite, et votre base de données locale ne se retrouve jamais dans une image.

## Variables d'environnement

Passez les réglages avec `-e` :

```sh
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://app:secret@db.example.com:5432/app \
  -e ADMIN_EMAIL=moi@example.com \
  mon-app
```

`PORT` vaut `8080` dans l'image. Les plateformes qui choisissent le port (Heroku, Cloud Run…) définissent `PORT` au démarrage du conteneur, ce qui l'emporte sur la valeur de l'image. Pour joindre depuis le conteneur une base de données qui tourne sur votre propre ordinateur, utilisez le nom d'hôte `host.docker.internal` (Docker Desktop) au lieu de `localhost`.

## Un fichier SQLite dans un volume

Le système de fichiers d'un conteneur disparaît avec lui. Pour conserver une base SQLite, stockez-la dans un volume. Le programme tourne avec l'utilisateur `app` : préparez donc un dossier dans lequel il peut écrire, en ajoutant cette ligne à l'étape runtime du `Dockerfile`, juste avant `USER app` :

```dockerfile
RUN mkdir /data && chown app /data
```

Lancez ensuite le conteneur avec un volume nommé :

```sh
docker run --rm -p 8080:8080 \
  -v mon-app-data:/data \
  -e DATABASE_URL=sqlite:///data/app.db \
  mon-app
```

`sqlite:///data/app.db` désigne le chemin absolu `/data/app.db`. Le volume `mon-app-data` survit au conteneur : vos données sont toujours là au `docker run` suivant.

## Avec PostgreSQL : docker compose

Pour un serveur de base de données à côté de l'application, décrivez les deux dans un fichier `compose.yaml` :

```yaml
services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://app:secret@db:5432/app
    depends_on:
      db:
        condition: service_healthy
  db:
    image: postgres:17
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: app
    volumes:
      - db-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app"]
      interval: 2s
      retries: 15
volumes:
  db-data:
```

```sh
docker compose up --build
```

L'application attend que la base de données soit prête, s'y connecte, crée ses tables et sert sur http://localhost:8080.

## Autres plateformes

L'image tourne telle quelle sur toute plateforme qui exécute des conteneurs. Deux points à vérifier sur chacune : le port, et l'endroit où se trouve l'adresse IP du client.

Le bouclier de sécurité limite les requêtes par adresse IP. Derrière le proxy d'une plateforme, toutes les requêtes viennent du proxy : indiquez donc au bouclier où se trouve la véritable adresse, sinon tous vos visiteurs partagent une seule limite de débit. Le bouclier par défaut le fait automatiquement sur Heroku uniquement. Ailleurs :

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // l'entrée la plus à droite de X-Forwarded-For
        .build(),
);
```

| Plateforme | Port | Source de l'IP du client |
|---|---|---|
| Heroku | définit `PORT` | automatique |
| Render | détecte le port exposé, ou définissez `PORT` | `ClientIpSource::heroku()` |
| Fly.io | `internal_port = 8080` dans `fly.toml` | `ClientIpSource::fly()` |
| Google Cloud Run | définit `PORT` (8080 par défaut) | `ClientIpSource::heroku()` |
| Derrière Cloudflare | — | `ClientIpSource::cloudflare()` |
| Votre serveur, derrière Nginx ou Caddy | selon votre configuration | `ClientIpSource::heroku()`, ou `ClientIpSource::x_real_ip()` avec `proxy_set_header X-Real-IP` |
| Votre serveur, sans proxy | `-p 80:8080` | la valeur par défaut (l'adresse de la connexion) |

Vérifiez dans la documentation de votre plateforme l'en-tête exact qu'elle définit, et lisez [Sécurité](security.md#lip-du-client-derrière-un-proxy) avant de faire confiance à l'un d'eux.

HTTPS : Paquets sert du HTTP simple. Toutes les plateformes ci-dessus fournissent HTTPS devant le conteneur. Sur votre propre serveur, placez Caddy ou Nginx devant, qui s'occupent aussi des certificats et de la compression.

## Un health check

La plupart des plateformes savent vérifier que l'application répond. Ajoutez une route qui répond rapidement :

```rust
app.get("/health", |_| async { "ok" });
```

et faites pointer le health check de la plateforme vers `/health`.

## Des constructions plus rapides

Chaque `docker build` recompile vos dépendances dès que quoi que ce soit change dans le projet. Quand les constructions deviennent lentes :

- gardez le `.dockerignore` ci-dessus ;
- dans le `Dockerfile`, remplacez `cargo install wasm-bindgen-cli` par le téléchargement d'un binaire précompilé depuis les [releases de wasm-bindgen](https://crates.io/crates/wasm-bindgen) (le `Dockerfile` du dépôt de Paquets fait exactement cela) ;
- mettez en cache les dépendances compilées dans une couche séparée avec un outil comme [cargo-chef](https://crates.io/crates/cargo-chef).

## Étapes suivantes

- [Déployer sur Heroku depuis votre téléphone](deploy-heroku-mobile.md) : cette image, déployée en quelques gestes.
- [Configuration](configuration.md) : toutes les variables d'environnement et tous les réglages.
- [Sécurité](security.md) : le bouclier derrière un proxy.
