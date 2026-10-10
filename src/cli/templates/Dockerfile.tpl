# Production image (Heroku, and as is on Render, Fly.io, Cloud Run…).
#
#   docker build -t {{name}} . && docker run -p 8080:8080 {{name}}

FROM rust:1-slim-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
# The islands, compiled to WebAssembly with the wasm-bindgen version of Cargo.lock.
RUN cargo build --lib --target wasm32-unknown-unknown --profile wasm-release \
    && version=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p') \
    && cargo install wasm-bindgen-cli --version "$version" --locked \
    && wasm-bindgen --target web --no-typescript --out-dir pkg \
        target/wasm32-unknown-unknown/wasm-release/{{crate}}.wasm
# The server.
RUN cargo build --release --bin {{name}} && cp target/release/{{name}} /server

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
