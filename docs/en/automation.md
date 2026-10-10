# Automation

The Paquets repository, and every project created with `paquets new`, comes with GitHub workflows that do the repetitive work: they test every push, **merge every green branch into `master` automatically**, then deploy `master` to Heroku (and, for the Paquets repository, publish the website on GitHub Pages). This page explains each workflow, the rules of the automatic merge, and how to turn any of it off.

## The big picture

```text
push a branch ──▶ CI: format, Clippy, tests…
                    │
                    ├── red   ──▶ nothing else happens: fix the branch and push again
                    │
                    └── green ──▶ the branch is merged into master (unless it is named wip/…)
                                    │
                                    ├──▶ Deploy to Heroku   (if HEROKU_API_KEY and HEROKU_APP_NAME are set)
                                    └──▶ Pages              (Paquets repository: the website)
```

Nothing reaches `master`, or production, without passing the tests. And you never have to open, review or merge a pull request yourself, which makes the whole cycle possible from a phone.

## The workflows

| File | Name in the Actions tab | Runs | Does |
|---|---|---|---|
| `.github/workflows/ci.yml` | **CI** | on every push and pull request, and by hand | checks everything, then merges the branch into `master` |
| `.github/workflows/heroku.yml` | **Deploy to Heroku** | after each automatic merge, on pushes to `master`, by hand | deploys `master` to Heroku (see [Heroku](deploy-heroku-mobile.md#4-automatic-deployments-with-github-actions)) |
| `.github/workflows/pages.yml` | **Pages** | after each automatic merge, on pushes to `master` that change the site or the docs, by hand | publishes the website on GitHub Pages (Paquets repository only) |
| `.github/workflows/release.yml` | **Release** | when a GitHub release is published, by hand | publishes the crates on crates.io (Paquets repository only, see [Publish on crates.io](publish-crates-io.md)) |

A project created with `paquets new` has the first two: `ci.yml` and `heroku.yml`.

## CI: what is checked

### In a project created with paquets new

One `test` job, then the merge:

| Step | Command |
|---|---|
| formatting | `cargo fmt --check` |
| lints, server | `cargo clippy --all-targets -- -D warnings` |
| lints, islands (if `src/lib.rs` exists) | `cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings` |
| tests | `cargo test` |

### In the Paquets repository

Four jobs run in parallel, and the merge waits for all of them:

| Job | Checks |
|---|---|
| **Crate** | formatting; Clippy on the server with the default features and with every feature; Clippy on the browser side (`wasm32`, feature `client`); the tests with the default features and with every feature; the API documentation; the crates.io packages (`paquets-macros` completely, and the file list of `paquets`) |
| **MSRV** | the crate compiles with Rust 1.88, the oldest supported version |
| **paquets new** | generates a full and a minimal project with the command line tool, and runs their own checks (formatting, Clippy, tests, and the islands' Clippy for wasm32) |
| **Website** | the website (a Paquets app in `site/`): formatting, Clippy (server and WebAssembly), its tests (including every page of the documentation, in the three languages, with their links), the build of its islands, and the static export for GitHub Pages |

Any red check stops everything: the branch is not merged.

To run the main checks on your computer before pushing:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings   # projects with islands
cargo test
```

## The automatic merge

When every check of a push is green, the **Merge into master** job merges the branch into `master`:

- only for **pushes** (not for pull requests from forks), on any branch except `master` itself;
- **except branches whose name starts with `wip/`** (`wip/new-design`, `wip/experiment`): they are tested, never merged automatically. Use them for work in progress;
- with a merge commit, `Merge branch '<branch>' into master (auto-merge)`, made by `github-actions[bot]`;
- if the branch is already in `master`, nothing happens;
- merges never run at the same time: they wait for each other.

After the merge, the job starts the deployment workflows itself (`gh workflow run heroku.yml`, and `pages.yml` in the Paquets repository). This is needed because GitHub doesn't start workflows for a push made by a workflow.

### When the merge fails

If `master` has changed in a way that conflicts with your branch, the merge job fails and `master` is left untouched. Bring `master` into your branch, fix the conflicts, and push: the CI runs again and merges. From a phone, the GitHub website can do it in a pull request (**Update branch**, then **Resolve conflicts**).

If the push to `master` is refused (`403` or "protected branch"), see [Branch protection](#branch-protection) below.

## GitHub Pages: the website

The Paquets website (in `site/`) is a Paquets app: pages rendered on the server with active, islands in WebAssembly, and a live demo. For GitHub Pages, which only serves files, the **Pages** workflow builds the islands, then exports every page to static files with the site's own server code:

```sh
cd site
BASE_PATH=/Paquets cargo run --release -- export dist
```

`BASE_PATH` prefixes every link with the repository's path, because GitHub Pages serves the site at `https://<owner>.github.io/<repository>/`. The workflow uploads `site/dist` and publishes it.

One-time setup, possible from a phone: **Settings** → **Pages** → **Build and deployment** → **Source**: **GitHub Actions**. Until then, the workflow is skipped with a notice instead of failing. To publish right away: **Actions** → **Pages** → **Run workflow**.

The same site, with its live demo and REST API, is what the **Deploy to Heroku** button deploys (see [Heroku](deploy-heroku-mobile.md)).

## No Dependabot

The repositories have no Dependabot configuration: no bot opens pull requests to update dependencies. Since every green branch is merged and deployed automatically, a bot's update would reach production without anyone deciding it. Dependencies change only when someone runs `cargo update` and pushes the new `Cargo.lock` on a branch, which the CI tests like any other change.

To check for outdated dependencies from time to time, run `cargo update --dry-run` (or [`cargo outdated`](https://crates.io/crates/cargo-outdated)) on your computer.

## Permissions

The merge job asks for two permissions in the workflow file: `contents: write` (to push to `master`) and `actions: write` (to start the deployments). They work with GitHub's default settings. If your organisation restricts the permissions of workflows, check **Settings** → **Actions** → **General** → **Workflow permissions**.

### Branch protection

If `master` is protected by a rule that requires pull requests or reviews, the bot's push is refused. Either allow GitHub Actions to bypass the rule, or turn the automatic merge off (below) and merge pull requests yourself.

## Turning it off

| To stop… | Do this |
|---|---|
| merging one branch | name it `wip/...` |
| the automatic merge entirely | delete the `merge:` job at the end of `.github/workflows/ci.yml`; the tests keep running |
| a whole workflow | **Actions** → the workflow → **⋯** → **Disable workflow** (GitHub website) |
| the Heroku deployments | delete the `HEROKU_API_KEY` secret or the `HEROKU_APP_NAME` variable: the workflow then skips with a notice |
| the GitHub Pages site | **Settings** → **Pages**, unpublish the site, or disable the **Pages** workflow |
| the crates.io publication | don't publish releases, or delete the `CARGO_REGISTRY_TOKEN` secret |

Disabled workflows can be turned back on from the same menu.

## Next steps

- [Deploy to Heroku from your phone](deploy-heroku-mobile.md): the secret and the variable of the deployments.
- [Publish on crates.io](publish-crates-io.md): the release workflow.
- [Testing](testing.md): write the tests the CI runs.
