# Deploy to Heroku from your phone

You don't need a computer to put a Paquets app online. This guide goes through every step from a phone: a one-tap deployment with the **Deploy to Heroku** button, a PostgreSQL database, automatic deployments on every change with GitHub Actions, then the day-to-day: redeploying, reading the logs, adding a domain name, fixing problems. The same steps work for the Paquets repository and for your own project created with `paquets new`.

> [!NOTE]
> Every step works in a phone's web browser (Safari, Chrome, Firefox…). The GitHub app is handy for the **Actions** and **Releases** tabs, but a few settings (secrets, variables, file editing) are only on the GitHub website: open https://github.com in your browser for those. If a menu seems to be missing on github.com, switch your browser to its "Desktop site" mode. Heroku and GitHub sometimes rename their menus: if a label differs slightly, look for the closest one.

## Before you start

You need:

- **A Heroku account**: sign up at https://signup.heroku.com. Heroku has **no free plan**: the cheapest dynos (the containers that run your app) are **Eco**, about **$5 a month**, and **Basic**, about $7 a month. Heroku asks for a payment method before you can create an app. Prices change: check https://www.heroku.com/pricing.
- **A GitHub account**, for your own copy of the code and for automatic deployments (sections 3 and 4).
- Optionally, a few dollars a month more for a **Heroku Postgres** database (section 2).

### How it works

Everything Heroku needs is already in the repository, so nothing is compiled on your phone: Heroku builds the app on its own servers.

In the Paquets repository, Heroku builds the Paquets website, which is itself a Paquets app (pages rendered on the server, islands in WebAssembly), with a live demo at `/demo` and a REST API at `/api/todos`. By default, it uses the [**emk/rust** buildpack](https://elements.heroku.com/buildpacks/emk/heroku-buildpack-rust) on the **heroku-26** stack:

| File | Role |
|---|---|
| `app.json` | describes the app for the Deploy button (name, description, the heroku-26 stack, the emk/rust buildpack) |
| `RustConfig` | configures the buildpack: the Rust version, and instead of its own build, `heroku-build.sh` |
| `heroku-build.sh` | the build, run once the buildpack has installed Rust: compiles the server (`paquets-site`) in release mode, in `site/`, then the islands (WebAssembly) into `site/pkg` |
| `Procfile` | the command that starts the website |
| `heroku.yml` and `Dockerfile` | the other way: a Docker image, on the "container" stack (`HEROKU_BUILD=docker`, see section 4) |
| `.github/workflows/heroku.yml` | the **Deploy to Heroku** workflow, for automatic deployments |
| `.github/workflows/ci.yml` | the **CI** workflow: tests every push, then merges the branch into `master` |

A project created with `paquets new` is built from its `Dockerfile`: its `app.json` chooses the "container" stack, and its `heroku.yml` tells Heroku to build the image from the `Dockerfile`, which compiles your app (`src/main.rs` and the islands of `src/lib.rs`) in release mode and packages it in a small image.

## 1. The fastest way: the Deploy to Heroku button

1. Tap the button above (it is also in the repository's README), or open this address: `https://dashboard.heroku.com/new-app`. Log in to Heroku if asked.
2. Heroku shows a form to create the app:
   - **App name**: for example `my-paquets`. Names are unique across all of Heroku, in lowercase letters, digits and dashes. You can leave it empty and let Heroku choose.
   - **Choose a region**: United States or Europe.
3. Tap **Deploy app**. Heroku fetches the code and builds the app (with the buildpack in the Paquets repository, from the `Dockerfile` in a project created with `paquets new`); the build log scrolls on the page. The first build takes a while (often 10 to 20 minutes): Rust compiles the app and its WebAssembly bundle with every optimisation turned on.
4. When the build is done, tap **View**. The site opens at `https://<app-name>.herokuapp.com` (the exact address is also in the app's **Settings** tab on Heroku).

Try the demo by adding these paths to the address:

| Path | What you see |
|---|---|
| `/demo` | a to-do list rendered on the server, with a form |
| `/api/todos` | the same to-dos as JSON |

> [!NOTE]
> Until you add a database, the app keeps its data in memory: it is lost at every restart (each deployment, each settings change, and the automatic restart Heroku performs about once a day). Section 2 fixes that.

## 2. Add a database

1. Open https://dashboard.heroku.com and tap your app.
2. Open the **Resources** tab.
3. In **Add-ons**, search for **Heroku Postgres** and select it.
4. Choose a plan (the smallest one, Essential-0, costs about $5 a month at the time of writing) and confirm with **Submit Order Form**.

That's all. Heroku creates the database, sets the `DATABASE_URL` config var, and restarts the app. Paquets reads `DATABASE_URL` at startup, connects with TLS, and creates the tables of your models if they don't exist yet. To check, open the logs (**More** → **View logs**): the startup line now says `database: postgres`.

```text
📦 Paquets · http://localhost:21345
   database: postgres · security: on (10 guards) · islands: /pkg/paquets_site.js
```

(The address in this line is the inside of the container: your app is reached through `https://<app-name>.herokuapp.com`.)

> [!WARNING]
> Don't use SQLite on Heroku: the file system of a dyno is temporary, and the file would be erased at every restart.

## 3. Your own copy of the code

To change the code, you need your own copy of the repository on GitHub:

1. Open https://github.com/maxlestage/Paquets (the GitHub app works), tap **Fork**, then **Create fork**.
2. Deploy your fork by typing this address in your browser, with your GitHub user name instead of `<you>`:

   ```text
   https://heroku.com/deploy?template=https://github.com/<you>/Paquets
   ```

3. Follow the steps of section 1.

> [!IMPORTANT]
> The button deploys **once**. When you change your fork later, the Heroku app is not updated by itself. Set up automatic deployments (section 4), with the **same app name**: the workflow updates the app you just created.

## 4. Automatic deployments with GitHub Actions

Goal: every change merged into `master` rebuilds and redeploys the app, with nothing to do. The **Deploy to Heroku** workflow talks directly to the Heroku Platform API: no Heroku CLI, no computer. On each run, it:

1. checks that it is configured (if not, it stops without failing);
2. creates the Heroku app if it doesn't exist yet, and chooses how Heroku builds it: in the Paquets repository, the heroku-26 stack and the emk/rust buildpack (or the "container" stack with `HEROKU_BUILD=docker`); in a project created with `paquets new`, the "container" stack;
3. sends the code to Heroku, which builds it (with the buildpack, or the image from `heroku.yml`), and shows the build log live in GitHub;
4. writes the app's address in the run's summary.

### Step 1: get a Heroku API key

1. In your browser, open https://dashboard.heroku.com and log in.
2. Open **Account settings** (from the menu or your avatar, at the top right).
3. In the **API Key** section, tap **Reveal**, then copy the key.

For a key that you can revoke on its own, open the **Applications** tab of Account settings instead, tap **Create authorization** under **Authorizations**, give it a description such as "GitHub Actions", and copy the token.

> [!WARNING]
> This key acts on your whole Heroku account. Never paste it in a file, a commit, an issue or a message: only in GitHub's secrets, as described next.

### Step 2: add the secret and the variable on GitHub

1. On https://github.com, open **your repository** (your fork, or your own project) and tap **Settings** (in the tab bar of the repository; scroll it sideways if needed).
2. Open **Secrets and variables**, then **Actions**.
3. In the **Secrets** tab, tap **New repository secret**:
   - **Name**: `HEROKU_API_KEY`
   - **Secret**: paste the key
   - tap **Add secret**.
4. In the **Variables** tab, tap **New repository variable**:
   - **Name**: `HEROKU_APP_NAME`
   - **Value**: the name of your Heroku app (the one created with the button, or a new name: the workflow creates the app if needed)
   - tap **Add variable**.
5. Optional: a variable `HEROKU_REGION` with the value `eu` creates the app in Europe. It is only used when the workflow creates the app.
6. Optional, in the Paquets repository: a variable `HEROKU_BUILD` with the value `docker` builds the Docker image (`heroku.yml` and the `Dockerfile`, "container" stack) instead of the buildpack. With `buildpack`, or without the variable, the workflow uses the emk/rust buildpack. It switches the app's stack itself when you change this value.

| Name | Kind | Required | Value |
|---|---|---|---|
| `HEROKU_API_KEY` | secret | yes | your Heroku API key |
| `HEROKU_APP_NAME` | variable | yes | for example `my-paquets` |
| `HEROKU_REGION` | variable | no | `us` (default) or `eu` |
| `HEROKU_BUILD` | variable | no | `buildpack` (default) or `docker` (Paquets repository) |

### Step 3: turn on Actions in a fork

GitHub turns off workflows in a new fork. Open the **Actions** tab of your fork: if a message says that workflows aren't running on this fork, tap the button that turns them on. (Your own repositories don't need this.)

### Step 4: run the first deployment

1. Open the **Actions** tab (in the GitHub app or the website) and choose **Deploy to Heroku** in the list of workflows.
2. Tap **Run workflow**, keep the `master` branch, and confirm with **Run workflow**.
3. Tap the run that appears to follow it. The first build takes several minutes.
4. When it turns green, the run's summary shows the app's address: `🚀 Deployed to https://my-paquets.herokuapp.com`.

From now on, the app is redeployed:

- after every branch that the **CI** workflow merges into `master` (see section 5 and [Automation](automation.md));
- after every direct commit to `master` (a commit that only changes Markdown files doesn't trigger it in the Paquets repository);
- whenever you run the workflow by hand.

> [!NOTE]
> As long as the secret or the variable is missing, the workflow skips the deployment with a notice ("Heroku is not configured"), instead of failing.

### Without GitHub Actions

For the Paquets repository, an app created another way (not by the button or the workflow) must use the buildpack:

- **Heroku dashboard**: your app → **Settings** → **Buildpacks** → **Add buildpack**, type `emk/rust` and save. The app's stack must not be "container" (that stack builds with `heroku.yml` instead).
- **Heroku CLI** (on a computer, or in a cloud environment such as GitHub Codespaces), from a clone of the repository:

  ```sh
  heroku create my-paquets --stack heroku-26 --buildpack emk/rust
  git push heroku master
  ```

  For an existing app: `heroku stack:set heroku-26`, then `heroku buildpacks:set emk/rust`, then `git push heroku master`.

## 5. Change the code from your phone

1. On the GitHub website, in your repository, open the file to change: for your own project, `src/main.rs`.
2. Tap the **pencil** icon (✏️ "Edit this file") and make your change. For example, change the text of the home page:

   ```rust
   .child(h1().text("Hello from my phone 👋"))
   ```

3. Tap **Commit changes…**. Choose **Create a new branch for this commit**, give it a name such as `new-title`, and confirm (**Propose changes**). You don't need to open the pull request.
4. The **CI** workflow starts (follow it in the **Actions** tab): formatting, Clippy, tests. When everything is green, it merges your branch into `master` by itself and starts the deployment. A few minutes later, the new version is online.

If the CI fails, open the failed run: the compiler's message gives the file, the line and the column. Fix the file on the same branch: the CI runs again. Nothing reaches `master`, or Heroku, until the CI is green.

> [!TIP]
> You can also commit directly to `master`: the deployment starts at once, but without waiting for the tests. A branch is safer. A branch whose name starts with `wip/` is tested but never merged automatically, for work in progress.

Heroku only releases builds that succeed: if a build fails, the previous version stays online.

## 6. Your own project, created with paquets new

A project created with `paquets new my-app` contains `app.json`, `heroku.yml`, a `Dockerfile` and both workflows, and is built from its `Dockerfile` ("container" stack). Every step of this guide applies to it, except the buildpack and `HEROKU_BUILD`, which are specific to the Paquets repository.

Creating the project needs Rust, once: on any computer, or in a cloud development environment that runs in a browser, such as GitHub Codespaces. Run `paquets new my-app`, then publish the folder as a new GitHub repository (for example `gh repo create my-app --private --source=my-app --push`). From then on, everything works from your phone:

- **First deployment**: the Deploy button with your repository's address, `https://heroku.com/deploy?template=https://github.com/<you>/my-app` (for a private repository, use section 4 instead: the button needs a public repository).
- **Automatic deployments**: the secret `HEROKU_API_KEY` and the variable `HEROKU_APP_NAME`, as in section 4.
- **A database**: the Heroku Postgres add-on, as in section 2.

The project's `README.md` repeats these steps.

## 7. Day-to-day operations

### Redeploy by hand

**Actions** → **Deploy to Heroku** → **Run workflow** → **Run workflow**. This works in the GitHub app. Useful after changing a config var that needs a fresh build, or to retry a deployment.

### Read the logs

Heroku dashboard → your app → **More** (top right) → **View logs**. You see Heroku's own messages (starts, stops, crashes, one line per request from its router) and everything your app prints:

```text
app[web.1]: 📦 Paquets · http://localhost:21345
app[web.1]:    database: postgres · security: on (10 guards) · islands: /pkg/my_app.js
app[web.1]: GET /api/todos 200 0.412 ms
heroku[router]: at=info method=GET path="/api/todos" host=my-app.herokuapp.com status=200 ...
```

Errors of your handlers that become a `500` are logged with their cause (`[vitesse] error 500: ...`). The **Activity** tab lists builds and releases, with a link to each build log.

### Settings: config vars

**Settings** → **Reveal Config Vars**. Each config var is an environment variable, read in your code with `std::env::var("NAME")`. Saving a change restarts the app. `DATABASE_URL` is managed by the Postgres add-on. Don't set `PORT`: Heroku sets it at each start.

### Restart, sleep, dyno type

- **Restart**: **More** → **Restart all dynos**.
- **Sleep**: an Eco dyno goes to sleep after about 30 minutes without visits; the next visit wakes it up in a few seconds. A Basic dyno never sleeps.
- **Dyno type**: in the **Resources** tab, edit the `web` line (pencil icon) to change the type; the price changes accordingly.

### A custom domain

1. **Settings** → **Domains** → **Add domain**, and type your domain, for example `www.example.com`.
2. Heroku gives you a **DNS target**. At your domain registrar, create a `CNAME` record for `www` pointing to that target. A root domain (`example.com`) needs a registrar that supports `ALIAS`/`ANAME` records or CNAME flattening.
3. For HTTPS on your domain, turn on Heroku's **Automated Certificate Management** in the same tab. It is not available on every dyno type: see https://devcenter.heroku.com/articles/automated-certificate-management.

### Stop paying

Delete the add-ons in **Resources**, and the app itself in **Settings** → **Delete app**. Removing the `HEROKU_API_KEY` secret stops the automatic deployments.

## 8. Troubleshooting

### The workflow fails before building

| Message | Fix |
|---|---|
| `Heroku is not configured` (a notice) | Add the `HEROKU_API_KEY` secret and the `HEROKU_APP_NAME` variable (section 4, step 2). |
| `Heroku refused the API key (HEROKU_API_KEY).` | The key is wrong, expired or revoked: copy a new one and update the secret. |
| `This Heroku account cannot use the app "…" (the name may be taken).` | The name belongs to another Heroku account: choose another name in `HEROKU_APP_NAME`. |
| creating the app fails | Check that your Heroku account has a payment method, or create the app with the button first and put its name in `HEROKU_APP_NAME`. |

### The build fails

Open the build log, in GitHub (**Actions**, the failed run) or on Heroku (**Activity** → build log), and look for the first line that starts with `error`:

| Message | Cause | Fix |
|---|---|---|
| `error[E…]` with a file and a line | a Rust compilation error | fix the code at that line and commit again |
| `no matching package named …` | a misspelled crate name in `Cargo.toml` | check the name on https://crates.io |
| `The Heroku build ended with status "failed".` | the build failed on Heroku | read the build log just above in the same run |

### "Application error" in the browser

The build succeeded but the app doesn't answer. Open the logs and look for an error code:

| Code | Meaning | What to do |
|---|---|---|
| `H10` App crashed | the program stopped | read the lines just above: `Error: … database: …` means `DATABASE_URL` is wrong or the database is unreachable; `startup: …` means an `on_start` hook failed; `duplicate route` means two routes collide |
| `R10` Boot timeout | the app didn't listen on `$PORT` within 60 seconds | Paquets listens on `$PORT` by itself: don't call `app.port(...)` with a fixed port in production |
| `H14` No web dynos running | no dyno runs the `web` process | in **Resources**, check that the `web` dyno is switched on |
| `H12` Request timeout | a request took more than 30 seconds | make the handler faster, or move the work out of the request |

All the codes are listed at https://devcenter.heroku.com/articles/error-codes.

### Other surprises

- **Every visitor gets `429 Too Many Requests`**: the shield counts requests per IP address. The default shield reads the real address behind Heroku's router; if you replaced it with `app.security(...)`, build it from `paquets::shield()`, which keeps that setting (see [Security](security.md#the-client-ip-behind-a-proxy)).
- **The first visit is slow**: an Eco dyno was asleep.
- **My data disappeared**: the app has no database add-on (data in memory), or uses SQLite. Add Heroku Postgres (section 2).
- **A new field of a model causes `500` errors**: the existing table wasn't changed. See [Database](database.md#changing-a-table-later).
- **WebSocket connections close after a minute**: Heroku closes connections that stay silent for about 55 seconds. Send a ping regularly (see [WebSocket](websocket.md#in-production)).
- **HTTP/3 doesn't work**: Heroku's router doesn't forward UDP. Leave the `http3` feature off on Heroku.

## Next steps

- [Automation](automation.md): the CI, the automatic merge into `master`, and how to turn them off.
- [Docker](deploy-docker.md): the same image on your computer or on other platforms.
- [Configuration](configuration.md): environment variables and settings.
