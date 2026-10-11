# WebSocket

Une WebSocket garde une connexion ouverte dans les deux sens : le serveur peut donc envoyer des messages à tout moment. Discussion en direct, notifications, tableaux de bord en temps réel, jeux multijoueurs… Paquets la prend en charge d'office avec `app.ws(chemin, handler)`, issu de [Vitesse](https://maxlestage.github.io/Vitesse/). Cette page construit un serveur d'écho, puis un salon de discussion, et couvre la sécurité, le déploiement et les tests.

## Un serveur d'écho

```rust
use paquets::prelude::*;

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.ws("/echo", |_req, mut socket| async move {
        while let Some(Ok(message)) = socket.recv().await {
            if let ws::Message::Text(text) = message {
                if socket.send(format!("écho : {text}")).await.is_err() {
                    break; // le client est parti
                }
            }
        }
        // La boucle se termine quand le client ferme la connexion.
    });

    app.run()
}
```

Le handler reçoit la requête et la socket ouverte, et la connexion vit aussi longtemps que le handler s'exécute. Pas de callbacks : une WebSocket, c'est une boucle. `socket.recv().await` attend le message suivant sans bloquer quoi que ce soit d'autre.

Pour l'essayer, utilisez [websocat](https://crates.io/crates/websocat), un client WebSocket pour le terminal (`cargo install websocat`) :

```sh
websocat ws://localhost:3000/echo
```

Tapez une ligne : le serveur répond `écho : ...`.

`app.ws` enregistre une route `GET`, avec paramètres et jokers comme n'importe quelle route. Une requête HTTP ordinaire sur ce chemin (un navigateur qui ouvre l'URL) reçoit `426 Upgrade Required`.

## Messages

| Méthode | Effet |
|---|---|
| `socket.recv().await` | le message suivant : `Some(Ok(message))`, `Some(Err(erreur))`, ou `None` une fois la connexion fermée |
| `socket.send(valeur).await` | envoie un message : une `String` ou un `&str` en texte, un `Vec<u8>` en binaire, ou un `ws::Message` |
| `socket.close(code, raison).await` | ferme la connexion (`1000` correspond à une fermeture normale) |
| `socket.split()` | deux moitiés, pour envoyer depuis une tâche pendant qu'une autre reçoit |

Un `ws::Message` est l'un des variants suivants :

| Variant | Contenu |
|---|---|
| `Message::Text(texte)` | un message texte |
| `Message::Binary(octets)` | un message binaire |
| `Message::Ping(..)`, `Message::Pong(..)` | des messages de contrôle ; les pings reçoivent une réponse automatique |
| `Message::Close(..)` | le client ferme la connexion |

Raccourcis : `message.as_text()` renvoie une `Option<&str>`, et `message.is_close()` un `bool`. Les messages sont limités à 16 Mio.

### Messages JSON

Les messages sont souvent du JSON. Analysez-les dans une struct `#[data]` et répondez avec `json!` :

```rust
#[data]
struct Move {
    x: i32,
    y: i32,
}

app.ws("/game", |_req, mut socket| async move {
    while let Some(Ok(message)) = socket.recv().await {
        let Some(text) = message.as_text() else {
            continue; // binaire, ping, pong…
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

## Un salon de discussion

Chaque message envoyé par un client est transmis à tous les autres. Un canal `broadcast` de tokio, rangé dans l'[état](configuration.md#état-partagé) de l'application, fait office de salon :

```rust
use paquets::prelude::*;
use paquets::vitesse::tokio::{self, sync::broadcast};

/// Le salon de discussion : chaque client connecté écoute ce canal.
struct Room(broadcast::Sender<String>);

fn app() -> App {
    let mut app = App::new();
    let (sender, _) = broadcast::channel(256);
    app.state(Room(sender));

    app.ws("/chat/:name", |req, socket| async move {
        let name = req.param("name").unwrap_or("anonyme").to_owned();
        let room = &req.state::<Room>().0;
        let mut inbox = room.subscribe();
        let (mut tx, mut rx) = socket.split();

        let _ = room.send(format!("* {name} est arrivé"));
        // Transmettre les messages du salon à ce client…
        let forward = tokio::spawn(async move {
            while let Ok(line) = inbox.recv().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });
        // …et envoyer au salon ce que dit ce client.
        while let Some(Ok(message)) = rx.recv().await {
            match message {
                ws::Message::Text(text) => {
                    let _ = room.send(format!("{name} : {}", text.trim_end()));
                }
                ws::Message::Close(_) => break,
                _ => {}
            }
        }
        forward.abort();
        let _ = room.send(format!("* {name} est parti"));
    });

    app
}

fn main() -> std::io::Result<()> {
    app().run()
}
```

Le fonctionnement :

- chaque connexion s'abonne au salon et coupe sa socket en deux : une tâche transmet les messages du salon au client, pendant que la boucle du handler envoie au salon ce que dit le client ;
- quand le client s'en va, la boucle se termine : le handler arrête la tâche de transmission et annonce le départ.

tokio est réexporté par Vitesse (`paquets::vitesse::tokio`) : inutile de l'ajouter à vos dépendances.

Essayez avec deux terminaux : `websocat ws://localhost:3000/chat/ada` et `websocat ws://localhost:3000/chat/grace`.

## Se connecter depuis une page

Les navigateurs ouvrent les connexions WebSocket depuis JavaScript. Le client le plus simple est un petit fichier dans `public/`, chargé par votre page :

```rust
app.get("/", |_| async {
    Page::new("Discussion")
        .head(r#"<script type="module" src="/chat.js"></script>"#)
        .body(h1().text("💬 Discussion"))
        .body(ul().id("log"))
        .body(
            form()
                .id("send")
                .class("row")
                .child(input().attr("name", "message").attr("autocomplete", "off"))
                .child(button().text("Envoyer")),
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

La politique de sécurité de la page autorise les scripts de votre propre site et les connexions vers celui-ci : rien d'autre n'est nécessaire. Depuis un [îlot](islands.md), utilisez la `WebSocket` du navigateur via la crate `web-sys`.

## Sécurité

La poignée de main (handshake) est une requête `GET` ordinaire : toute la chaîne s'exécute donc avant l'ouverture de la connexion, à savoir le bouclier (limitation de débit, pare-feu, blocage des robots), vos middlewares globaux et ceux d'un routeur. Un middleware qui répond `401` ou `403` refuse la connexion.

Les navigateurs envoient les cookies du site avec la poignée de main, mais les règles de même origine qui protègent les requêtes normales ne s'appliquent pas à WebSocket. Si vous identifiez les utilisateurs par cookies, vérifiez l'en-tête `Origin`, pour qu'une page d'un autre site ne puisse pas ouvrir de connexion au nom de vos utilisateurs :

```rust
async fn same_origin_only(req: Request, next: Next) -> Response {
    match req.header("origin") {
        // Les navigateurs envoient toujours Origin ; les autres clients (scripts, applications), en général non.
        Some(origin) if origin != "https://chat.example.com" => {
            Error::forbidden("origine inconnue").into_response()
        }
        _ => next.run(req).await,
    }
}

let mut live = Router::new();
live.middleware(same_origin_only);
live.ws("/notifications", |_req, mut socket| async move {
    let _ = socket.send("connecté").await;
});
app.mount("/live", live); // ws://.../live/notifications
```

Les navigateurs ne peuvent pas ajouter d'en-tête `Authorization` à une connexion WebSocket : authentifiez avec un cookie de session, ou avec un jeton de courte durée dans la query string (`/live/notifications?token=...`, lu avec `req.query("token")`).

## En production

- **Heroku** prend en charge WebSocket (utilisez `wss://` avec l'adresse HTTPS de votre application). Son routeur ferme une connexion qui reste silencieuse pendant environ 55 secondes : pour les connexions qui peuvent rester inactives, envoyez un ping depuis le serveur toutes les 30 secondes environ (`tx.send(ws::Message::Ping(Default::default()))` depuis une tâche, comme dans le salon de discussion) ; les navigateurs répondent automatiquement.
- **Derrière Nginx**, le proxy doit transmettre l'en-tête `Upgrade` : voir [le guide WebSocket de Vitesse](https://maxlestage.github.io/Vitesse/fr/docs/websocket/). Caddy s'en charge automatiquement.
- **Les redémarrages** : quand le serveur s'arrête (un déploiement, `SIGTERM`), les connexions ouvertes sont coupées. Faites en sorte que les clients se reconnectent automatiquement, ce dont ils ont besoin de toute façon pour les coupures réseau.
- **HTTP/1.1 uniquement** : WebSocket ne fonctionne pas sur HTTP/3.

## Tests

Le client de test de `app.test()` n'a pas de vraie socket : pour une route WebSocket, il ne voit que la poignée de main, ce qui suffit pour tester les refus :

```rust
#[tokio::test]
async fn plain_http_is_refused() {
    let client = app().test().await;
    assert_eq!(client.get("/echo").await.status(), 426);
}
```

Pour tester une conversation, démarrez le serveur sur un vrai port et connectez-vous avec un client WebSocket comme `tokio-tungstenite` :

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
        app.port(4417).quiet(); // un port utilisé uniquement par ce test
        tokio::spawn(app.serve());
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (mut socket, _) = connect_async("ws://127.0.0.1:4417/echo").await.unwrap();
        socket.send(Message::text("bonjour")).await.unwrap();
        let reply = socket.next().await.unwrap().unwrap();
        assert_eq!(reply, Message::text("écho : bonjour"));
    }
}
```

`app.serve()` démarre le serveur sur le runtime du test lui-même. Donnez à chacun de ces tests son propre port. Voir [Tests](testing.md).

## Pour aller plus loin

[Le guide WebSocket de Vitesse](https://maxlestage.github.io/Vitesse/fr/docs/websocket/) détaille les sous-protocoles, les limites de taille des messages (`ws::Upgrade`), `Stream` et `Sink`, et le comportement derrière les proxys.

## Étapes suivantes

- [Tests](testing.md) : tester vos routes sans réseau.
- [Configuration](configuration.md) : l'état partagé et les réglages.
- [Déployer sur Heroku](deploy-heroku-mobile.md) : WebSocket y fonctionne tel quel.
