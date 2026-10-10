# The image deployed by the "Deploy to Heroku" button: the Paquets website, which is
# itself a Paquets app (pages rendered by active, islands in WebAssembly), with its
# live demo at /demo and /api/todos.
#
#   docker build -t paquets . && docker run --rm -p 8080:8080 paquets

FROM rust:1-slim-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY . .
WORKDIR /src/site
# The islands, compiled to WebAssembly, then bound with the wasm-bindgen version of
# site/Cargo.lock (a prebuilt binary, or compiled if it cannot be downloaded).
# No --locked: a dependency changed from a phone (where Cargo.lock cannot be
# regenerated) is resolved here; the others keep their locked versions.
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && (curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}/wasm-bindgen-${version}-x86_64-unknown-linux-musl.tar.gz" \
            | tar -xz --strip-components=1 -C /usr/local/bin \
        || cargo install wasm-bindgen-cli --version "$version" --locked) \
    && wasm-bindgen --target web --no-typescript --out-dir pkg \
        target/wasm32-unknown-unknown/wasm-release/paquets_site.wasm
# The server.
RUN cargo build --release --bin paquets-site && cp target/release/paquets-site /server

# A small image: the program, its static files, the islands and the documentation.
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 paquets
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY --from=build /src/site/public ./public
COPY --from=build /src/site/pkg ./pkg
COPY --from=build /src/docs ./docs
USER paquets
# Heroku sets PORT at startup; 8080 elsewhere.
ENV PORT=8080 DOCS_DIR=/app/docs
EXPOSE 8080
CMD ["server"]
