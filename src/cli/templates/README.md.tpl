# {{title}}

A full-stack Rust app built with [Paquets](https://maxlestage.github.io/Paquets/) 📦:
server, security, database, REST API, pages{{readme_islands}}, installable on phones
(PWA) and usable offline.

## Run it

```sh
paquets dev          # http://localhost:3000
cargo test           # the tests of src/main.rs
```

- `src/main.rs`: the server (pages, the `Todo` model, its REST API at `/api/todos`).
{{readme_lib}}- `public/`: static files, served as is.
- The database is in memory until you set `DATABASE_URL`, e.g. `DATABASE_URL=sqlite://app.db paquets dev`.

## Deploy to Heroku from your phone

1. Push this project to a GitHub repository (the GitHub app works).
2. Open `https://heroku.com/deploy?template=<the address of your repository>` in your phone's
   browser, choose an app name and tap **Deploy app**. Heroku builds the `Dockerfile`.
3. For automatic deploys of `master`: in GitHub, *Settings → Secrets and variables → Actions*, add the
   secret `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key*) and the variable `HEROKU_APP_NAME`.

Every branch pushed to GitHub is tested, then merged into `master` automatically when everything is
green (`.github/workflows/ci.yml`; name a branch `wip/...` to opt out), and `master` is deployed.

Add a database: Heroku → your app → *Resources* → *Heroku Postgres*. Its `DATABASE_URL` is used automatically.

Guide: https://maxlestage.github.io/Paquets/#/docs/deploy-heroku-mobile
