# Contributing to Paquets

Thank you for your interest! Bug reports, ideas, documentation fixes and pull requests
are welcome. For a large change, open an issue first so we can discuss it.

## Before you push

Run the same checks as the CI:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --lib --no-default-features --features client --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --all-features
```

The website (`site/`) is a Paquets app in its own crate: if you change `site/` or
`docs/`, also run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings` and `cargo test` in
`site/` (the tests check every documentation page and link). See `site/README.md`.

## Branches

Every branch pushed to GitHub is tested and, when everything is green, **merged into
`master` automatically**. Name a branch `wip/...` to opt out while you work.

## Documentation

The documentation lives in `docs/` in English, French and Spanish, with the same page
names: when you change a page, please update the other languages too, or say so in your
pull request. Notable changes go in [`CHANGELOG.md`](CHANGELOG.md).

## The four packages

Paquets assembles [vitesse](https://github.com/maxlestage/Vitesse),
[securite](https://github.com/maxlestage/Securite),
[memoire-orm](https://github.com/maxlestage/Memoire) and
[active](https://github.com/maxlestage/Active). A bug in one of them is best fixed in its
own repository; Paquets then picks it up with `cargo update`.
