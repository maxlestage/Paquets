# Publishing on crates.io

Paquets is published on [crates.io](https://crates.io/crates/paquets) as two crates,
`paquets-macros` (the attribute macros) and `paquets`, under its
[proprietary license](LICENSE): the source is visible there so that it can be audited and
built, and anyone may depend on it.

## One release = one tag

1. Bump `version` in [`Cargo.toml`](Cargo.toml) and [`macros/Cargo.toml`](macros/Cargo.toml)
   (and the `=x.y.z` pin of `paquets-macros` in the root manifest), update
   [`CHANGELOG.md`](CHANGELOG.md) and commit.
2. Open **Releases → Draft a new release**, create the tag `vX.Y.Z` (the same version with a
   `v` in front) and tap **Publish release**.

The [`release.yml`](.github/workflows/release.yml) workflow checks that the tag matches the
version and that the four packages Paquets is built on are on crates.io, runs the tests and
publishes `paquets-macros`, then `paquets`. It works from the GitHub website on a phone;
nothing is installed locally. From the **Actions** tab it can also be started by hand, as a dry
run by default (the packages are built and checked, nothing is uploaded). The full guide:
[`docs/en/publish-crates-io.md`](docs/en/publish-crates-io.md).

## One-time setup

- A crates.io account with a verified e-mail address.
- A token from [crates.io/settings/tokens](https://crates.io/settings/tokens) with the scopes
  **publish-new** and **publish-update**, saved as the repository secret
  `CARGO_REGISTRY_TOKEN` (GitHub → Settings → Secrets and variables → Actions).

## From a computer

```sh
cargo login                              # paste the token, once
cargo publish -p paquets-macros
cargo publish -p paquets                 # once the macros and the four packages are online
```

## The ecosystem, in order

The five projects by Maxime Nathan Lestage depend on each other through crates.io, so they
are published in this order, each one after the previous one is online:

1. **Active** — `active`
2. **Vitesse** — `vitesse`
3. **Securite** — `securite`
4. **Memoire** — `memoire-orm-derive`, then `memoire-orm`
5. **Paquets** — `paquets-macros`, then `paquets` (this repository; depends on the four above)
