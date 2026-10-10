# Desplegar con Docker

Todo proyecto creado con `paquets new` tiene un `Dockerfile` que construye una pequeña imagen de producción: el servidor en modo release, las islas en WebAssembly y los archivos estáticos. Heroku lo usa, y también pueden hacerlo Render, Fly.io, Google Cloud Run, Railway o tu propio servidor. Esta página explica la imagen, cómo construirla y ejecutarla en tu ordenador, cómo añadir una base de datos y qué comprobar en cada plataforma.

## El Dockerfile

```dockerfile
# Imagen de producción (Heroku, y tal cual en Render, Fly.io, Cloud Run…).
#
#   docker build -t my-app . && docker run -p 8080:8080 my-app

FROM rust:1-slim-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
# Las islas, compiladas a WebAssembly con la versión de wasm-bindgen de Cargo.lock.
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && cargo install wasm-bindgen-cli --version "$version" --locked \
    && wasm-bindgen --target web --no-typescript --out-dir pkg \
        target/wasm32-unknown-unknown/wasm-release/my_app.wasm
# El servidor.
RUN cargo build --release --bin my-app && cp target/release/my-app /server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /app/pkg ./pkg
COPY public ./public
USER app
# Heroku define PORT al arrancar; 8080 en los demás sitios.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
```

Tiene dos etapas:

1. **build**, a partir de la imagen oficial de Rust: compila las islas a WebAssembly (con la versión de `wasm-bindgen-cli` que corresponde a `Cargo.lock`), escribe el bundle en `pkg/` y luego compila el servidor en modo release.
2. **runtime**, a partir de una imagen Debian reducida: solo contiene el programa, `pkg/`, `public/` y los certificados raíz (necesarios para las conexiones TLS a una base de datos). El programa se ejecuta con un usuario sin privilegios de root.

El directorio de trabajo es `/app`, donde el servidor encuentra `public/` y `pkg/` al arrancar. El `Dockerfile` del proyecto mínimo (`paquets new --minimal`) es el mismo sin los pasos de WebAssembly.

## Constrúyela y ejecútala en tu ordenador

Necesitas [Docker](https://docs.docker.com/get-docker/) (`paquets doctor` te dice si está instalado).

```sh
docker build -t my-app .
docker run --rm -p 8080:8080 my-app
```

Abre http://localhost:8080. La primera construcción tarda varios minutos: compila todas las dependencias, y `wasm-bindgen-cli`, en modo release. Detén el contenedor con `Ctrl+C`: Paquets recibe la señal y se apaga de forma ordenada.

### El .dockerignore

`COPY . .` envía a Docker toda la carpeta del proyecto. Si no tienes cuidado, eso incluye `target/`, que puede pesar gigabytes. `paquets new` crea junto al `Dockerfile` un `.dockerignore` que lo deja fuera (añade uno con este contenido a un proyecto más antiguo):

```text
target
pkg
.git
*.db
```

Las construcciones empiezan más rápido, y tu base de datos local nunca acaba en una imagen.

## Variables de entorno

Pasa los ajustes con `-e`:

```sh
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://app:secret@db.example.com:5432/app \
  -e ADMIN_EMAIL=me@example.com \
  my-app
```

`PORT` vale `8080` en la imagen. Las plataformas que eligen el puerto (Heroku, Cloud Run…) definen `PORT` al arrancar el contenedor, y su valor tiene prioridad sobre el de la imagen. Para llegar desde el contenedor a una base de datos que se ejecuta en tu propio ordenador, usa el nombre de host `host.docker.internal` (Docker Desktop) en lugar de `localhost`.

## Un archivo SQLite en un volumen

El sistema de archivos de un contenedor desaparece con el contenedor. Para conservar una base de datos SQLite, guárdala en un volumen. El programa se ejecuta con el usuario `app`, así que prepara una carpeta en la que pueda escribir: añade esta línea a la etapa runtime del `Dockerfile`, justo antes de `USER app`:

```dockerfile
RUN mkdir /data && chown app /data
```

Después ejecuta el contenedor con un volumen con nombre:

```sh
docker run --rm -p 8080:8080 \
  -v my-app-data:/data \
  -e DATABASE_URL=sqlite:///data/app.db \
  my-app
```

`sqlite:///data/app.db` es la ruta absoluta `/data/app.db`. El volumen `my-app-data` sobrevive al contenedor: tus datos siguen ahí en el siguiente `docker run`.

## Con PostgreSQL: docker compose

Para tener un servidor de base de datos junto a la aplicación, describe los dos en un archivo `compose.yaml`:

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

La aplicación espera a que la base de datos esté lista, se conecta a ella, crea sus tablas y sirve en http://localhost:8080.

## Otras plataformas

La imagen funciona tal cual en cualquier plataforma que ejecute contenedores. Hay dos cosas que comprobar en cada una: el puerto, y dónde está la dirección IP del cliente.

El escudo de seguridad limita las peticiones por dirección IP. Detrás del proxy de una plataforma, todas las peticiones vienen del proxy, así que dile al escudo dónde está la dirección real; si no, todos tus visitantes comparten un mismo límite de tasa. El escudo por defecto lo hace automáticamente solo en Heroku. En otros sitios:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // la entrada de X-Forwarded-For situada más a la derecha
        .build(),
);
```

| Plataforma | Puerto | Origen de la IP del cliente |
|---|---|---|
| Heroku | define `PORT` | automático |
| Render | detecta el puerto expuesto, o define `PORT` | `ClientIpSource::heroku()` |
| Fly.io | `internal_port = 8080` en `fly.toml` | `ClientIpSource::fly()` |
| Google Cloud Run | define `PORT` (8080 por defecto) | `ClientIpSource::heroku()` |
| Detrás de Cloudflare | — | `ClientIpSource::cloudflare()` |
| Tu servidor, detrás de Nginx o Caddy | según lo configures | `ClientIpSource::heroku()`, o `ClientIpSource::x_real_ip()` con `proxy_set_header X-Real-IP` |
| Tu servidor, sin proxy | `-p 80:8080` | el valor por defecto (la dirección de la conexión) |

Consulta la documentación de tu plataforma para saber exactamente qué cabecera define, y lee [Seguridad](security.md#la-ip-del-cliente-detrás-de-un-proxy) antes de confiar en una.

HTTPS: Paquets sirve HTTP sin cifrar. Todas las plataformas anteriores proporcionan HTTPS delante del contenedor. En tu propio servidor, pon Caddy o Nginx delante, que además se encargan de los certificados y de la compresión.

## Una comprobación de salud

La mayoría de las plataformas pueden comprobar que la aplicación responde. Añade una ruta que responda rápido:

```rust
app.get("/health", |_| async { "ok" });
```

y haz que la comprobación de salud de la plataforma apunte a `/health`.

## Construcciones más rápidas

Cada `docker build` recompila tus dependencias cuando cambia cualquier cosa del proyecto. Cuando las construcciones se vuelvan lentas:

- conserva el `.dockerignore` de arriba;
- en el `Dockerfile`, reemplaza `cargo install wasm-bindgen-cli` por la descarga de un binario precompilado desde las [releases de wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen/releases) (el propio `Dockerfile` del repositorio de Paquets lo hace);
- guarda en caché las dependencias compiladas en una capa aparte con una herramienta como [cargo-chef](https://github.com/LukeMathWalker/cargo-chef).

## Próximos pasos

- [Desplegar en Heroku desde el móvil](deploy-heroku-mobile.md): esta imagen, desplegada en unos pocos toques.
- [Configuración](configuration.md): todas las variables de entorno y todos los ajustes.
- [Seguridad](security.md): el escudo detrás de un proxy.
