# Páginas

Una `Page` es un documento HTML completo, renderizado en el servidor y devuelto por un handler. Viene con todas las etiquetas SEO (título, descripción, Open Graph, tarjetas de Twitter…), un tema integrado que se ve bien sin escribir CSS y una Content-Security-Policy que permite exactamente lo que contiene la página. Esta página muestra cómo construir vistas, usar el tema, definir las etiquetas SEO y trabajar con la política de seguridad.

## Una primera página

```rust
async fn home(_req: Request) -> Page {
    Page::new("Inicio")
        .description("Mi sitio, en Rust.")
        .body(h1().text("Hola 👋"))
        .body(p().text("Hecho con Paquets."))
}

app.get("/", home);
```

El navegador recibe el HTML terminado:

```html
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Inicio</title><meta name="description" content="Mi sitio, en Rust."/>
<meta property="og:title" content="Inicio"/><meta property="og:description" content="Mi sitio, en Rust."/>
<meta property="og:type" content="website"/><meta name="twitter:card" content="summary"/>
<style>/* el tema */</style>...</head>
<body><h1>Hola 👋</h1><p>Hecho con Paquets.</p></body></html>
```

La respuesta también lleva una cabecera `Content-Security-Policy` calculada para esta página, y las cabeceras de seguridad del escudo.

## Métodos de Page

| Método | Efecto |
|---|---|
| `Page::new(title)` | una página con su `<title>` (que también se usa para Open Graph y Twitter) |
| `.description(text)` | `<meta name="description">`, que también se usa para Open Graph y Twitter |
| `.lang("fr")` | `<html lang="fr">` (`en` por defecto) |
| `.body(view)` | añade contenido al `<body>`; llámalo tantas veces como quieras |
| `.seo(Seo::new()...)` | más etiquetas SEO: URL canónica, imagen, `noindex`, idiomas, JSON-LD… |
| `.style(css)` | CSS escrito en la página, permitido por su política de seguridad |
| `.stylesheet(href)` | un `<link rel="stylesheet">`, por ejemplo a un archivo de `public/` |
| `.head(html)` | HTML de confianza añadido a `<head>` (un icono, una precarga…) |
| `.plain()` | quita el tema integrado |
| `.status(404)` | el estado HTTP (`200` por defecto) |
| `.render()` | el HTML como `String`, sin enviarlo |

## Construir vistas

Las vistas se construyen con **una función por etiqueta HTML**, todas en el prelude: `div()`, `p()`, `a()`, `ul()`, `li()`, `h1()`, `section()`, `form()`, `input()`, `button()`, `table()`, `img()`… Cada una devuelve un elemento que configuras con métodos encadenados:

```rust
let card = div()
    .class("card")
    .child(h2().text("Rápido"))
    .child(p().text("Renderizado en el servidor."))
    .child(a().href("/docs").text("Lee la documentación"));
```

| Método | Efecto |
|---|---|
| `.text(s)` | añade texto, siempre escapado: `"<b>"` se muestra, nunca se interpreta |
| `.child(view)` | añade un hijo: un elemento, un texto, un `Option` (nada para `None`)… |
| `.children(iter)` | añade varios hijos, normalmente a partir de `.map(...)` |
| `.class("a b")` | añade clases; vuelve a llamarlo para añadir más |
| `.id("x")`, `.href(url)`, `.style(css)` | los atributos habituales |
| `.attr(name, value)` | cualquier atributo: `.attr("type", "email")` |
| `.html(raw)` | HTML en bruto, sin escapar: solo para HTML de confianza |

Las listas salen de iteradores, y las condiciones de `Option`:

```rust
async fn todos_page(req: Request) -> Result<Page> {
    let todos = Todo::query().order_by(Todo::ID.asc()).all(req.db()).await?;
    let empty = todos.is_empty();
    Ok(Page::new("Tareas").body(
        section()
            .child(h1().text("Mis tareas"))
            .child(empty.then(|| p().class("muted").text("¡Nada que hacer!")))
            .child(ul().children(todos.into_iter().map(|t| li().text(t.title)))),
    ))
}
```

Los argumentos de texto aceptan `&'static str` y `String`. Un `&str` prestado debe convertirse antes en `String`: `name.to_string()` o `format!(...)`.

### Los componentes son funciones

Un componente es una función de Rust normal que devuelve una vista. Sus argumentos son sus props:

```rust
/// Una tarjeta con un título, un texto y un enlace opcional.
fn card(title: &str, body: &str, link: Option<&str>) -> Element {
    article()
        .class("card")
        .child(h2().text(title.to_string()))
        .child(p().text(body.to_string()))
        .child(link.map(|href| a().href(href.to_string()).text("Leer más")))
}

fn features() -> Element {
    div()
        .class("grid")
        .child(card("Rápido", "Renderizado en el servidor.", None))
        .child(card("Seguro", "Un escudo delante de cada petición.", Some("/security")))
}
```

Devuelve `Element` cuando el componente es un solo elemento (quien lo llama puede seguir encadenando `.class(...)`), y `Node` cuando puede ser varios hermanos o nada. Convierte un `Element` en un `Node` con `.into()`.

### Un layout compartido

Escribe una sola vez las partes que comparten todas las páginas, en una función que devuelve una `Page`:

```rust
fn layout(title: &str, content: impl Into<Node>) -> Page {
    Page::new(title)
        .head(r#"<link rel="icon" href="/favicon.svg"/>"#)
        .body(
            header()
                .child(a().href("/").child(strong().text("📦 Mi sitio")))
                .child(nav().child(a().href("/blog").text("Blog")).child(a().href("/about").text("Acerca de"))),
        )
        .body(main().child(content))
        .body(footer().text("© Mi sitio"))
}

async fn about(_req: Request) -> Page {
    layout("Acerca de", p().text("Hacemos cosas en Rust."))
        .description("Quiénes somos.")
}
```

### Construye la página al final del handler

Las vistas viven en el hilo que las renderiza, así que no se pueden conservar a través de un `.await`. Haz primero todos los `.await` (base de datos, archivos…) y después construye la página. Si creas una vista antes de un `.await`, el compilador te detiene con `future cannot be sent between threads safely`. Todos los ejemplos de esta página siguen este orden.

## La etiqueta main en main.rs

El elemento `<main>` se crea con `main()`. En un archivo que define la función `fn main` del programa (normalmente `src/main.rs`), ese nombre lo ocupa tu función, y `main().child(...)` falla con un error como `no method named child found for enum Result`. En ese archivo, escribe `html::main()` en su lugar:

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.get("/", |_| async {
        Page::new("Inicio").body(html::main().child(h1().text("Hola 👋")))
    });
    app.run()
}
```

En todos los demás archivos (módulos, `src/lib.rs`, pruebas), `main()` funciona como se espera.

## El tema integrado

Cada página incluye un pequeño tema (unos 3.5 kB): tipografía legible, botones, formularios, tablas y un modo claro u oscuro que sigue los ajustes del sistema del visitante. El HTML sin más ya se ve bien, y unas pocas clases cubren las maquetaciones habituales:

| Clase o elemento | Aspecto |
|---|---|
| `header`, `nav`, `footer` | una fila de elementos; la cabecera tiene un borde inferior y el pie, un borde superior |
| `.hero` | una gran introducción centrada, para la parte superior de una página de inicio |
| `.card` | una caja con borde y esquinas redondeadas |
| `.grid` | una cuadrícula adaptable: tantas columnas como quepan (de al menos 15rem cada una) |
| `.row` | elementos uno al lado del otro, que pasan a la línea siguiente en pantallas pequeñas |
| `.stack` | hijos espaciados verticalmente |
| `.center` | texto centrado |
| `.muted` | texto secundario, en gris |
| `button`, `.button` | un botón redondeado con el color de acento; `.button` hace que un enlace parezca un botón |
| `.ghost` | un botón transparente con borde, para las acciones secundarias |
| `input`, `select`, `textarea`, `label` | campos a todo lo ancho con un anillo de foco |
| `table` | a todo lo ancho, con líneas entre las filas |

Una página de inicio que las usa:

```rust
async fn home(_req: Request) -> Page {
    Page::new("Acme")
        .description("Muebles hechos en Rust.")
        .body(
            section()
                .class("hero")
                .child(h1().text("Muebles, sin complicaciones"))
                .child(p().text("Roble, acero y un poco de Rust."))
                .child(
                    div()
                        .class("row")
                        .style("justify-content:center")
                        .child(a().class("button").href("/shop").text("Visita la tienda"))
                        .child(a().class("button ghost").href("/about").text("Quiénes somos")),
                ),
        )
        .body(
            div()
                .class("grid")
                .child(div().class("card").child(h3().text("Sillas")).child(p().class("muted").text("Desde 49 €")))
                .child(div().class("card").child(h3().text("Mesas")).child(p().class("muted").text("Desde 199 €")))
                .child(div().class("card").child(h3().text("Lámparas")).child(p().class("muted").text("Desde 29 €"))),
        )
}
```

### Cambiar los colores

El tema se basa en variables CSS. Sobrescríbelas con `.style(...)`:

```rust
Page::new("Inicio")
    .style(":root { --accent: #e11d48; --radius: 6px; }")
    .body(h1().text("Botones rojos, esquinas más marcadas"))
```

| Variable | Función |
|---|---|
| `--bg`, `--fg` | colores del fondo y del texto |
| `--muted` | texto secundario |
| `--line` | bordes |
| `--card` | fondo de las tarjetas, los campos y los bloques de código |
| `--accent`, `--accent-fg` | botones y enlaces, y el texto de los botones |
| `--radius` | radio de las esquinas de las tarjetas |
| `--font`, `--mono` | las fuentes del texto y del código |

El modo oscuro redefine `--bg`, `--fg`, `--muted`, `--line`, `--card` y `--accent` dentro de `@media (prefers-color-scheme: dark)`.

### Tu propio CSS

- Para unas pocas reglas, usa `.style(css)`: el CSS se escribe en la página y su política lo permite.
- Para una hoja de estilos de todo el sitio, ponla en `public/` y enlázala: `.stylesheet("/site.css")`. Los navegadores la guardan en caché entre páginas.
- Para empezar desde una página en blanco, llama a `.plain()`: el tema no se incluye, solo tus estilos.

El CSS del tema también está disponible como la constante `paquets::THEME`, por si quieres reutilizarlo en otro sitio.

## Etiquetas SEO

`Page::new(title)` y `.description(...)` ya producen el título, la descripción y las etiquetas Open Graph y Twitter que leen los buscadores y las redes sociales. Añade más con `.seo(...)`:

```rust
async fn article(_req: Request) -> Page {
    Page::new("Nuestra nueva silla de roble")
        .description("Roble macizo, diseñada para durar cincuenta años.")
        .seo(
            Seo::new()
                .base_url("https://acme.example")
                .canonical("/blog/oak-chair")
                .image("/images/oak-chair.jpg")
                .og_type("article")
                .site_name("Acme")
                .title_template("%s · Acme")
                .alternate("fr", "/fr/blog/chaise-chene")
                .json_ld(r#"{"@context":"https://schema.org","@type":"Article","headline":"Nuestra nueva silla de roble"}"#),
        )
        .body(h1().text("Nuestra nueva silla de roble"))
}
```

| Método de `Seo` | Etiqueta |
|---|---|
| `.canonical(url)` | `<link rel="canonical">`: la dirección oficial de la página |
| `.image(url)` | `og:image` y una tarjeta grande de Twitter |
| `.base_url(origin)` | convierte las URL relativas (canónica, imagen, alternativas) en absolutas |
| `.site_name(name)` | `og:site_name` |
| `.title_template("%s · Acme")` | el `<title>` final, con `%s` reemplazado por el título de la página |
| `.noindex(true)`, `.nofollow(true)` | `<meta name="robots">`, para que una página no aparezca en los buscadores |
| `.alternate(lang, url)` | `<link rel="alternate" hreflang>`, una vez por idioma |
| `.json_ld(json)` | datos estructurados para obtener resultados enriquecidos en los buscadores |
| `.og_type(...)`, `.locale(...)`, `.twitter(handle)` | las demás etiquetas Open Graph y Twitter |
| `.meta_name(name, content)`, `.meta_property(...)` | cualquier otra etiqueta `<meta>` |

Un `Seo` también se puede colocar dentro de una vista (`.child(Seo::new().title(...))`): todos los `Seo` de la página se fusionan, gana el valor más profundo y no se duplican etiquetas. Así, un layout puede fijar valores para todo el sitio que cada página completa. Consulta la [guía de SEO de active](https://github.com/maxlestage/Active/blob/master/site/docs/es/seo.md).

## La Content-Security-Policy

Cada página se envía con una cabecera `Content-Security-Policy` que le dice al navegador qué puede cargar y ejecutar la página. Paquets la calcula a partir de la propia página:

| Permitido | Bloqueado |
|---|---|
| los archivos de tu propio sitio (`/site.css`, `/app.js`, imágenes, fuentes) | scripts, hojas de estilo y fuentes de otros sitios (CDN, Google Fonts) |
| los bloques `<style>` y las etiquetas `<script>` inline propios de la página, por su hash SHA-256 | cualquier script o estilo inline que no estuviera en la página cuando se renderizó |
| los atributos `style="..."` | los manejadores de eventos inline como `onclick="..."` |
| las imágenes de tu sitio, de URL `https:` y de URL `data:` | mostrarse en un marco (frame) de otro sitio |
| el WebAssembly de tus islas | |

Aunque un atacante consiguiera colar algo de HTML en una página (a través de un campo de comentarios, por ejemplo), su script no se ejecutaría.

Los archivos HTML estáticos de `public/` (una landing page, o un sitio exportado por un generador de sitios estáticos) también están cubiertos: al arrancar, el escudo por defecto los lee y su política permite sus scripts y estilos inline (por hash), las hojas de estilo y los scripts externos que enlazan (Google Fonts incluido, con sus archivos de fuentes), WebAssembly y las imágenes `https:`. Esta política automática pertenece al escudo por defecto: si reemplazas el escudo con `app.security(...)`, define tú la política (consulta [Seguridad](security.md#cabeceras-de-seguridad-y-content-security-policy)).

La forma más sencilla de usar una fuente o una biblioteca de otro sitio es **descargarla en `public/`** y enlazarla desde ahí: además es más rápido y mejor para la privacidad de tus visitantes. Si de verdad necesitas una fuente externa, añádela a la política de esa respuesta:

```rust
app.get("/", |_| async {
    let res = Page::new("Inicio")
        .stylesheet("https://fonts.googleapis.com/css2?family=Inter")
        .body(h1().text("Hola"))
        .into_response();
    // Permitir Google Fonts en la política de esta página.
    let csp = res
        .get_header("content-security-policy")
        .unwrap_or_default()
        .replace("style-src 'self'", "style-src 'self' https://fonts.googleapis.com")
        .replace("font-src 'self'", "font-src 'self' https://fonts.gstatic.com");
    res.header("content-security-policy", csp)
});
```

## Formularios

Un formulario es una vista como cualquier otra. Haz que su `action` apunte a una ruta que lo lea con `req.form()`:

```rust
fn contact_form() -> Element {
    form()
        .attr("method", "post")
        .attr("action", "/contact")
        .child(label().attr("for", "email").text("Tu correo electrónico"))
        .child(input().attr("type", "email").attr("name", "email").id("email"))
        .child(label().attr("for", "message").text("Mensaje"))
        .child(textarea().attr("name", "message").id("message"))
        .child(button().text("Enviar"))
}
```

No hace falta ningún token CSRF: el escudo rechaza los formularios enviados desde otros sitios web. La parte del handler está en [Peticiones y respuestas](requests-responses.md#formularios).

## Partes interactivas

Una página es HTML estático. Para que una parte reaccione en el navegador (un contador, un cuadro de búsqueda, un carrito), convierte esa parte en una **isla**: `island("counter", "0", counter)`. Solo las páginas que contienen islas cargan el bundle del navegador. Consulta [Islas](islands.md).

## Aplicación instalable (PWA)

Una línea hace que la aplicación se pueda instalar en móviles y ordenadores, y usar sin conexión:

```rust
app.pwa(
    Pwa::new("Mis tareas")
        .short_name("Tareas")
        .theme_color("#6d4aff")
        .background_color("#0b0b12"),
);
```

- La aplicación sirve el manifiesto de aplicación web en `/manifest.webmanifest` y un service worker en `/sw.js`; cada `Page` los enlaza (con el color del tema, el título en la pantalla de inicio y el icono de Apple), y la política de la página permite el script de registro.
- Sin conexión: la página de inicio, el bundle de las islas y los iconos se descargan al instalar. Las páginas vienen primero de la red y, sin red, de la caché; las API REST de `app.api` vienen siempre de la red.
- Sin iconos, se usa la caja de Paquets. Tus propios iconos (al menos un PNG de 192×192 y otro de 512×512 en `public/`): `.icon("/icon-192.png", "192x192", "image/png").icon("/icon-512.png", "512x512", "image/png").maskable_icon("/icon-maskable.png", "512x512", "image/png")`.
- `.start_url("/demo")` abre otra página; `.description(...)`, `.lang("es")`, `.display("fullscreen")`… completan el manifiesto (lo construye [el módulo `pwa` de active](https://docs.rs/active/latest/active/pwa/index.html)).
- Los navegadores solo instalan una PWA servida por HTTPS (o en `localhost`): Heroku da HTTPS a cada aplicación. En el móvil, abre el sitio y luego *Añadir a pantalla de inicio* (menú de compartir de Safari) o *Instalar aplicación* (menú de Chrome).

## Más información

Las vistas vienen de [active](https://github.com/maxlestage/Active). Su documentación trata en profundidad los [componentes y las vistas](https://github.com/maxlestage/Active/blob/master/site/docs/es/components.md), el [SEO](https://github.com/maxlestage/Active/blob/master/site/docs/es/seo.md) y los [datos estructurados](https://github.com/maxlestage/Active/blob/master/site/docs/es/structured-data.md).

## Próximos pasos

- [Base de datos](database.md): llena tus páginas de datos.
- [Islas](islands.md): partes interactivas en Rust.
- [Seguridad](security.md): el escudo y las cabeceras que añade.
