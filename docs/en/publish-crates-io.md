# Publish on crates.io

[crates.io](https://crates.io) is Rust's package registry: once Paquets is published there, anyone can install the command with `cargo install paquets` and use the library with `paquets = "0.1"`. This page is for the maintainers of Paquets (or of a fork under another name). It explains why the four packages must be published first, then how to publish everything from a phone, with GitHub releases.

## Why the order matters

crates.io only accepts packages whose dependencies are **also on crates.io**: a published package can't depend on a Git repository. Paquets depends on its four packages, so they must be published before it.

That is why Paquets' `Cargo.toml` gives each package both a `version` and a `git` source:

```toml
[dependencies]
active = { version = "0.1", git = "https://github.com/maxlestage/Active" }
memoire-orm = { version = "0.1", git = "https://github.com/maxlestage/Memoire", default-features = false, features = ["memory"] }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
vitesse = { version = "0.1", git = "https://github.com/maxlestage/Vitesse" }
securite = { version = "0.1", git = "https://github.com/maxlestage/Securite", default-features = false }
```

- While you develop, Cargo uses the `git` source: the latest code of each repository.
- When you publish, `cargo publish` removes the `git` source and keeps the `version`: the published `paquets` depends on `vitesse 0.1` **from crates.io**. If that version isn't there, crates.io refuses the upload.

## The publication order

Seven crates are published from five repositories. Each repository has its own release workflow, started the same way: by publishing a GitHub release.

| Order | Crate | Repository | Workflow (Actions tab) |
|---|---|---|---|
| 1 | `memoire-orm-derive` | [Memoire](https://github.com/maxlestage/Memoire) | **Publish to crates.io** (publishes both Memoire crates, the derive first) |
| 2 | `memoire-orm` | [Memoire](https://github.com/maxlestage/Memoire) | same run |
| 3 | `securite` | [Securite](https://github.com/maxlestage/Securite) | **Publish crate** |
| 4 | `vitesse` | [Vitesse](https://github.com/maxlestage/Vitesse) | **Release** |
| 5 | `active` | [Active](https://github.com/maxlestage/Active) | **Release** |
| 6 | `paquets-macros`, then `paquets` | [Paquets](https://github.com/maxlestage/Paquets) | **Release** |

`memoire-orm` needs `memoire-orm-derive`, and the Memoire workflow publishes them in that order. The four repositories of steps 1 to 5 don't depend on each other: you can release them in any order, as long as all of them are on crates.io before step 6. Paquets' workflow checks it, and says which crates are missing.

Each workflow skips a crate whose version is already on crates.io, so running one again is safe.

## One-time setup: the crates.io token

The workflows publish with a crates.io API token, stored as a GitHub secret called `CARGO_REGISTRY_TOKEN` in **each** of the five repositories.

### Create the token

1. In your phone's browser, open https://crates.io and tap **Log in with GitHub**.
2. Open **Account Settings** and check that your email address is verified: crates.io refuses to publish otherwise.
3. Open **API Tokens** and tap **New Token**.
4. Give it a name, for example "GitHub Actions", and optionally an expiration date.
5. Select the scopes **publish-new** (needed the first time a crate name is published) and **publish-update** (needed for new versions).
6. Tap **Generate Token** and copy it: crates.io shows it only once.

### Store it in each repository

For each repository (Memoire, Securite, Vitesse, Active, Paquets):

1. On the GitHub website, open the repository, then **Settings** → **Secrets and variables** → **Actions**.
2. Tap **New repository secret**.
3. **Name**: `CARGO_REGISTRY_TOKEN`. **Secret**: the token. Tap **Add secret**.

> [!WARNING]
> The token can publish crates in your name. Keep it in GitHub's secrets only: never in a file, a commit or a message. If it leaks, revoke it in crates.io's **API Tokens** page and create a new one.

## Publish a package from your phone

The steps are the same in every repository.

1. Check that the version in `Cargo.toml` is the one you want to publish (`0.1.0` for a first release), and that the CI is green on `master`.
2. Open the repository's **Releases** page (`https://github.com/<owner>/<repository>/releases`) and tap **Draft a new release**.
3. Tap **Choose a tag**, type the version with a `v`, for example `v0.1.0`, and select **Create new tag: v0.1.0 on publish**. Keep `master` as the target.
4. Give the release a title (for example `v0.1.0`) and notes; **Generate release notes** fills them from the merged changes.
5. Tap **Publish release**.

The workflow starts within seconds. Follow it in the **Actions** tab (the GitHub app works). When it turns green, the crate is on crates.io, and docs.rs builds its documentation a few minutes later.

> [!NOTE]
> The tag must match the version in `Cargo.toml`: tag `v0.1.0` for version `0.1.0`. Paquets', Vitesse's, Securite's and Memoire's workflows stop with an error otherwise; Active's workflow takes the version from the tag.

## Publishing Paquets

Once the four packages are on crates.io, publish Paquets the same way (tag `v0.1.0`). Its **Release** workflow:

1. checks that the tag matches the version of `Cargo.toml`;
2. checks that `memoire-orm-derive`, `memoire-orm`, `securite`, `vitesse` and `active` are on crates.io, and stops with the list of the missing ones otherwise;
3. runs the tests;
4. publishes `paquets-macros` (unless this version is already there), then `paquets`;
5. writes a link to the crate in the run's summary.

To check everything without publishing, run it by hand: **Actions** → **Release** → **Run workflow**, with **Only check** left on. It runs a dry run instead of the publication.

Then:

- `cargo install paquets --locked` installs the command;
- `paquets = "0.1"` works in any project. Projects created with `paquets new` keep using the GitHub repository (`version = "0.1", git = ...`) until you simplify their `Cargo.toml` to `paquets = "0.1"`.

## A new version

crates.io never accepts the same version twice: every release needs a new version number. Follow [semantic versioning](https://semver.org): `0.1.0` → `0.1.1` for fixes, `0.1.1` → `0.2.0` for new features or, before 1.0, breaking changes.

In the Paquets repository, four places change:

| File | Change |
|---|---|
| `Cargo.toml` | `version = "0.1.1"` in `[package]` |
| `macros/Cargo.toml` | the same `version = "0.1.1"` |
| `Cargo.toml` | the pin of the macros: `paquets-macros = { version = "=0.1.1", path = "macros" }` |
| `CHANGELOG.md` | a section for the new version |

From a phone, edit each file on the GitHub website (pencil icon), on a new branch: the CI checks it and merges it into `master`. Then publish a release with the new tag (`v0.1.1`). If the new version of Paquets needs a new version of one of the packages, publish that package first.

## If something goes wrong

| Message | Cause | Fix |
|---|---|---|
| `Publish these packages on crates.io first…` | one of the four packages isn't published yet | publish it from its repository, then run Paquets' release again |
| `no matching package named 'vitesse' found` (or another package) | the same, seen by `cargo publish` | the same |
| `The tag (…) does not match the version of Cargo.toml` | the tag and the version differ | delete the release, fix the version or the tag, publish again |
| `Add the CARGO_REGISTRY_TOKEN secret` or `401`/`403` from crates.io | the token is missing, wrong, expired, or lacks a scope | create a new token with publish-new and publish-update, update the secret |
| `crate version … is already uploaded` | this version is on crates.io | bump the version |
| `the remote server responded with an error: … verified email` | your crates.io email isn't verified | verify it in crates.io's Account Settings |
| `crate name is already taken` | someone else owns this name on crates.io | in a fork, publish under another name |

> [!WARNING]
> A published version can never be changed or deleted. If a release is broken, publish a fixed version, then *yank* the broken one with `cargo yank paquets --version 0.1.1` (from a computer): projects that already use it keep working, new projects won't select it.

## Next steps

- [Automation](automation.md): the CI that checks the packages before every release.
- [The four packages](packages.md): what each package does.
- [Installation](installation.md): what users do once Paquets is on crates.io.
