# {{title}}

A full-stack Rust app built with [Paquets](https://paquets-90dfc06b6fea.herokuapp.com/) 📦:
server, security, database, REST API, pages{{readme_islands}}, installable on phones
(PWA) and usable offline.

## Run it

```sh
paquets dev          # http://localhost:3000
cargo test           # the tests of src/main.rs
```

- `src/main.rs`: the server (pages, the `Todo` model, its REST API at `/api/todos`).
{{readme_lib}}- `public/`: static files, served as is.
- `data/`: the database in development, [Nuage DB](https://github.com/maxlestage/Nuage)'s local
  mode (`app.nuage("data")` in `src/main.rs`). Tests use a fresh in-memory database.

## Deploy to Heroku from your phone

1. Push this project to a GitHub repository (the GitHub app works).
2. Open `https://heroku.com/deploy?template=<the address of your repository>` in your phone's
   browser, choose an app name and tap **Deploy app**. Heroku builds the `Dockerfile`.
3. For automatic deploys of `master`: in GitHub, *Settings → Secrets and variables → Actions*, add the
   secret `HEROKU_API_KEY` (Heroku → *Account settings* → *API Key*) and the variable `HEROKU_APP_NAME`.

Every branch pushed to GitHub is tested, then merged into `master` automatically when everything is
green (`.github/workflows/ci.yml`; name a branch `wip/...` to opt out), and `master` is deployed.

The database in production: a Nuage DB server (https://github.com/maxlestage/Nuage deploys on
Heroku as is). Set the config vars `NUAGE_URL` (its address, `https://…`) and `NUAGE_TOKEN` (its
token): the same code then uses the server. `NUAGE_BRANCH=preview` runs on a branch of it, made
in an instant. Without `NUAGE_URL`, the data stays on the dyno's disk, erased at each restart.
`DATABASE_URL` (Heroku Postgres…) still works when `NUAGE_URL` is not set.

Nuage DB guide: https://paquets-90dfc06b6fea.herokuapp.com/en/docs/nuage/

Guide: https://paquets-90dfc06b6fea.herokuapp.com/en/docs/deploy-heroku-mobile/
