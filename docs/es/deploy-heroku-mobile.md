# Desplegar en Heroku desde el móvil

No necesitas un ordenador para poner en línea una aplicación Paquets. Esta guía recorre cada paso desde un teléfono: un despliegue con un solo toque gracias al botón **Deploy to Heroku**, una base de datos PostgreSQL, despliegues automáticos con cada cambio gracias a GitHub Actions y, después, el día a día: redesplegar, leer los logs, añadir un nombre de dominio, solucionar problemas. Los mismos pasos sirven para el repositorio de Paquets y para tu propio proyecto creado con `paquets new`.

> [!NOTE]
> Todos los pasos funcionan en el navegador web de un teléfono (Safari, Chrome, Firefox…). La app de GitHub resulta práctica para las pestañas **Actions** y **Releases**, pero algunos ajustes (secretos, variables, edición de archivos) solo están en el sitio web de GitHub: para ellos, abre https://github.com en tu navegador. Si en github.com parece faltar un menú, cambia tu navegador al modo «Sitio de escritorio». Las interfaces de Heroku y GitHub están en inglés, y a veces cambian el nombre de sus menús: si una etiqueta es ligeramente distinta, busca la más parecida.

## Antes de empezar

Necesitas:

- **Una cuenta de Heroku**: regístrate en https://signup.heroku.com. Heroku **no tiene plan gratuito**: los dynos más baratos (los contenedores que ejecutan tu aplicación) son **Eco**, unos **5 $ al mes**, y **Basic**, unos 7 $ al mes. Heroku pide un método de pago antes de poder crear una aplicación. Los precios cambian: consulta https://www.heroku.com/pricing.
- **Una cuenta de GitHub**, para tu propia copia del código y para los despliegues automáticos (secciones 3 y 4).
- Opcionalmente, unos pocos dólares más al mes para una base de datos **Heroku Postgres** (sección 2).

### Cómo funciona

Todo lo que Heroku necesita ya está en el repositorio, así que no se compila nada en tu teléfono: Heroku construye la aplicación en sus propios servidores.

| Archivo | Función |
|---|---|
| `app.json` | describe la aplicación para el botón Deploy (nombre, descripción, el stack «container») |
| `heroku.yml` | le indica a Heroku que construya la imagen a partir del `Dockerfile` |
| `Dockerfile` | compila la aplicación y sus islas en modo release y las empaqueta en una imagen pequeña |
| `.github/workflows/heroku.yml` | el workflow **Deploy to Heroku**, para los despliegues automáticos |
| `.github/workflows/ci.yml` | el workflow **CI**: prueba cada push y luego fusiona la rama en `master` |

En el repositorio de Paquets, el `Dockerfile` construye el sitio web de Paquets, que es a su vez una aplicación Paquets (páginas renderizadas en el servidor, islas en WebAssembly), con una demo en vivo en `/demo` y una API REST en `/api/todos`. En un proyecto creado con `paquets new`, los mismos archivos construyen tu propia aplicación: `src/main.rs` y las islas de `src/lib.rs`.

## 1. La forma más rápida: el botón Deploy to Heroku

[![Deploy to Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets)

1. Toca el botón de arriba (también está en el README del repositorio), o abre esta dirección: `https://heroku.com/deploy?template=https://github.com/maxlestage/Paquets`. Inicia sesión en Heroku si te lo pide.
2. Heroku muestra un formulario para crear la aplicación:
   - **App name**: por ejemplo `my-paquets`. Los nombres son únicos en todo Heroku y se escriben con letras minúsculas, números y guiones. Puedes dejarlo vacío y dejar que Heroku elija uno.
   - **Choose a region**: Estados Unidos o Europa.
3. Toca **Deploy app**. Heroku descarga el código y construye la imagen Docker; el log de construcción va apareciendo en la página. La primera construcción tarda un rato (a menudo de 10 a 20 minutos): Rust compila la aplicación y su bundle de WebAssembly con todas las optimizaciones activadas.
4. Cuando termine la construcción, toca **View**. El sitio se abre en `https://<app-name>.herokuapp.com` (la dirección exacta también aparece en la pestaña **Settings** de la aplicación en Heroku).

Prueba la demo añadiendo estas rutas a la dirección:

| Ruta | Lo que ves |
|---|---|
| `/demo` | una lista de tareas renderizada en el servidor, con un formulario |
| `/api/todos` | las mismas tareas en JSON |

> [!NOTE]
> Hasta que añadas una base de datos, la aplicación guarda sus datos en memoria: se pierden en cada reinicio (cada despliegue, cada cambio de ajustes y el reinicio automático que Heroku hace aproximadamente una vez al día). La sección 2 lo soluciona.

## 2. Añadir una base de datos

1. Abre https://dashboard.heroku.com y toca tu aplicación.
2. Abre la pestaña **Resources**.
3. En **Add-ons**, busca **Heroku Postgres** y selecciónalo.
4. Elige un plan (el más pequeño, Essential-0, cuesta unos 5 $ al mes en el momento de escribir esto) y confirma con **Submit Order Form**.

Eso es todo. Heroku crea la base de datos, define la config var `DATABASE_URL` y reinicia la aplicación. Paquets lee `DATABASE_URL` al arrancar, se conecta con TLS y crea las tablas de tus modelos si todavía no existen. Para comprobarlo, abre los logs (**More** → **View logs**): la línea de arranque ahora dice `database: postgres`.

```text
📦 Paquets · http://localhost:21345
   database: postgres · security: on (10 guards) · islands: /pkg/paquets_site.js
```

(La dirección de esta línea es la del interior del contenedor: a tu aplicación se llega a través de `https://<app-name>.herokuapp.com`).

> [!WARNING]
> No uses SQLite en Heroku: el sistema de archivos de un dyno es temporal, y el archivo se borraría en cada reinicio.

## 3. Tu propia copia del código

Para modificar el código, necesitas tu propia copia del repositorio en GitHub:

1. Abre https://github.com/maxlestage/Paquets (la app de GitHub sirve), toca **Fork** y luego **Create fork**.
2. Despliega tu fork escribiendo esta dirección en tu navegador, con tu nombre de usuario de GitHub en lugar de `<you>`:

   ```text
   https://heroku.com/deploy?template=https://github.com/<you>/Paquets
   ```

3. Sigue los pasos de la sección 1.

> [!IMPORTANT]
> El botón despliega **una sola vez**. Cuando más adelante modifiques tu fork, la aplicación de Heroku no se actualizará sola. Configura los despliegues automáticos (sección 4), con el **mismo nombre de aplicación**: el workflow actualiza la aplicación que acabas de crear.

## 4. Despliegues automáticos con GitHub Actions

Objetivo: cada cambio fusionado en `master` reconstruye y redespliega la aplicación, sin que tengas que hacer nada. El workflow **Deploy to Heroku** habla directamente con la Platform API de Heroku: sin la CLI de Heroku y sin ordenador. En cada ejecución:

1. comprueba que está configurado (si no, se detiene sin fallar);
2. crea la aplicación de Heroku si todavía no existe (con el stack «container»);
3. envía el código a Heroku, que construye la imagen a partir de `heroku.yml`, y muestra el log de construcción en directo en GitHub;
4. escribe la dirección de la aplicación en el resumen de la ejecución.

### Paso 1: consigue una clave de API de Heroku

1. En tu navegador, abre https://dashboard.heroku.com e inicia sesión.
2. Abre **Account settings** (desde el menú o desde tu avatar, arriba a la derecha).
3. En la sección **API Key**, toca **Reveal** y copia la clave.

Si prefieres una clave que puedas revocar por separado, abre en su lugar la pestaña **Applications** de Account settings, toca **Create authorization** en **Authorizations**, dale una descripción como «GitHub Actions» y copia el token.

> [!WARNING]
> Esta clave actúa sobre toda tu cuenta de Heroku. No la pegues nunca en un archivo, un commit, una issue o un mensaje: solo en los secretos de GitHub, como se describe a continuación.

### Paso 2: añade el secreto y la variable en GitHub

1. En https://github.com, abre **tu repositorio** (tu fork, o tu propio proyecto) y toca **Settings** (en la barra de pestañas del repositorio; desplázala hacia un lado si hace falta).
2. Abre **Secrets and variables** y luego **Actions**.
3. En la pestaña **Secrets**, toca **New repository secret**:
   - **Name**: `HEROKU_API_KEY`
   - **Secret**: pega la clave
   - toca **Add secret**.
4. En la pestaña **Variables**, toca **New repository variable**:
   - **Name**: `HEROKU_APP_NAME`
   - **Value**: el nombre de tu aplicación de Heroku (la que creaste con el botón, o un nombre nuevo: el workflow crea la aplicación si hace falta)
   - toca **Add variable**.
5. Opcional: una variable `HEROKU_REGION` con el valor `eu` crea la aplicación en Europa. Solo se usa cuando el workflow crea la aplicación.

| Nombre | Tipo | Obligatorio | Valor |
|---|---|---|---|
| `HEROKU_API_KEY` | secreto | sí | tu clave de API de Heroku |
| `HEROKU_APP_NAME` | variable | sí | por ejemplo `my-paquets` |
| `HEROKU_REGION` | variable | no | `us` (por defecto) o `eu` |

### Paso 3: activa Actions en un fork

GitHub desactiva los workflows en un fork nuevo. Abre la pestaña **Actions** de tu fork: si un mensaje dice que los workflows no se están ejecutando en este fork, toca el botón que los activa. (Tus propios repositorios no lo necesitan).

### Paso 4: lanza el primer despliegue

1. Abre la pestaña **Actions** (en la app de GitHub o en el sitio web) y elige **Deploy to Heroku** en la lista de workflows.
2. Toca **Run workflow**, deja la rama `master` y confirma con **Run workflow**.
3. Toca la ejecución que aparece para seguirla. La primera construcción tarda varios minutos.
4. Cuando se ponga en verde, el resumen de la ejecución muestra la dirección de la aplicación: `🚀 Deployed to https://my-paquets.herokuapp.com`.

A partir de ahora, la aplicación se redespliega:

- después de cada rama que el workflow **CI** fusiona en `master` (consulta la sección 5 y [Automatización](automation.md));
- después de cada commit directo en `master` (en el repositorio de Paquets, un commit que solo modifica archivos Markdown no lo desencadena);
- cada vez que lanzas el workflow a mano.

> [!NOTE]
> Mientras falte el secreto o la variable, el workflow se salta el despliegue con un aviso («Heroku is not configured»), en lugar de fallar.

## 5. Modifica el código desde tu teléfono

1. En el sitio web de GitHub, en tu repositorio, abre el archivo que quieres modificar: para tu propio proyecto, `src/main.rs`.
2. Toca el icono del **lápiz** (✏️ «Edit this file») y haz tu cambio. Por ejemplo, cambia el texto de la página de inicio:

   ```rust
   .child(h1().text("Hola desde mi teléfono 👋"))
   ```

3. Toca **Commit changes…**. Elige **Create a new branch for this commit**, dale un nombre como `new-title` y confirma (**Propose changes**). No hace falta que abras la pull request.
4. El workflow **CI** se pone en marcha (síguelo en la pestaña **Actions**): formato, Clippy, pruebas. Cuando todo está en verde, fusiona tu rama en `master` por sí solo y lanza el despliegue. Unos minutos después, la nueva versión está en línea.

Si la CI falla, abre la ejecución fallida: el mensaje del compilador indica el archivo, la línea y la columna. Corrige el archivo en la misma rama: la CI se vuelve a ejecutar. Nada llega a `master`, ni a Heroku, hasta que la CI está en verde.

> [!TIP]
> También puedes hacer commit directamente en `master`: el despliegue empieza enseguida, pero sin esperar a las pruebas. Una rama es más segura. Una rama cuyo nombre empieza por `wip/` se prueba pero nunca se fusiona automáticamente, para el trabajo en curso.

Heroku solo publica las construcciones que tienen éxito: si una construcción falla, la versión anterior sigue en línea.

## 6. Tu propio proyecto, creado con paquets new

Un proyecto creado con `paquets new my-app` contiene los mismos archivos (`app.json`, `heroku.yml`, `Dockerfile` y los dos workflows), así que todos los pasos de esta guía se le aplican.

Crear el proyecto requiere Rust, una sola vez: en cualquier ordenador, o en un entorno de desarrollo en la nube que funcione en un navegador, como [GitHub Codespaces](https://github.com/features/codespaces). Ejecuta `paquets new my-app` y publica la carpeta como un repositorio nuevo de GitHub (por ejemplo `gh repo create my-app --private --source=my-app --push`). A partir de ahí, todo funciona desde tu teléfono:

- **Primer despliegue**: el botón Deploy con la dirección de tu repositorio, `https://heroku.com/deploy?template=https://github.com/<you>/my-app` (para un repositorio privado, usa en su lugar la sección 4: el botón necesita un repositorio público).
- **Despliegues automáticos**: el secreto `HEROKU_API_KEY` y la variable `HEROKU_APP_NAME`, como en la sección 4.
- **Una base de datos**: el add-on Heroku Postgres, como en la sección 2.

El `README.md` del proyecto repite estos pasos.

## 7. Operaciones del día a día

### Redesplegar a mano

**Actions** → **Deploy to Heroku** → **Run workflow** → **Run workflow**. Funciona en la app de GitHub. Útil después de cambiar una config var que necesita una construcción nueva, o para reintentar un despliegue.

### Leer los logs

Panel de Heroku → tu aplicación → **More** (arriba a la derecha) → **View logs**. Ves los mensajes propios de Heroku (arranques, paradas, caídas, una línea por petición de su router) y todo lo que imprime tu aplicación:

```text
app[web.1]: 📦 Paquets · http://localhost:21345
app[web.1]:    database: postgres · security: on (10 guards) · islands: /pkg/my_app.js
app[web.1]: GET /api/todos 200 0.412 ms
heroku[router]: at=info method=GET path="/api/todos" host=my-app.herokuapp.com status=200 ...
```

Los errores de tus handlers que se convierten en un `500` se registran con su causa (`[vitesse] error 500: ...`). La pestaña **Activity** enumera las construcciones y las publicaciones, con un enlace al log de cada construcción.

### Ajustes: config vars

**Settings** → **Reveal Config Vars**. Cada config var es una variable de entorno, que se lee en tu código con `std::env::var("NAME")`. Guardar un cambio reinicia la aplicación. `DATABASE_URL` la gestiona el add-on de Postgres. No definas `PORT`: Heroku lo define en cada arranque.

### Reiniciar, suspensión, tipo de dyno

- **Reiniciar**: **More** → **Restart all dynos**.
- **Suspensión**: un dyno Eco se suspende tras unos 30 minutos sin visitas; la siguiente visita lo despierta en unos segundos. Un dyno Basic nunca se suspende.
- **Tipo de dyno**: en la pestaña **Resources**, edita la línea `web` (icono del lápiz) para cambiar el tipo; el precio cambia en consecuencia.

### Un dominio propio

1. **Settings** → **Domains** → **Add domain**, y escribe tu dominio, por ejemplo `www.example.com`.
2. Heroku te da un **DNS target**. En tu registrador de dominios, crea un registro `CNAME` para `www` que apunte a ese destino. Un dominio raíz (`example.com`) necesita un registrador que admita registros `ALIAS`/`ANAME` o el aplanamiento de CNAME (CNAME flattening).
3. Para tener HTTPS en tu dominio, activa la **Automated Certificate Management** de Heroku en la misma pestaña. No está disponible en todos los tipos de dyno: consulta https://devcenter.heroku.com/articles/automated-certificate-management.

### Dejar de pagar

Elimina los add-ons en **Resources** y la propia aplicación en **Settings** → **Delete app**. Quitar el secreto `HEROKU_API_KEY` detiene los despliegues automáticos.

## 8. Solución de problemas

### El workflow falla antes de construir

| Mensaje | Solución |
|---|---|
| `Heroku is not configured` (un aviso) | Añade el secreto `HEROKU_API_KEY` y la variable `HEROKU_APP_NAME` (sección 4, paso 2). |
| `Heroku refused the API key (HEROKU_API_KEY).` | La clave es incorrecta, ha caducado o se ha revocado: copia una nueva y actualiza el secreto. |
| `This Heroku account cannot use the app "…" (the name may be taken).` | El nombre pertenece a otra cuenta de Heroku: elige otro nombre en `HEROKU_APP_NAME`. |
| falla la creación de la aplicación | Comprueba que tu cuenta de Heroku tiene un método de pago, o crea primero la aplicación con el botón y pon su nombre en `HEROKU_APP_NAME`. |

### La construcción falla

Abre el log de construcción, en GitHub (**Actions**, la ejecución fallida) o en Heroku (**Activity** → build log), y busca la primera línea que empiece por `error`:

| Mensaje | Causa | Solución |
|---|---|---|
| `error[E…]` con un archivo y una línea | un error de compilación de Rust | corrige el código en esa línea y vuelve a hacer commit |
| `no matching package named …` | un nombre de crate mal escrito en `Cargo.toml` | comprueba el nombre en https://crates.io |
| `The Heroku build ended with status "failed".` | la construcción falló en Heroku | lee el log de construcción justo encima, en la misma ejecución |

### «Application error» en el navegador

La construcción tuvo éxito, pero la aplicación no responde. Abre los logs y busca un código de error:

| Código | Significado | Qué hacer |
|---|---|---|
| `H10` App crashed | el programa se detuvo | lee las líneas justo encima: `Error: … database: …` significa que `DATABASE_URL` es incorrecta o que la base de datos es inaccesible; `startup: …` significa que un hook `on_start` falló; `duplicate route` significa que dos rutas chocan |
| `R10` Boot timeout | la aplicación no escuchó en `$PORT` en 60 segundos | Paquets escucha en `$PORT` por sí solo: no llames a `app.port(...)` con un puerto fijo en producción |
| `H14` No web dynos running | ningún dyno ejecuta el proceso `web` | en **Resources**, comprueba que el dyno `web` está activado |
| `H12` Request timeout | una petición tardó más de 30 segundos | haz que el handler sea más rápido, o saca el trabajo de la petición |

Todos los códigos se enumeran en https://devcenter.heroku.com/articles/error-codes.

### Otras sorpresas

- **Todos los visitantes reciben `429 Too Many Requests`**: el escudo cuenta las peticiones por dirección IP. El escudo por defecto lee la dirección real detrás del router de Heroku; si lo reemplazaste con `app.security(...)`, constrúyelo a partir de `paquets::shield()`, que conserva ese ajuste (consulta [Seguridad](security.md#la-ip-del-cliente-detrás-de-un-proxy)).
- **La primera visita es lenta**: un dyno Eco estaba suspendido.
- **Mis datos han desaparecido**: la aplicación no tiene ningún add-on de base de datos (datos en memoria), o usa SQLite. Añade Heroku Postgres (sección 2).
- **Un campo nuevo de un modelo provoca errores `500`**: la tabla existente no se modificó. Consulta [Base de datos](database.md#modificar-una-tabla-más-adelante).
- **Las conexiones WebSocket se cierran al cabo de un minuto**: Heroku cierra las conexiones que permanecen en silencio unos 55 segundos. Envía un ping con regularidad (consulta [WebSocket](websocket.md#en-producción)).
- **HTTP/3 no funciona**: el router de Heroku no reenvía UDP. Deja desactivada la feature `http3` en Heroku.

## Próximos pasos

- [Automatización](automation.md): la CI, la fusión automática en `master` y cómo desactivarlas.
- [Docker](deploy-docker.md): la misma imagen en tu ordenador o en otras plataformas.
- [Configuración](configuration.md): variables de entorno y ajustes.
