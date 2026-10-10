# Production image (Heroku, and as is on Render, Fly.io, Cloud Run…).
#
#   docker build -t {{name}} . && docker run -p 8080:8080 {{name}}

FROM rust:1-slim-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release && cp target/release/{{name}} /server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /server /usr/local/bin/server
COPY public ./public
USER app
# Heroku sets PORT at startup; 8080 elsewhere.
ENV PORT=8080
EXPOSE 8080
CMD ["server"]
