# Pages

A `Page` is a complete HTML document, rendered on the server and returned by a handler. It comes with every SEO tag (title, description, Open Graph, Twitter cards…), a built-in theme that looks good without writing CSS, and a Content-Security-Policy that allows exactly what the page contains. This page shows how to build views, use the theme, set SEO tags and work with the security policy.

## A first page

```rust
async fn home(_req: Request) -> Page {
    Page::new("Home")
        .description("My site, in Rust.")
        .body(h1().text("Hello 👋"))
        .body(p().text("Built with Paquets."))
}

app.get("/", home);
```

The browser receives finished HTML:

```html
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Home</title><meta name="description" content="My site, in Rust."/>
<meta property="og:title" content="Home"/><meta property="og:description" content="My site, in Rust."/>
<meta property="og:type" content="website"/><meta name="twitter:card" content="summary"/>
<style>/* the theme */</style>...</head>
<body><h1>Hello 👋</h1><p>Built with Paquets.</p></body></html>
```

The response also carries a `Content-Security-Policy` header computed for this page, and the security headers of the shield.

## Page methods

| Method | Effect |
|---|---|
| `Page::new(title)` | a page with its `<title>` (also used for Open Graph and Twitter) |
| `.description(text)` | `<meta name="description">`, also used for Open Graph and Twitter |
| `.lang("fr")` | `<html lang="fr">` (`en` by default) |
| `.body(view)` | adds content to the `<body>`; call it as many times as you like |
| `.seo(Seo::new()...)` | more SEO tags: canonical URL, image, `noindex`, languages, JSON-LD… |
| `.style(css)` | CSS written in the page, allowed by its security policy |
| `.stylesheet(href)` | a `<link rel="stylesheet">`, for example a file of `public/` |
| `.head(html)` | trusted HTML added to `<head>` (an icon, a preload…) |
| `.plain()` | removes the built-in theme |
| `.status(404)` | the HTTP status (`200` by default) |
| `.render()` | the HTML as a `String`, without sending it |

## Building views

Views are built with **one function per HTML tag**, all in the prelude: `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`, `table()`, `img()`… Each returns an element that you configure with chained methods:

```rust
let card = div()
    .class("card")
    .child(h2().text("Fast"))
    .child(p().text("Rendered on the server."))
    .child(a().href("/docs").text("Read the docs"));
```

| Method | Effect |
|---|---|
| `.text(s)` | adds text, always escaped: `"<b>"` is shown, never interpreted |
| `.child(view)` | adds one child: an element, a text, an `Option` (nothing for `None`)… |
| `.children(iter)` | adds several children, typically from `.map(...)` |
| `.class("a b")` | adds classes; call it again to add more |
| `.id("x")`, `.href(url)`, `.style(css)` | the common attributes |
| `.attr(name, value)` | any attribute: `.attr("type", "email")` |
| `.html(raw)` | raw HTML, not escaped: only for HTML you trust |

Lists come from iterators, and conditions from `Option`:

```rust
async fn todos_page(req: Request) -> Result<Page> {
    let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
    let empty = todos.is_empty();
    Ok(Page::new("Todos").body(
        section()
            .child(h1().text("My todos"))
            .child(empty.then(|| p().class("muted").text("Nothing to do!")))
            .child(ul().children(todos.into_iter().map(|t| li().text(t.title)))),
    ))
}
```

Text arguments accept `&'static str` and `String`. A borrowed `&str` must become a `String` first: `name.to_string()` or `format!(...)`.

### Components are functions

A component is a plain Rust function that returns a view. Its arguments are its props:

```rust
/// A card with a title, a text and an optional link.
fn card(title: &str, body: &str, link: Option<&str>) -> Element {
    article()
        .class("card")
        .child(h2().text(title.to_string()))
        .child(p().text(body.to_string()))
        .child(link.map(|href| a().href(href.to_string()).text("Read more")))
}

fn features() -> Element {
    div()
        .class("grid")
        .child(card("Fast", "Rendered on the server.", None))
        .child(card("Secure", "A shield in front of every request.", Some("/security")))
}
```

Return `Element` when the component is one element (the caller can keep chaining `.class(...)`), and `Node` when it can be several siblings or nothing. Turn an `Element` into a `Node` with `.into()`.

### A shared layout

Write the parts every page shares once, in a function that returns a `Page`:

```rust
fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .head(r#"<link rel="icon" href="/favicon.svg"/>"#)
        .body(
            header()
                .child(a().href("/").child(strong().text("📦 My site")))
                .child(nav().child(a().href("/blog").text("Blog")).child(a().href("/about").text("About"))),
        )
        .body(main().child(content))
        .body(footer().text("© My site"))
}

async fn about(_req: Request) -> Page {
    layout("About", p().text("We make things in Rust."))
        .description("Who we are.")
}
```

### Build the page at the end of the handler

Views live on the thread that renders them, so they cannot be kept across an `.await`. Do every `.await` (database, files…) first, then build the page. If you create a view before an `.await`, the compiler stops you with `future cannot be sent between threads safely`. The examples on this page all follow this order.

## The main tag in main.rs

The `<main>` element is created with `main()`. In a file that defines the program's `fn main` (usually `src/main.rs`), that name is taken by your function, and `main().child(...)` fails with an error such as `no method named child found for enum Result`. In that file, write `html::main()` instead:

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Home").body(html::main().child(h1().text("Hello 👋")))
    });
    app.run()
}
```

In every other file (modules, `src/lib.rs`, tests), `main()` works as expected.

## The built-in theme

Every page includes a small theme (about 3.5 kB): readable typography, buttons, forms, tables, and a light or dark mode that follows the visitor's system settings. Plain HTML already looks good, and a few classes cover common layouts:

| Class or element | Look |
|---|---|
| `header`, `nav`, `footer` | a row of items; the header has a bottom border, the footer a top border |
| `.hero` | a large centered introduction, for the top of a home page |
| `.card` | a box with a border and rounded corners |
| `.grid` | a responsive grid: as many columns as fit (at least 15rem each) |
| `.row` | items side by side, wrapping on small screens |
| `.stack` | children spaced vertically |
| `.center` | centered text |
| `.muted` | grey, secondary text |
| `button`, `.button` | a rounded button in the accent colour; `.button` makes a link look like one |
| `.ghost` | a transparent button with a border, for secondary actions |
| `input`, `select`, `textarea`, `label` | full-width fields with a focus ring |
| `table` | full width, with lines between rows |

A home page that uses them:

```rust
async fn home(_req: Request) -> Page {
    Page::new("Acme")
        .description("Furniture made in Rust.")
        .body(
            section()
                .class("hero")
                .child(h1().text("Furniture, made simple"))
                .child(p().text("Oak, steel and a little bit of Rust."))
                .child(
                    div()
                        .class("row")
                        .style("justify-content:center")
                        .child(a().class("button").href("/shop").text("Visit the shop"))
                        .child(a().class("button ghost").href("/about").text("About us")),
                ),
        )
        .body(
            div()
                .class("grid")
                .child(div().class("card").child(h3().text("Chairs")).child(p().class("muted").text("From 49 €")))
                .child(div().class("card").child(h3().text("Tables")).child(p().class("muted").text("From 199 €")))
                .child(div().class("card").child(h3().text("Lamps")).child(p().class("muted").text("From 29 €"))),
        )
}
```

### Changing the colours

The theme is built on CSS variables. Override them with `.style(...)`:

```rust
Page::new("Home")
    .style(":root { --accent: #e11d48; --radius: 6px; }")
    .body(h1().text("Red buttons, sharper corners"))
```

| Variable | Role |
|---|---|
| `--bg`, `--fg` | background and text colours |
| `--muted` | secondary text |
| `--line` | borders |
| `--card` | background of cards, fields and code blocks |
| `--accent`, `--accent-fg` | buttons and links, and the text on buttons |
| `--radius` | corner radius of cards |
| `--font`, `--mono` | the text and code fonts |

The dark mode redefines `--bg`, `--fg`, `--muted`, `--line`, `--card` and `--accent` inside `@media (prefers-color-scheme: dark)`.

### Your own CSS

- For a few rules, use `.style(css)`: the CSS is written in the page and allowed by its policy.
- For a site-wide stylesheet, put it in `public/` and link it: `.stylesheet("/site.css")`. Browsers cache it between pages.
- To start from a blank page, call `.plain()`: the theme is not included, only your styles.

The theme's CSS is also available as the constant `paquets::THEME`, if you want to reuse it elsewhere.

## SEO tags

`Page::new(title)` and `.description(...)` already produce the title, the description, and the Open Graph and Twitter tags that search engines and social networks read. Add more with `.seo(...)`:

```rust
async fn article(_req: Request) -> Page {
    Page::new("Our new oak chair")
        .description("Solid oak, designed to last fifty years.")
        .seo(
            Seo::new()
                .base_url("https://acme.example")
                .canonical("/blog/oak-chair")
                .image("/images/oak-chair.jpg")
                .og_type("article")
                .site_name("Acme")
                .title_template("%s · Acme")
                .alternate("fr", "/fr/blog/chaise-chene")
                .json_ld(r#"{"@context":"https://schema.org","@type":"Article","headline":"Our new oak chair"}"#),
        )
        .body(h1().text("Our new oak chair"))
}
```

| `Seo` method | Tag |
|---|---|
| `.canonical(url)` | `<link rel="canonical">`: the official address of the page |
| `.image(url)` | `og:image` and a large Twitter card |
| `.base_url(origin)` | turns relative URLs (canonical, image, alternates) into absolute ones |
| `.site_name(name)` | `og:site_name` |
| `.title_template("%s · Acme")` | the final `<title>`, with `%s` replaced by the page title |
| `.noindex(true)`, `.nofollow(true)` | `<meta name="robots">`, to keep a page out of search engines |
| `.alternate(lang, url)` | `<link rel="alternate" hreflang>`, once per language |
| `.json_ld(json)` | structured data for rich search results |
| `.og_type(...)`, `.locale(...)`, `.twitter(handle)` | the other Open Graph and Twitter tags |
| `.meta_name(name, content)`, `.meta_property(...)` | any other `<meta>` tag |

A `Seo` can also be placed inside a view (`.child(Seo::new().title(...))`): every `Seo` of the page is merged, the deepest value winning, without duplicate tags. A layout can therefore set site-wide values that each page completes. See [Active's SEO guide](https://maxlestage.github.io/Active/en/docs/seo/).

## The Content-Security-Policy

Every page is sent with a `Content-Security-Policy` header that tells the browser what the page may load and run. Paquets computes it from the page itself:

| Allowed | Blocked |
|---|---|
| files from your own site (`/site.css`, `/app.js`, images, fonts) | scripts, stylesheets and fonts from other sites (CDNs, Google Fonts) |
| the page's own `<style>` blocks and inline `<script>` tags, by their SHA-256 hash | any inline script or style that was not in the page when it was rendered |
| `style="..."` attributes | inline event handlers such as `onclick="..."` |
| images from your site, from `https:` URLs and `data:` URLs | being shown in a frame on another site |
| your islands' WebAssembly | |

Even if an attacker managed to slip some HTML into a page (through a comment field, for example), their script would not run.

Static HTML files of `public/` (a landing page, or a site exported by a static site generator) are covered too: at startup, the default shield reads them and its policy allows their inline scripts and styles (by hash), the external stylesheets and scripts they link to (Google Fonts included, with its font files), WebAssembly and `https:` images. This automatic policy belongs to the default shield: if you replace the shield with `app.security(...)`, set the policy yourself (see [Security](security.md#security-headers-and-the-content-security-policy)).

The simplest way to use a font or a library from another site is to **download it into `public/`** and link it from there: it is also faster and better for your visitors' privacy. If you really need an external source, add it to the policy of that response:

```rust
app.get("/", |_| async {
    let res = Page::new("Home")
        .stylesheet("https://fonts.googleapis.com/css2?family=Inter")
        .body(h1().text("Hello"))
        .into_response();
    // Allow Google Fonts in this page's policy.
    let csp = res
        .get_header("content-security-policy")
        .unwrap_or_default()
        .replace("style-src 'self'", "style-src 'self' https://fonts.googleapis.com")
        .replace("font-src 'self'", "font-src 'self' https://fonts.gstatic.com");
    res.header("content-security-policy", csp)
});
```

## Forms

A form is a view like any other. Point its `action` to a route that reads it with `req.form()`:

```rust
fn contact_form() -> Element {
    form()
        .attr("method", "post")
        .attr("action", "/contact")
        .child(label().attr("for", "email").text("Your email"))
        .child(input().attr("type", "email").attr("name", "email").id("email"))
        .child(label().attr("for", "message").text("Message"))
        .child(textarea().attr("name", "message").id("message"))
        .child(button().text("Send"))
}
```

No CSRF token is needed: the shield refuses forms posted from other websites. The handler side is in [Requests and responses](requests-responses.md#forms).

## Interactive parts

A page is static HTML. To make part of it react in the browser (a counter, a search box, a cart), make that part an **island**: `island("counter", "0", counter)`. Only pages that contain islands load the browser bundle. See [Islands](islands.md).

## Installable app (PWA)

One line makes the app installable on phones and computers, and usable offline:

```rust
app.pwa(
    Pwa::new("My todos")
        .short_name("Todos")
        .theme_color("#6d4aff")
        .background_color("#0b0b12"),
);
```

- The app serves the web app manifest at `/manifest.webmanifest` and a service worker at `/sw.js`; every `Page` links them (with the theme color, the home screen title and the Apple touch icon), and the page's policy allows the registration script.
- Offline: the start page, the islands bundle and the icons are downloaded at installation. Pages come from the network first, then from the cache when the network is down; the REST APIs of `app.api` always come from the network.
- Without icons, the Paquets box is used. Your own icons (at least a 192×192 and a 512×512 PNG in `public/`): `.icon("/icon-192.png", "192x192", "image/png").icon("/icon-512.png", "512x512", "image/png").maskable_icon("/icon-maskable.png", "512x512", "image/png")`.
- `.start_url("/{lang}/")` gives a site in several languages one manifest per language (`/fr/manifest.webmanifest`, linked by the pages whose `Page::lang` is `fr`): the installed app opens in the language of the page it was installed from.
- `.start_url("/demo")` opens another page; `.description(...)`, `.lang("fr")`, `.display("fullscreen")`… complete the manifest ([Active's `pwa` module](https://docs.rs/active/latest/active/pwa/index.html) builds it).
- Browsers only install a PWA served over HTTPS (or `localhost`): Heroku gives every app HTTPS. On a phone, open the site, then *Add to Home Screen* (Safari's share menu) or *Install app* (Chrome's menu).

## Learn more

Views come from [Active](https://maxlestage.github.io/Active/). Its documentation covers [components and views](https://maxlestage.github.io/Active/en/docs/components/), [SEO](https://maxlestage.github.io/Active/en/docs/seo/) and [structured data](https://maxlestage.github.io/Active/en/docs/structured-data/) in depth.

## Next steps

- [Database](database.md): fill your pages with data.
- [Islands](islands.md): interactive parts in Rust.
- [Security](security.md): the shield and the headers it adds.
