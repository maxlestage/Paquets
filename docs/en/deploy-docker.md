# Deploy with Docker

Every project created with `paquets new` has a `Dockerfile` that builds a small production image: the server in release mode, the islands in WebAssembly, and the static files. Heroku uses it, and so can Render, Fly.io, Google Cloud Run, Railway or your own server. This page explains the image, how to build and run it on your computer, how to add a database, and what to check on each platform.

## The Dockerfile

```dockerfile
# Production image (Heroku, and as is on Render, Fly.io, Cloud Run…).
#
#   docker build -t my-app . && docker run -p 8080:8080 my-app

FROM rust:1-slim-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
# The islands, compiled to WebAssembly with the wasm-bindgen version of Cargo.lock.
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && cargo install wasm-bindgen-cli --version "$version" --locked \
    && wasm-bindgen --target web --no-typescript --out-dir pkg \
        target/wasm32-unknown-unknown/wasm-release/my_app.wasm
# The server.
RUN cargo build --release --bin my-app && cp target/release/my-app /server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /app/pkg ./pkg
COPY public ./public
USER app
# Heroku sets PORT at startup; 8080 elsewhere.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
```

It has two stages:

1. **build**, from the official Rust image: it compiles the islands to WebAssembly (with the `wasm-bindgen-cli` version that matches `Cargo.lock`), writes the bundle to `pkg/`, then compiles the server in release mode.
2. **runtime**, from a slim Debian image: it contains only the program, `pkg/`, `public/` and the root certificates (needed for TLS connections to a database). The program runs as a non-root user.

The working directory is `/app`, where the server finds `public/` and `pkg/` at startup. The minimal project's `Dockerfile` (`paquets new --minimal`) is the same without the WebAssembly steps.

## Build and run it on your computer

You need [Docker](https://docs.docker.com/get-docker/) (`paquets doctor` tells you whether it is installed).

```sh
docker build -t my-app .
docker run --rm -p 8080:8080 my-app
```

Open http://localhost:8080. The first build takes several minutes: it compiles every dependency, and `wasm-bindgen-cli`, in release mode. Stop the container with `Ctrl+C`: Paquets receives the signal and shuts down gracefully.

### Add a .dockerignore

`COPY . .` sends the whole project folder to Docker, including `target/`, which can weigh gigabytes. Create a `.dockerignore` file next to the `Dockerfile`:

```text
target
pkg
.git
*.db
```

Builds start faster, and your local database never ends up in an image.

## Environment variables

Pass settings with `-e`:

```sh
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://app:secret@db.example.com:5432/app \
  -e ADMIN_EMAIL=me@example.com \
  my-app
```

`PORT` is `8080` in the image. Platforms that choose the port (Heroku, Cloud Run…) set `PORT` when they start the container, which wins over the image's value. To reach a database running on your own computer from the container, use the host name `host.docker.internal` (Docker Desktop) instead of `localhost`.

## A SQLite file in a volume

A container's file system disappears with the container. To keep a SQLite database, store it in a volume. The program runs as the `app` user, so prepare a folder it can write to: add this line to the runtime stage of the `Dockerfile`, just before `USER app`:

```dockerfile
RUN mkdir /data && chown app /data
```

Then run the container with a named volume:

```sh
docker run --rm -p 8080:8080 \
  -v my-app-data:/data \
  -e DATABASE_URL=sqlite:///data/app.db \
  my-app
```

`sqlite:///data/app.db` is the absolute path `/data/app.db`. The volume `my-app-data` survives the container: your data is still there at the next `docker run`.

## With PostgreSQL: docker compose

For a database server next to the app, describe both in a `compose.yaml` file:

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

The app waits for the database to be ready, connects to it, creates its tables and serves on http://localhost:8080.

## Other platforms

The image runs as is on any platform that runs containers. Two things to check on each one: the port, and where the client's IP address is.

The security shield limits requests per IP address. Behind a platform's proxy, every request comes from the proxy, so tell the shield where the real address is, otherwise all your visitors share one rate limit. The default shield does it automatically on Heroku only. Elsewhere:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // the right-most X-Forwarded-For entry
        .build(),
);
```

| Platform | Port | Client IP source |
|---|---|---|
| Heroku | sets `PORT` | automatic |
| Render | detects the exposed port, or set `PORT` | `ClientIpSource::heroku()` |
| Fly.io | `internal_port = 8080` in `fly.toml` | `ClientIpSource::fly()` |
| Google Cloud Run | sets `PORT` (8080 by default) | `ClientIpSource::heroku()` |
| Behind Cloudflare | — | `ClientIpSource::cloudflare()` |
| Your server, behind Nginx or Caddy | as you configure it | `ClientIpSource::heroku()`, or `ClientIpSource::x_real_ip()` with `proxy_set_header X-Real-IP` |
| Your server, without a proxy | `-p 80:8080` | the default (the connection's address) |

Check your platform's documentation for the exact header it sets, and read [Security](security.md#the-client-ip-behind-a-proxy) before trusting one.

HTTPS: Paquets serves plain HTTP. Every platform above provides HTTPS in front of the container. On your own server, put Caddy or Nginx in front, which also handles certificates and compression.

## A health check

Most platforms can check that the app answers. Add a route that answers quickly:

```rust
app.get("/health", |_| async { "ok" });
```

and point the platform's health check to `/health`.

## Faster builds

Each `docker build` recompiles your dependencies when anything in the project changes. When builds get slow:

- keep the `.dockerignore` above;
- in the `Dockerfile`, replace `cargo install wasm-bindgen-cli` by the download of a prebuilt binary from the [wasm-bindgen releases](https://github.com/wasm-bindgen/wasm-bindgen/releases) (the Paquets repository's own `Dockerfile` does this);
- cache the compiled dependencies in a separate layer with a tool such as [cargo-chef](https://github.com/LukeMathWalker/cargo-chef).

## Next steps

- [Deploy to Heroku from your phone](deploy-heroku-mobile.md): this image, deployed in a few taps.
- [Configuration](configuration.md): every environment variable and setting.
- [Security](security.md): the shield behind a proxy.
