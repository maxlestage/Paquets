# Seguridad

Toda aplicación Paquets está protegida desde su primera línea: `App::new()` pone el escudo recomendado de [Securite](https://maxlestage.github.io/Securite/) delante de cada petición. Los ataques se bloquean antes de que se ejecute tu código, y cada respuesta recibe las cabeceras de seguridad que necesitan los navegadores. Esta página explica qué hace el escudo, cómo modificarlo y qué sigue siendo tarea tuya, como el inicio de sesión de los usuarios.

## Qué hace el escudo

El escudo por defecto ejecuta diez guards, en este orden, para cada petición (unos 6 µs en total):

| Guard | Nombre | Protege contra | Cuándo bloquea |
|---|---|---|---|
| Identificador de petición | `request_id` | — | nunca: da a cada petición un `X-Request-Id`, que también se devuelve en la respuesta |
| Higiene | `hygiene` | peticiones mal formadas o demasiado grandes | URI de más de 8 KiB, más de 100 cabeceras, segmentos `..`, codificación incorrecta: `400` |
| Filtro de métodos | `method` | métodos HTTP poco habituales, trucos de sustitución de método | métodos distintos de `GET HEAD POST PUT PATCH DELETE OPTIONS`: `405` |
| Límite del cuerpo | `body_limit` | subidas enormes | cuerpo anunciado de más de 2 MiB: `413` |
| Límite de tasa | `rate_limit` | avalanchas de peticiones, fuerza bruta, scraping | más de 20 peticiones por segundo por IP, con ráfagas de hasta 100: `429` |
| Bloqueador de bots | `bot_blocker` | herramientas de ataque | escáneres conocidos (sqlmap, nikto, nuclei…): `403` |
| Cortafuegos (WAF) | `waf` | inyección SQL, XSS, inyección de comandos, Log4Shell, SSRF, path traversal… | un patrón de ataque en la URL, la query string o una cabecera: `403` |
| CSRF | `csrf` | falsificación de peticiones entre sitios | un `POST`, `PUT`, `PATCH` o `DELETE` enviado por un navegador desde otro sitio web: `403` |
| Cabeceras de seguridad | `security_headers` | XSS, clickjacking, MIME sniffing, degradación a HTTP | nunca: añade cabeceras a cada respuesta |
| Refuerzo de cookies | `cookies` | robo de cookies de sesión | nunca: añade `Secure`, `HttpOnly` y `SameSite=Lax` a cada cookie |

La línea de arranque lo confirma: `security: on (10 guards)`.

### Cómo es una petición bloqueada

Una petición bloqueada nunca llega a tus handlers. El cliente recibe un mensaje corto y genérico (sin detalles que puedan ayudar a un atacante), en el formato JSON estándar «problem details»:

```http
HTTP/1.1 429 Too Many Requests
content-type: application/problem+json
retry-after: 1
ratelimit-limit: 100
ratelimit-remaining: 0

{"type":"about:blank","title":"Too Many Requests","status":429,"code":"rate_limited","detail":"Too many requests, slow down.","request_id":"222caeb2-c3fa-4692-a3de-b6650499864c"}
```

| Código | Estado | Guard |
|---|---|---|
| `waf_blocked` | `403` | cortafuegos |
| `csrf_rejected` | `403` | CSRF |
| `rate_limited` | `429` | límite de tasa |
| `bot_blocked` | `403` | bloqueador de bots |

El log de peticiones muestra el estado (`GET /?q=... 403 0.05 ms`). El `request_id` te permite encontrar una petición en tus logs cuando un usuario informa de un problema.

### Cabeceras de seguridad

Cada respuesta, incluidos los errores y las peticiones bloqueadas, lleva:

| Cabecera | Valor | Efecto |
|---|---|---|
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains` | los navegadores solo usan HTTPS, durante dos años |
| `X-Content-Type-Options` | `nosniff` | no se adivinan los tipos de contenido |
| `X-Frame-Options` | `DENY` | tus páginas no se pueden mostrar en un marco (clickjacking) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | las URL completas no se filtran a otros sitios |
| `Permissions-Policy` | cámara, micrófono, geolocalización… desactivados | el código de terceros no puede usarlos |
| `Cross-Origin-Opener-Policy`, `Cross-Origin-Resource-Policy` | `same-origin` | aislamiento respecto a otros sitios |
| `Content-Security-Policy` | ver más abajo | qué scripts, estilos y recursos se pueden cargar |

## Cabeceras de seguridad y Content-Security-Policy

La Content-Security-Policy (CSP) le dice al navegador qué puede cargar y ejecutar una página. Aunque un atacante cuele algo de HTML en una página, una CSP estricta impide que su script se ejecute. Paquets la define de tres maneras:

| Respuesta | Política |
|---|---|
| Una [`Page`](pages.md) | su propia política, calculada al renderizar la página: los estilos y scripts inline de la página se permiten por su hash SHA-256, además de tus propios archivos, las imágenes `https:` y WebAssembly (consulta [Páginas](pages.md#la-content-security-policy)) |
| Los archivos HTML de `public/` | cuando `public/` contiene archivos `.html`, el escudo por defecto los lee al arrancar y permite sus scripts y estilos inline (por hash), las hojas de estilo y los scripts externos que enlazan (Google Fonts incluido, con `https://fonts.gstatic.com` para sus fuentes), WebAssembly y las imágenes `https:` |
| Todo lo demás (JSON, texto, `Html(...)`) | la política del escudo: solo recursos del mismo origen, sin scripts ni estilos inline |

La política automática para `public/` viene con el escudo por defecto. Si reemplazas el escudo con `app.security(...)`, define tú una política cuando tu HTML estático la necesite, con el guard `SecurityHeaders`:

```rust
use paquets::securite::headers::ContentSecurityPolicy;
use paquets::securite::prelude::*;

let csp = ContentSecurityPolicy::strict()
    .add_sources("script-src", ["https://cdn.example.com"])
    .remove_directive("upgrade-insecure-requests");
app.security(
    paquets::shield()
        .guard(SecurityHeaders::new().content_security_policy(csp))
        .build(),
);
```

## La IP del cliente detrás de un proxy

El límite de tasa cuenta las peticiones **por dirección IP**, así que el escudo debe conocer la dirección real de cada visitante. En Heroku, la aplicación está detrás del router de Heroku, y todas las conexiones vienen del router: la dirección real está en la cabecera `X-Forwarded-For`. El escudo por defecto detecta Heroku (la variable de entorno `DYNO`) y lee esa cabecera desde la derecha, de la forma segura. En otros entornos, usa la dirección de la conexión.

En otras plataformas detrás de un proxy, dile al escudo dónde encontrar la dirección; si no, todos los visitantes comparten la dirección del proxy, y el límite de tasa se aplica a todos ellos juntos:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .client_ip(ClientIpSource::heroku()) // Render, balanceadores de carga de AWS, un solo Nginx
        // .client_ip(ClientIpSource::fly())        // Fly.io
        // .client_ip(ClientIpSource::cloudflare()) // detrás de Cloudflare
        .build(),
);
```

> [!WARNING]
> Confía en una cabecera de reenvío solo cuando tu aplicación sea accesible **únicamente** a través del proxy que la define. Si no, cualquiera puede enviar la cabecera y elegir su dirección. Consulta la [guía de la IP del cliente](https://maxlestage.github.io/Securite/es/docs/client-ip/) de Securite.

Los handlers pueden leer la dirección que encontró el escudo con `req.extensions().get::<ClientIp>()` (consulta [Peticiones y respuestas](requests-responses.md#la-dirección-ip-del-cliente)).

## Modificar el escudo

`paquets::shield()` devuelve el escudo por defecto como un builder, listo para modificarlo, con la detección de Heroku incluida. Cambia lo que necesites, llama a `.build()` y pasa el resultado a `app.security(...)`:

```rust
use paquets::prelude::*;
use paquets::securite::prelude::*;

let mut app = App::new();
app.security(
    paquets::shield()
        .guard(RateLimit::per_minute(600)) // reemplaza el límite de tasa por defecto
        .without("bot_blocker")            // quita un guard, por su nombre
        .build(),
);
```

| Método del builder | Efecto |
|---|---|
| `.guard(g)` | añade un guard, o reemplaza el guard con el mismo nombre |
| `.without(name)` | quita un guard: `"rate_limit"`, `"waf"`, `"csrf"`… |
| `.client_ip(source)` | de dónde viene la IP del cliente (ver más arriba) |
| `.error_format(format)` | cómo se responde a las peticiones bloqueadas: `ErrorFormat::ProblemJson` (por defecto), `PlainText`, `Empty` |
| `.on_event(f)` | una función que se llama para cada petición bloqueada |
| `.build()` | el escudo terminado |

Importa los guards con `use paquets::securite::prelude::*;`. `app.security(...)` se puede llamar en cualquier momento antes de `run()`: el escudo se elige cuando arranca la aplicación.

### Recetas habituales

**Una API con más tráfico.** Sube el límite de tasa, por minuto o con una ráfaga a medida:

```rust
app.security(paquets::shield().guard(RateLimit::per_minute(600)).build());
app.security(paquets::shield().guard(RateLimit::new(Quota::per_second(50).burst(200))).build());
```

**Subidas más grandes.** Sube a la vez el límite del escudo y el límite de lectura (consulta [Peticiones y respuestas](requests-responses.md#tamaño-del-cuerpo)):

```rust
app.body_limit(10 * 1024 * 1024);
app.security(paquets::shield().guard(BodyLimit::new(10 * 1024 * 1024)).build());
```

**Un front end en otro origen.** Permítelo en CORS y declara que confías en él en el guard CSRF (consulta [API REST](rest-api.md#llamar-a-la-api-desde-un-navegador)):

```rust
app.security(
    paquets::shield()
        .guard(Cors::new().allow_origin("https://app.example.com"))
        .guard(Csrf::new().trusted_origin("https://app.example.com"))
        .build(),
);
```

**Un formulario reenviado por otro sitio.** Algunos proveedores de inicio de sesión y de pago devuelven al visitante a tu sitio con el `POST` de un formulario. Exime esa ruta del guard CSRF:

```rust
app.security(paquets::shield().guard(Csrf::new().exempt("/auth/callback")).build());
```

Los webhooks que envían los servidores (GitHub, Stripe…) no lo necesitan: las peticiones que no vienen de un navegador pasan el guard CSRF.

**Ver qué se bloquea.** Muestra cada petición bloqueada:

```rust
app.security(
    paquets::shield()
        .on_event(|e| {
            if e.enforced {
                eprintln!("[seguridad] {} bloqueó {} {} ({})", e.guard, e.method, e.path, e.status);
            }
        })
        .build(),
);
```

```text
[seguridad] waf bloqueó GET / (403 Forbidden)
```

**Probar un guard sin bloquear.** La mayoría de los guards tienen un modo de observación, que informa a través de `on_event` (con `e.enforced == false`) en lugar de bloquear: `.guard(Waf::new().monitor())`.

### Presets

Securite tiene tres presets. `paquets::shield()` es el recomendado, más la detección de Heroku:

| Preset | Para | Diferencias con el preset recomendado |
|---|---|---|
| `ShieldBuilder::recommended()` | sitios web y APIs que usan los navegadores | — |
| `ShieldBuilder::strict()` | sitios sensibles | añade una redirección a HTTPS, bloqueos automáticos tras rechazos repetidos, un límite de concurrencia por IP y una comprobación del tipo de contenido; límites más bajos (1 MiB, 10 peticiones por segundo); cookies `SameSite=Strict`; `Cross-Origin-Embedder-Policy: require-corp` (las imágenes y los scripts de otros sitios deben permitirlo) |
| `ShieldBuilder::api()` | APIs JSON que se usan con tokens, no con cookies | sin guard CSRF, solo cuerpos JSON, una CSP que no permite nada, `Cache-Control: no-store` |

Los presets no conocen Heroku: añade tú la IP del cliente.

```rust
app.security(ShieldBuilder::strict().client_ip(ClientIpSource::heroku()).build());
```

`Shield::strict()` y `Shield::api()` son los mismos presets, ya construidos.

### Desactivar el escudo

```rust
app.security(None);
```

Hazlo solo cuando otra capa ya proteja la aplicación (un WAF en tu CDN, por ejemplo), o para un benchmark. La línea de arranque dice entonces `security: off`.

## Lo que el escudo no hace

- **No lee los cuerpos de las peticiones.** El cortafuegos inspecciona la URL, la query string y las cabeceras. Los cuerpos JSON y de formulario son tarea de tus handlers: valídalos (consulta [Peticiones y respuestas](requests-responses.md#cuerpos-json)). Dos cosas ya te protegen ahí: Memoire envía cada valor como un parámetro enlazado, así que las consultas de los modelos no se pueden convertir en una inyección SQL, y las vistas escapan todo el texto, así que los datos que se muestran en una página no pueden inyectar HTML.
- **No conoce a tus usuarios.** La autenticación y los permisos quedan en tus manos (sección siguiente).
- **No proporciona HTTPS por sí mismo.** Paquets sirve HTTP sin cifrar; Heroku, un proxy inverso o una CDN proporcionan HTTPS delante. La cabecera `Strict-Transport-Security` mantiene después a los navegadores en HTTPS.

## Autenticación

Paquets no impone ninguna forma de iniciar la sesión de los usuarios. Tres enfoques habituales, del más sencillo al más completo:

### Un token compartido para una API

Para una API que usan tus propios scripts o aplicaciones, comprueba un token secreto en un middleware: consulta [API REST](rest-api.md#proteger-la-api).

### Una contraseña para una sección de administración

El guard `Auth` de Securite protege rutas con autenticación HTTP Basic (el navegador muestra un cuadro de inicio de sesión), bearer tokens o claves de API:

```rust
use paquets::securite::prelude::*;

app.security(
    paquets::shield()
        .guard(Auth::basic("Admin").user("admin", "correct horse battery").protect(["/admin"]))
        .build(),
);

app.get("/admin", |req: Request| async move {
    let name = req.extensions().get::<Principal>().map(|p| p.name.to_string()).unwrap_or_default();
    format!("Hola, {name}")
});
```

Ahora todas las rutas bajo `/admin` necesitan el nombre de usuario y la contraseña; las demás siguen siendo públicas. Lee la contraseña de una variable de entorno (`std::env::var("ADMIN_PASSWORD")`) en lugar de escribirla en el código. La [guía de autenticación](https://maxlestage.github.io/Securite/es/docs/auth/) de Securite trata los tokens, las claves de API y las comprobaciones personalizadas como JWT.

### Cuentas y sesiones

Para las cuentas de usuario, el patrón habitual es:

1. guardar los usuarios en un modelo, con un **hash** de la contraseña (usa un crate dedicado como `argon2`, nunca un hash simple ni la propia contraseña);
2. al iniciar sesión, comprobar la contraseña, crear un token de sesión aleatorio (con un generador aleatorio criptográfico), guardarlo en un modelo `Session` y enviarlo en una cookie: `res::redirect("/").cookie(Cookie::new("session", token))`;
3. en un middleware, leer `req.cookie("session")`, buscar la sesión con `req.db()` y adjuntar el usuario a la petición con `req.set(user)`; los handlers lo leen con `req.get::<User>()`.

El escudo se encarga del resto: la cookie recibe `Secure`, `HttpOnly` y `SameSite=Lax`, el límite de tasa frena los intentos de adivinar contraseñas y el guard CSRF impide que otros sitios envíen formularios en nombre de tus usuarios.

## Más información

La documentación de Securite tiene una página por guard: [WAF](https://maxlestage.github.io/Securite/es/docs/waf/), [límite de tasa](https://maxlestage.github.io/Securite/es/docs/rate-limit/), [CSRF](https://maxlestage.github.io/Securite/es/docs/csrf/), [cabeceras de seguridad](https://maxlestage.github.io/Securite/es/docs/security-headers/), [presets](https://maxlestage.github.io/Securite/es/docs/shield-presets/), y más: filtro de IP, filtro de hosts, bloqueos automáticos, firmas de webhooks. En Paquets, Securite se reexporta como `paquets::securite`.

## Próximos pasos

- [Errores](errors.md): cómo los errores se convierten en respuestas.
- [Configuración](configuration.md): todos los ajustes de la aplicación.
- [Desplegar en Heroku](deploy-heroku-mobile.md): HTTPS y la IP del cliente se gestionan por ti.
