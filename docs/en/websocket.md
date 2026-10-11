# WebSocket

A WebSocket keeps a connection open in both directions, so the server can send messages at any time: chat, notifications, live dashboards, multiplayer games. Paquets supports it out of the box with `app.ws(path, handler)`, from [Vitesse](https://maxlestage.github.io/Vitesse/). This page builds an echo server, then a chat room, and covers security, deployment and testing.

## An echo server

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.ws("/echo", |_req, mut socket| async move {
        while let Some(Ok(message)) = socket.recv().await {
            if let ws::Message::Text(text) = message {
                if socket.send(format!("echo: {text}")).await.is_err() {
                    break; // the client is gone
                }
            }
        }
        // The loop ends when the client closes the connection.
    });

    app.run()
}
```

The handler receives the request and the open socket, and the connection lives as long as the handler runs. Instead of callbacks, a WebSocket is a loop: `socket.recv().await` waits for the next message without blocking anything else.

To try it, use [websocat](https://crates.io/crates/websocat), a WebSocket client for the terminal (`cargo install websocat`):

```sh
websocat ws://localhost:3000/echo
```

Type a line: the server answers `echo: ...`.

`app.ws` registers a `GET` route, with parameters and wildcards like any route. A plain HTTP request on that path (a browser opening the URL) gets `426 Upgrade Required`.

## Messages

| Method | Effect |
|---|---|
| `socket.recv().await` | the next message: `Some(Ok(message))`, `Some(Err(error))`, or `None` once the connection is closed |
| `socket.send(value).await` | sends a message: a `String` or `&str` as text, a `Vec<u8>` as binary, or a `ws::Message` |
| `socket.close(code, reason).await` | closes the connection (`1000` is a normal closure) |
| `socket.split()` | two halves, to send from one task while receiving in another |

A `ws::Message` is one of:

| Variant | Content |
|---|---|
| `Message::Text(text)` | a text message |
| `Message::Binary(bytes)` | a binary message |
| `Message::Ping(..)`, `Message::Pong(..)` | control messages; pings are answered automatically |
| `Message::Close(..)` | the client closes the connection |

Shortcuts: `message.as_text()` returns an `Option<&str>`, and `message.is_close()` a `bool`. Messages are limited to 16 MiB.

### JSON messages

Messages are often JSON. Parse them into a `#[data]` struct and answer with `json!`:

```rust
#[data]
struct Move {
    x: i32,
    y: i32,
}

app.ws("/game", |_req, mut socket| async move {
    while let Some(Ok(message)) = socket.recv().await {
        let Some(text) = message.as_text() else {
            continue; // binary, ping, pong…
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

## A chat room

Every message sent by one client is sent to all the others. A tokio `broadcast` channel, kept in the app's [state](configuration.md#shared-state), is the room:

```rust
use paquets::prelude::*;
use paquets::vitesse::tokio::{self, sync::broadcast};

/// The chat room: every connected client listens to this channel.
struct Room(broadcast::Sender<String>);

fn app() -> App {
    let mut app = App::new();
    let (sender, _) = broadcast::channel(256);
    app.state(Room(sender));

    app.ws("/chat/:name", |req, socket| async move {
        let name = req.param("name").unwrap_or("anonymous").to_owned();
        let room = &req.state::<Room>().0;
        let mut inbox = room.subscribe();
        let (mut tx, mut rx) = socket.split();

        let _ = room.send(format!("* {name} joined"));
        // Forward the room's messages to this client...
        let forward = tokio::spawn(async move {
            while let Ok(line) = inbox.recv().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });
        // ...and send what this client says to the room.
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
        let _ = room.send(format!("* {name} left"));
    });

    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

How it works:

- each connection subscribes to the room and splits its socket in two: a task forwards the room's messages to the client, while the handler's loop sends what the client says to the room;
- when the client leaves, the loop ends: the handler stops the forwarding task and announces the departure.

tokio is re-exported by Vitesse (`paquets::vitesse::tokio`), so you don't need to add it to your dependencies.

Try it with two terminals: `websocat ws://localhost:3000/chat/ada` and `websocat ws://localhost:3000/chat/grace`.

## Connecting from a page

Browsers open WebSocket connections from JavaScript. The simplest client is a small file in `public/`, loaded by your page:

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
                .child(button().text("Send")),
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

The page's security policy allows scripts from your own site and connections to it, so nothing else is needed. From an [island](islands.md), use the browser's `WebSocket` through the `web-sys` crate.

## Security

The handshake is an ordinary `GET` request, so the whole chain runs before the connection opens: the shield (rate limit, firewall, bot blocker), your global middleware and a router's middleware. A middleware that answers `401` or `403` refuses the connection.

Browsers send the site's cookies with the handshake, but the same-origin rules that protect normal requests don't apply to WebSocket. If you identify users with cookies, check the `Origin` header so that a page of another site can't open a connection in your users' name:

```rust
async fn same_origin_only(req: Request, next: Next) -> Response {
    match req.header("origin") {
        // Browsers always send Origin; other clients (scripts, apps) usually don't.
        Some(origin) if origin != "https://chat.example.com" => {
            Error::forbidden("unknown origin").into_response()
        }
        _ => next.run(req).await,
    }
}

let mut live = Router::new();
live.middleware(same_origin_only);
live.ws("/notifications", |_req, mut socket| async move {
    let _ = socket.send("connected").await;
});
app.mount("/live", live); // ws://.../live/notifications
```

Browsers can't add an `Authorization` header to a WebSocket connection: authenticate with a session cookie, or a short-lived token in the query string (`/live/notifications?token=...`, read with `req.query("token")`).

## In production

- **Heroku** supports WebSocket (use `wss://` with your app's HTTPS address). Its router closes a connection that stays silent for about 55 seconds: for connections that can be idle, send a ping from the server every 30 seconds or so (`tx.send(ws::Message::Ping(Default::default()))` from a task, as in the chat room); browsers answer automatically.
- **Behind Nginx**, the proxy must forward the `Upgrade` header: see [Vitesse's WebSocket guide](https://maxlestage.github.io/Vitesse/en/docs/websocket/). Caddy handles it automatically.
- **Restarts**: when the server stops (a deployment, `SIGTERM`), open connections are cut. Make clients reconnect automatically, which they need anyway for network drops.
- **HTTP/1.1 only**: WebSocket doesn't run over HTTP/3.

## Testing

The test client of `app.test()` has no real socket: for a WebSocket route it only sees the handshake, which is enough to test refusals:

```rust
#[tokio::test]
async fn plain_http_is_refused() {
    let client = app().test().await;
    assert_eq!(client.get("/echo").await.status(), 426);
}
```

To test a conversation, start the server on a real port and connect with a WebSocket client such as `tokio-tungstenite`:

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
        app.port(4417).quiet(); // a port used only by this test
        tokio::spawn(app.serve());
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (mut socket, _) = connect_async("ws://127.0.0.1:4417/echo").await.unwrap();
        socket.send(Message::text("hello")).await.unwrap();
        let reply = socket.next().await.unwrap().unwrap();
        assert_eq!(reply, Message::text("echo: hello"));
    }
}
```

`app.serve()` starts the server on the test's own runtime. Give each such test its own port. See [Testing](testing.md).

## Learn more

[Vitesse's WebSocket guide](https://maxlestage.github.io/Vitesse/en/docs/websocket/) covers subprotocols, message size limits (`ws::Upgrade`), `Stream` and `Sink`, and the behaviour behind proxies in detail.

## Next steps

- [Testing](testing.md): test your routes without a network.
- [Configuration](configuration.md): shared state and settings.
- [Deploy to Heroku](deploy-heroku-mobile.md): WebSocket works there as is.
