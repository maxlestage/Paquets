# Autenticación

`app.auth(Auth::new())` añade cuentas a una aplicación en una línea, como Laravel Breeze: registro, inicio y cierre de sesión, contraseña olvidada, códigos de dos factores, y las páginas que los acompañan. Todo se apoya en [Securite](https://securite-2db4c1994be7.herokuapp.com/): hash Argon2id, límite de intentos de inicio de sesión, cookies y enlaces firmados, códigos TOTP.

## Una línea

```rust
use paquets::prelude::*;
use paquets::auth::{self, Auth};

async fn dashboard(req: Request) -> String {
    let user = req.user().expect("checked by login_required");
    format!("Hello {}", user.name)
}

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    app.auth(Auth::new().name("Mi app").admins(["yo@example.com"]));
    app.get("/dashboard", dashboard.with(auth::login_required));
    app.run()
}
```

O `paquets make:auth` en un proyecto (ver [Generadores](generators.md)).

## Las páginas

| Ruta | Página |
|---|---|
| `GET/POST /register` | crea una cuenta (nombre, e-mail, contraseña dos veces) e inicia sesión |
| `GET/POST /login` | inicia sesión y va a `?next=` (solo una ruta local) o a la portada |
| `POST /logout` | cierra la sesión |
| `GET/POST /forgot-password` | envía un enlace de restablecimiento |
| `GET/POST /reset-password?token=…` | elige una nueva contraseña |
| `GET /account` | la cuenta: códigos de dos factores, cerrar sesión |
| `GET/POST /two-factor` | el código pedido tras la contraseña, una vez activados los códigos |

Las páginas se renderizan en el servidor con el tema por defecto, su propia Content-Security-Policy y `noindex`. Sus campos se validan con las mismas reglas que los formularios de Active (`required`, `email`, `min_length`), y un formulario inválido vuelve con un error bajo cada campo (estado `422`).

## En los handlers

`req.user()` (el trait `AuthExt` del preludio) es el [`User`](https://docs.rs/paquets/latest/paquets/auth/struct.User.html) conectado, o `None`:

```rust
use paquets::prelude::*;

async fn hello(req: Request) -> String {
    match req.user() {
        Some(user) => format!("Hola {}", user.name),
        None => "Hola desconocido".into(),
    }
}
```

Dos middlewares protegen las rutas: `auth::login_required` (los visitantes van a la página de inicio de sesión y luego vuelven; los clientes de API reciben un `401`) y `auth::admin_required` (`403` para los usuarios que no son administradores). Los usuarios son filas de la tabla `users` (`id`, `email`, `name`, `admin`, `created_at`…), creada al arrancar: consúltala como cualquier [modelo](database.md). El hash de la contraseña, el secreto de dos factores y la versión de sesión nunca aparecen en JSON.

## Ajustes

| Método | Por defecto |
|---|---|
| `.prefix("/auth")` | las rutas en la raíz (`/login`) |
| `.home("/dashboard")` | `/`, tras iniciar sesión sin `?next=` |
| `.name("Mi app")` | el título de las páginas y de las apps de autenticación |
| `.url("https://example.com")` | `APP_URL`, luego el `Host` de la petición: la dirección de los enlaces enviados por e-mail |
| `.secret(…)` | `APP_KEY` (32 bytes o más), o una clave aleatoria al arrancar |
| `.previous_secret(…)` | ninguno: las `APP_KEY` antiguas se siguen aceptando tras una rotación |
| `.registration(false)` | los visitantes pueden registrarse |
| `.two_factor(false)` | los usuarios pueden activar los códigos de dos factores |
| `.admins([…])` | las direcciones de e-mail de los administradores |
| `.session_ttl(duración)` | 30 días |
| `.hasher(PasswordHasher…)` | Argon2id, parámetros de OWASP |
| `.throttle(LoginThrottle…)` | 5 fallos por cuenta e IP, luego 15 minutos |
| `.mailer(|link| async {…})` | muestra el enlace en la terminal (builds de debug) |

En producción, define `APP_KEY` (`openssl rand -base64 48`): sin ella, se genera una clave aleatoria al arrancar (un aviso lo indica, en todos los builds), así que las sesiones y los códigos de dos factores dejan de funcionar en cada reinicio. Para cambiarla, pasa la antigua a `.previous_secret(…)` durante un tiempo. Define también `APP_URL`, para que los enlaces de restablecimiento nunca confíen en el `Host` de una petición.

### Enviar el e-mail de restablecimiento

```rust
use paquets::auth::{Auth, ResetLink};

let auth = Auth::new().mailer(|link: ResetLink| async move {
    // Envía `link.url` a `link.user.email` con la crate de e-mail que prefieras.
    println!("a {}: {}", link.user.email, link.url);
    Ok(())
});
```

La página de contraseña olvidada responde lo mismo tenga la dirección una cuenta o no.

## Cómo te protege

- **Contraseñas** con hash Argon2id, en los hilos bloqueantes de tokio; una dirección desconocida tarda lo mismo que una conocida (`verify_dummy`): los tiempos de respuesta no revelan nada.
- **Fuerza bruta**: los inicios de sesión fallidos, las peticiones de restablecimiento y los códigos de dos factores se cuentan por cuenta e IP; el sexto fallo responde `429` con `Retry-After`.
- **Sesiones**: cookies firmadas con HMAC-SHA256, `HttpOnly`, `SameSite=Lax`, `Secure` en HTTPS. Cambiar la contraseña termina todas las sesiones.
- **Enlaces de restablecimiento** firmados, válidos una hora y una sola vez.
- **Códigos de dos factores** TOTP (RFC 6238): cualquier app de autenticación, con la clave mostrada en la página de la cuenta.
- **Secretos de dos factores cifrados en reposo** con XChaCha20-Poly1305 (el `Encrypter` de Securite), con una clave derivada de `APP_KEY` (HMAC-SHA256, distinta de la clave de firma) y ligada a la fila del usuario. Un secreto guardado en claro por una versión anterior se sigue leyendo, y se cifra en el siguiente inicio de sesión con dos factores.
- **Registros**: el `Debug` de `User` muestra `<redacted>` en lugar del hash de la contraseña y del secreto de dos factores.
- **CSRF**: el escudo rechaza los formularios enviados desde otro sitio (Fetch Metadata).

## En las pruebas

Argon2 es lento a propósito, y más sin optimizaciones: da a las pruebas un hasher rápido.

```rust
use paquets::prelude::*;
use paquets::auth::{Auth, PasswordHasher};

fn app() -> App {
    let mut app = App::new();
    app.auth(
        Auth::new()
            .secret("a test secret that is long enough to sign cookies")
            .hasher(PasswordHasher::insecure_fast_for_tests()),
    );
    app
}
```

`paquets make:auth` compila además Argon2 con optimizaciones en debug (`[profile.dev.package.argon2]` en `Cargo.toml`).

## Próximos pasos

- [Panel de administración](admin.md): los administradores gestionan los modelos en el navegador.
- [Seguridad](security.md): el escudo delante de cada petición.
