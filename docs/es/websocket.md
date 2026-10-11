# WebSocket

Un WebSocket mantiene una conexión abierta en los dos sentidos, de modo que el servidor puede enviar mensajes en cualquier momento: chat, notificaciones, paneles en directo, juegos multijugador. Paquets lo admite de serie con `app.ws(path, handler)`, que viene de [Vitesse](https://maxlestage.github.io/Vitesse/). Esta página construye un servidor de eco y luego una sala de chat, y trata la seguridad, el despliegue y las pruebas.

## Un servidor de eco

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.ws("/echo", |_req, mut socket| async move {
        while let Some(Ok(message)) = socket.recv().await {
            if let ws::Message::Text(text) = message {
                if socket.send(format!("eco: {text}")).await.is_err() {
                    break; // el cliente se ha ido
                }
            }
        }
        // El bucle termina cuando el cliente cierra la conexión.
    });

    app.run()
}
```

El handler recibe la petición y el socket abierto, y la conexión vive mientras el handler se ejecuta. En lugar de callbacks, un WebSocket es un bucle: `socket.recv().await` espera el siguiente mensaje sin bloquear nada más.

Para probarlo, usa [websocat](https://crates.io/crates/websocat), un cliente WebSocket para la terminal (`cargo install websocat`):

```sh
websocat ws://localhost:3000/echo
```

Escribe una línea: el servidor responde `eco: ...`.

`app.ws` registra una ruta `GET`, con parámetros y comodines como cualquier ruta. Una petición HTTP normal a esa ruta (un navegador que abre la URL) recibe `426 Upgrade Required`.

## Mensajes

| Método | Efecto |
|---|---|
| `socket.recv().await` | el siguiente mensaje: `Some(Ok(message))`, `Some(Err(error))`, o `None` una vez cerrada la conexión |
| `socket.send(value).await` | envía un mensaje: un `String` o un `&str` como texto, un `Vec<u8>` como binario, o un `ws::Message` |
| `socket.close(code, reason).await` | cierra la conexión (`1000` es un cierre normal) |
| `socket.split()` | dos mitades, para enviar desde una tarea mientras se recibe en otra |

Un `ws::Message` es uno de estos:

| Variante | Contenido |
|---|---|
| `Message::Text(text)` | un mensaje de texto |
| `Message::Binary(bytes)` | un mensaje binario |
| `Message::Ping(..)`, `Message::Pong(..)` | mensajes de control; los pings se responden automáticamente |
| `Message::Close(..)` | el cliente cierra la conexión |

Atajos: `message.as_text()` devuelve un `Option<&str>`, y `message.is_close()` un `bool`. Los mensajes están limitados a 16 MiB.

### Mensajes JSON

A menudo los mensajes son JSON. Analízalos en un struct `#[data]` y responde con `json!`:

```rust
#[data]
struct Move {
    x: i32,
    y: i32,
}

app.ws("/game", |_req, mut socket| async move {
    while let Some(Ok(message)) = socket.recv().await {
        let Some(text) = message.as_text() else {
            continue; // binario, ping, pong…
        };
        let reply = match paquets::serde_json::from_str::<Move>(text) {
            Ok(m) => json!({ "ok": true, "x": m.x, "y": m.y }),
            Err(e) => json!({ "ok": false, "error": e.to_string() }),
        };
        if socket.send(reply.to_string()).await.is_err() {
            break;
        }
    }
});
```

## Una sala de chat

Cada mensaje que envía un cliente se envía a todos los demás. Un canal `broadcast` de tokio, guardado en el [estado](configuration.md#estado-compartido) de la aplicación, hace de sala:

```rust
use paquets::prelude::*;
use paquets::vitesse::tokio::{self, sync::broadcast};

/// La sala de chat: todos los clientes conectados escuchan este canal.
struct Room(broadcast::Sender<String>);

fn app() -> App {
    let mut app = App::new();
    let (sender, _) = broadcast::channel(256);
    app.state(Room(sender));

    app.ws("/chat/:name", |req, socket| async move {
        let name = req.param("name").unwrap_or("anónimo").to_owned();
        let room = &req.state::<Room>().0;
        let mut inbox = room.subscribe();
        let (mut tx, mut rx) = socket.split();

        let _ = room.send(format!("* {name} ha entrado"));
        // Reenviar los mensajes de la sala a este cliente...
        let forward = tokio::spawn(async move {
            while let Ok(line) = inbox.recv().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });
        // ...y enviar a la sala lo que dice este cliente.
        while let Some(Ok(message)) = rx.recv().await {
            match message {
                ws::Message::Text(text) => {
                    let _ = room.send(format!("{name}: {}", text.trim_end()));
                }
                ws::Message::Close(_) => break,
                _ => {}
            }
        }
        forward.abort();
        let _ = room.send(format!("* {name} ha salido"));
    });

    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Cómo funciona:

- cada conexión se suscribe a la sala y divide su socket en dos: una tarea reenvía los mensajes de la sala al cliente, mientras el bucle del handler envía a la sala lo que dice el cliente;
- cuando el cliente se va, el bucle termina: el handler detiene la tarea de reenvío y anuncia la salida.

Vitesse reexporta tokio (`paquets::vitesse::tokio`), así que no necesitas añadirlo a tus dependencias.

Pruébalo con dos terminales: `websocat ws://localhost:3000/chat/ada` y `websocat ws://localhost:3000/chat/grace`.

## Conectarse desde una página

Los navegadores abren las conexiones WebSocket desde JavaScript. El cliente más sencillo es un pequeño archivo en `public/`, cargado por tu página:

```rust
app.get("/", |_| async {
    Page::new("Chat")
        .head(r#"<script type="module" src="/chat.js"></script>"#)
        .body(h1().text("💬 Chat"))
        .body(ul().id("log"))
        .body(
            form()
                .id("send")
                .class("row")
                .child(input().attr("name", "message").attr("autocomplete", "off"))
                .child(button().text("Enviar")),
        )
});
```

```js
// public/chat.js
const log = document.querySelector("#log");
const form = document.querySelector("#send");
const scheme = location.protocol === "https:" ? "wss" : "ws";
const socket = new WebSocket(`${scheme}://${location.host}/chat/guest`);

socket.onmessage = (event) => {
  const item = document.createElement("li");
  item.textContent = event.data;
  log.append(item);
};

form.onsubmit = (event) => {
  event.preventDefault();
  socket.send(form.message.value);
  form.message.value = "";
};
```

La política de seguridad de la página permite los scripts de tu propio sitio y las conexiones hacia él, así que no hace falta nada más. Desde una [isla](islands.md), usa el `WebSocket` del navegador a través del crate `web-sys`.

## Seguridad

El handshake es una petición `GET` normal, así que toda la cadena se ejecuta antes de que se abra la conexión: el escudo (límite de tasa, cortafuegos, bloqueador de bots), tus middlewares globales y el middleware de un router. Un middleware que responde `401` o `403` rechaza la conexión.

Los navegadores envían las cookies del sitio con el handshake, pero las reglas del mismo origen que protegen las peticiones normales no se aplican a WebSocket. Si identificas a los usuarios con cookies, comprueba la cabecera `Origin` para que una página de otro sitio no pueda abrir una conexión en nombre de tus usuarios:

```rust
async fn same_origin_only(req: Request, next: Next) -> Response {
    match req.header("origin") {
        // Los navegadores siempre envían Origin; otros clientes (scripts, aplicaciones) normalmente no.
        Some(origin) if origin != "https://chat.example.com" => {
            Error::forbidden("origen desconocido").into_response()
        }
        _ => next.run(req).await,
    }
}

let mut live = Router::new();
live.middleware(same_origin_only);
live.ws("/notifications", |_req, mut socket| async move {
    let _ = socket.send("conectado").await;
});
app.mount("/live", live); // ws://.../live/notifications
```

Los navegadores no pueden añadir una cabecera `Authorization` a una conexión WebSocket: autentica con una cookie de sesión o con un token de corta duración en la query string (`/live/notifications?token=...`, que se lee con `req.query("token")`).

## En producción

- **Heroku** admite WebSocket (usa `wss://` con la dirección HTTPS de tu aplicación). Su router cierra una conexión que permanece en silencio unos 55 segundos: para las conexiones que pueden quedarse inactivas, envía un ping desde el servidor más o menos cada 30 segundos (`tx.send(ws::Message::Ping(Default::default()))` desde una tarea, como en la sala de chat); los navegadores responden automáticamente.
- **Detrás de Nginx**, el proxy debe reenviar la cabecera `Upgrade`: consulta la [guía de WebSocket de Vitesse](https://maxlestage.github.io/Vitesse/es/docs/websocket/). Caddy lo gestiona automáticamente.
- **Reinicios**: cuando el servidor se detiene (un despliegue, `SIGTERM`), las conexiones abiertas se cortan. Haz que los clientes se reconecten automáticamente, algo que de todos modos necesitan para los cortes de red.
- **Solo HTTP/1.1**: WebSocket no funciona sobre HTTP/3.

## Pruebas

El cliente de pruebas de `app.test()` no tiene un socket real: para una ruta WebSocket solo ve el handshake, lo que basta para probar los rechazos:

```rust
#[tokio::test]
async fn plain_http_is_refused() {
    let client = app().test().await;
    assert_eq!(client.get("/echo").await.status(), 426);
}
```

Para probar una conversación, arranca el servidor en un puerto real y conéctate con un cliente WebSocket como `tokio-tungstenite`:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tokio-tungstenite = { version = "0.30", features = ["connect"] }
futures-util = "0.3"
```

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    #[tokio::test]
    async fn echoes_text_messages() {
        let mut app = app();
        app.port(4417).quiet(); // un puerto que solo usa esta prueba
        tokio::spawn(app.serve());
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (mut socket, _) = connect_async("ws://127.0.0.1:4417/echo").await.unwrap();
        socket.send(Message::text("hola")).await.unwrap();
        let reply = socket.next().await.unwrap().unwrap();
        assert_eq!(reply, Message::text("eco: hola"));
    }
}
```

`app.serve()` arranca el servidor en el propio runtime de la prueba. Da a cada una de estas pruebas su propio puerto. Consulta [Pruebas](testing.md).

## Más información

La [guía de WebSocket de Vitesse](https://maxlestage.github.io/Vitesse/es/docs/websocket/) trata en detalle los subprotocolos, los límites de tamaño de los mensajes (`ws::Upgrade`), `Stream` y `Sink`, y el comportamiento detrás de un proxy.

## Próximos pasos

- [Pruebas](testing.md): prueba tus rutas sin red.
- [Configuración](configuration.md): estado compartido y ajustes.
- [Desplegar en Heroku](deploy-heroku-mobile.md): WebSocket funciona allí tal cual.
