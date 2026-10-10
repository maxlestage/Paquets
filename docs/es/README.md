# Documentación de Paquets

Todo lo que necesitas para crear aplicaciones web full-stack con Paquets 📦, el framework full-stack más sencillo para Rust: desde tu primera página hasta un despliegue hecho desde tu teléfono. Estas páginas también están en el sitio web: https://maxlestage.github.io/Paquets/

## Primeros pasos

- [Introducción](introduction.md): qué es Paquets, qué hace por ti y cuáles son sus límites.
- [Instalación](installation.md): Rust, el comando `paquets` y las herramientas opcionales de WebAssembly.
- [Inicio rápido](quick-start.md): crea paso a paso una pequeña aplicación con páginas, un formulario, una base de datos y una prueba.
- [Estructura del proyecto](project-structure.md): todos los archivos que crea `paquets new`, y cómo organizar un proyecto que crece.

## Lo esencial

- [Enrutamiento](routing.md): rutas, parámetros, routers, middleware, la página 404 y los archivos estáticos.
- [Peticiones y respuestas](requests-responses.md): query strings, cabeceras, JSON y formularios; texto, JSON, redirecciones, cookies y archivos.
- [Páginas](pages.md): vistas, layouts, el tema integrado, las etiquetas SEO y la Content-Security-Policy.
- [Base de datos](database.md): modelos, elección de la base de datos, consultas, transacciones, migraciones y SQL sin procesar.
- [API REST](rest-api.md): una API JSON completa para un modelo en una sola línea, y cómo protegerla o ampliarla.
- [Seguridad](security.md): el escudo delante de cada petición, cómo modificarlo y la autenticación.

## Más allá

- [Islas](islands.md): partes interactivas de una página, escritas en Rust y ejecutadas en el navegador como WebAssembly.
- [WebSocket](websocket.md): conexiones bidireccionales, desde un servidor de eco hasta una sala de chat.
- [Pruebas](testing.md): prueba toda la aplicación en memoria con `app.test()`, sin abrir ningún puerto.
- [Errores](errors.md): `Result`, el operador `?`, el estado de cada error y los formatos de error personalizados.
- [Configuración](configuration.md): variables de entorno, ajustes de la aplicación, estado compartido y cómo funciona el servidor.
- [Rendimiento](performance.md): los benchmarks, cómo usa el servidor tu CPU y consejos para seguir siendo rápido.

## Desplegar

- [Desplegar en Heroku desde el móvil](deploy-heroku-mobile.md): el botón Deploy, una base de datos, despliegues automáticos, logs y dominios, todo sin ordenador.
- [Desplegar con Docker](deploy-docker.md): la imagen de producción, los volúmenes, docker compose y otras plataformas.
- [Publicar en crates.io](publish-crates-io.md): el orden de publicación de los paquetes, y las releases desde un teléfono.
- [Automatización](automation.md): pruebas en cada push, la fusión automática en `master`, los despliegues y cómo desactivarlos.

## Referencia

- [Herramienta de línea de comandos](cli.md): `paquets new`, `dev`, `build` y `doctor`.
- [Chuleta](cheatsheet.md): toda la API en una sola página.
- [Los cuatro paquetes](packages.md): vitesse, securite, memoire-orm y active, y cómo los usa Paquets.
- [Preguntas frecuentes](faq.md): dudas habituales y sus respuestas.

## Más recursos

- Referencia de la API generada a partir del código fuente: https://docs.rs/paquets
- Código fuente, issues y ejemplos: https://github.com/maxlestage/Paquets
- Los cuatro paquetes: [vitesse](https://github.com/maxlestage/Vitesse), [securite](https://github.com/maxlestage/Securite), [memoire-orm](https://github.com/maxlestage/Memoire), [active](https://github.com/maxlestage/Active)
- También disponible en [inglés](../en/README.md) y en [francés](../fr/README.md).
