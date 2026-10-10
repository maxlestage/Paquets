# Tests every push. When everything is green, the branch is merged into master
# automatically (branches named wip/... are not), then master is deployed to
# Heroku by heroku.yml.

name: CI

on:
  push:
  pull_request:
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - name: Islands (WebAssembly)
        if: hashFiles('src/lib.rs') != ''
        run: cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings
      - run: cargo test

  merge:
    name: Merge into master
    needs: test
    if: >-
      github.event_name == 'push' && github.ref_name != 'master' &&
      !startsWith(github.ref_name, 'wip/')
    runs-on: ubuntu-latest
    permissions:
      contents: write
      actions: write
    concurrency:
      group: merge-into-master
      cancel-in-progress: false
    steps:
      - uses: actions/checkout@v4
        with:
          ref: master
          fetch-depth: 0
      - name: Merge ${{ github.ref_name }}
        env:
          BRANCH: ${{ github.ref_name }}
          SHA: ${{ github.sha }}
        run: |
          set -euo pipefail
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          if git merge-base --is-ancestor "$SHA" HEAD; then
            echo "$BRANCH is already in master."
            exit 0
          fi
          git merge --no-ff --no-edit "$SHA" -m "Merge branch '$BRANCH' into master (auto-merge)"
          git push origin HEAD:master
      # A push made by a workflow starts no other workflow: start the deployment.
      - name: Deploy master
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh workflow run heroku.yml --ref master --repo "$GITHUB_REPOSITORY"
