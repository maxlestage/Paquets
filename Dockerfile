# The image deployed by the "Deploy to Heroku" button: the Paquets website (Yew,
# WebAssembly) served by a Paquets app, with its live demo at /demo and /api/todos.
#
#   docker build -t paquets . && docker run --rm -p 8080:8080 paquets

# 1. The website, compiled to WebAssembly by Trunk.
FROM rust:1-slim-bookworm AS site
ARG TRUNK_VERSION=v0.21.14
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown \
    && curl -sSfL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
        | tar -xz -C /usr/local/bin
WORKDIR /src
COPY docs ./docs
COPY site ./site
RUN cd site && trunk build --release

# 2. The server: examples/todo.rs.
FROM rust:1-slim-bookworm AS server
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY macros ./macros
COPY src ./src
COPY examples ./examples
COPY tests ./tests
# No --locked: a dependency changed from a phone (where Cargo.lock cannot be
# regenerated) is resolved here; the others keep their locked versions.
RUN cargo build --release --example todo && cp target/release/examples/todo /server

# 3. A small image: the program and the website (public/).
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 paquets
WORKDIR /app
COPY --from=server /server /usr/local/bin/server
COPY --from=site /src/site/dist ./public
USER paquets
# Heroku sets PORT at startup; 8080 elsewhere.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
