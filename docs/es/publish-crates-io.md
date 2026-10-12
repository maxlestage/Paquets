# Publicar en crates.io

[crates.io](https://crates.io) es el registro de paquetes de Rust: cuando Paquets esté publicado allí, cualquiera podrá instalar el comando con `cargo install --git https://github.com/maxlestage/Paquets paquets` y usar la biblioteca con `paquets = { git = "https://github.com/maxlestage/Paquets" }`. Esta página está pensada para quienes mantienen Paquets (o un fork con otro nombre). Explica por qué hay que publicar primero los cuatro paquetes y luego cómo publicarlo todo desde un teléfono, con las releases de GitHub.

## Por qué importa el orden

crates.io solo acepta paquetes cuyas dependencias **también están en crates.io**: un paquete publicado no puede depender de un repositorio Git. Paquets depende de sus cuatro paquetes, así que hay que publicarlos antes que él.

Por eso, hasta que estén todos publicados, el `Cargo.toml` de Paquets da a cada paquete a la vez una `version` y la fuente `git` de su repositorio, por ejemplo:

```toml
[dependencies]
active = { git = "https://github.com/maxlestage/Active" }
memoire-orm = { git = "https://github.com/maxlestage/Memoire", default-features = false, features = ["memory"] }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
vitesse = { git = "https://github.com/maxlestage/Vitesse" }
securite = { git = "https://github.com/maxlestage/Securite", default-features = false }
```

- Mientras desarrollas, Cargo usa la fuente `git`: el código más reciente de cada repositorio.
- Cuando publicas, `cargo publish` quita la fuente `git` y conserva la `version`: el `paquets` publicado depende de `vitesse 0.1` **de crates.io**. Si esa versión no está allí, crates.io rechaza la subida.

## El orden de publicación

Se publican siete crates desde cinco repositorios. Cada repositorio tiene su propio workflow de publicación, que se lanza de la misma forma: publicando una release de GitHub.

| Orden | Crate | Repositorio | Workflow (pestaña Actions) |
|---|---|---|---|
| 1 | `memoire-orm-derive` | [Memoire](https://maxlestage.github.io/Memoire/) | **Publish to crates.io** (publica los dos crates de Memoire, primero el derive) |
| 2 | `memoire-orm` | [Memoire](https://maxlestage.github.io/Memoire/) | la misma ejecución |
| 3 | `securite` | [Securite](https://maxlestage.github.io/Securite/) | **Publish crate** |
| 4 | `vitesse` | [Vitesse](https://maxlestage.github.io/Vitesse/) | **Release** |
| 5 | `active` | [Active](https://maxlestage.github.io/Active/) | **Release** |
| 6 | `paquets-macros` y luego `paquets` | [Paquets](https://maxlestage.github.io/Paquets/) | **Release** |

`memoire-orm` necesita `memoire-orm-derive`, y el workflow de Memoire los publica en ese orden. Los cuatro repositorios de los pasos 1 a 5 no dependen unos de otros: puedes publicarlos en cualquier orden, siempre que todos estén en crates.io antes del paso 6. El workflow de Paquets lo comprueba e indica qué crates faltan.

Cada workflow se salta un crate cuya versión ya está en crates.io, así que volver a ejecutar uno no supone ningún riesgo.

## Configuración inicial (una sola vez): el token de crates.io

Los workflows publican con un token de API de crates.io, guardado como un secreto de GitHub llamado `CARGO_REGISTRY_TOKEN` en **cada uno** de los cinco repositorios.

### Crea el token

1. En el navegador de tu teléfono, abre https://crates.io y toca **Log in with GitHub**.
2. Abre **Account Settings** y comprueba que tu dirección de correo está verificada: si no, crates.io se niega a publicar.
3. Abre **API Tokens** y toca **New Token**.
4. Ponle un nombre, por ejemplo «GitHub Actions», y, si quieres, una fecha de caducidad.
5. Selecciona los permisos (scopes) **publish-new** (necesario la primera vez que se publica un nombre de crate) y **publish-update** (necesario para las versiones nuevas).
6. Toca **Generate Token** y cópialo: crates.io solo lo muestra una vez.

### Guárdalo en cada repositorio

Para cada repositorio (Memoire, Securite, Vitesse, Active, Paquets):

1. En el sitio web de GitHub, abre el repositorio y luego **Settings** → **Secrets and variables** → **Actions**.
2. Toca **New repository secret**.
3. **Name**: `CARGO_REGISTRY_TOKEN`. **Secret**: el token. Toca **Add secret**.

> [!WARNING]
> El token puede publicar crates en tu nombre. Guárdalo solo en los secretos de GitHub: nunca en un archivo, un commit o un mensaje. Si se filtra, revócalo en la página **API Tokens** de crates.io y crea uno nuevo.

## Publica un paquete desde tu teléfono

Los pasos son los mismos en todos los repositorios.

1. Comprueba que la versión de `Cargo.toml` es la que quieres publicar (`0.1.0` para una primera release) y que la CI está en verde en `master`.
2. Abre la página **Releases** del repositorio (`https://github.com/<owner>/<repository>/releases`) y toca **Draft a new release**.
3. Toca **Choose a tag**, escribe la versión con una `v`, por ejemplo `v0.1.0`, y selecciona **Create new tag: v0.1.0 on publish**. Deja `master` como destino.
4. Dale a la release un título (por ejemplo `v0.1.0`) y unas notas; **Generate release notes** las rellena a partir de los cambios fusionados.
5. Toca **Publish release**.

El workflow se pone en marcha en unos segundos. Síguelo en la pestaña **Actions** (la app de GitHub sirve). Cuando se pone en verde, el crate está en crates.io, y docs.rs genera su documentación unos minutos después.

> [!NOTE]
> La etiqueta debe coincidir con la versión de `Cargo.toml`: la etiqueta `v0.1.0` para la versión `0.1.0`. Si no, los workflows de Paquets, Vitesse, Securite y Memoire se detienen con un error; el workflow de Active toma la versión de la etiqueta.

## Publicar Paquets

Cuando los cuatro paquetes estén en crates.io, publica Paquets de la misma forma (etiqueta `v0.1.1`). Su workflow **Release**:

1. comprueba que la etiqueta coincide con la versión de `Cargo.toml`;
2. comprueba que `memoire-orm-derive`, `memoire-orm`, `securite`, `vitesse` y `active` están en crates.io y, si no, se detiene con la lista de los que faltan;
3. ejecuta las pruebas;
4. publica `paquets-macros` (salvo que esa versión ya esté allí) y luego `paquets`;
5. escribe un enlace al crate en el resumen de la ejecución.

Para comprobarlo todo sin publicar, lánzalo a mano: **Actions** → **Release** → **Run workflow**, dejando activada la opción **Only check**. En lugar de la publicación, hace una ejecución de prueba (dry run).

Después:

- `cargo install --git https://github.com/maxlestage/Paquets paquets --locked` instala el comando;
- `paquets = { git = "https://github.com/maxlestage/Paquets" }` funciona en cualquier proyecto. Los proyectos creados con `paquets new` siguen usando el repositorio de GitHub (`version = "0.1", git = ...`) hasta que simplifiques su `Cargo.toml` a `paquets = { git = "https://github.com/maxlestage/Paquets" }`.

## Una versión nueva

crates.io nunca acepta dos veces la misma versión: cada release necesita un número de versión nuevo. Sigue el [versionado semántico](https://semver.org): `0.1.0` → `0.1.1` para las correcciones, `0.1.1` → `0.1.1` para las funcionalidades nuevas o, antes de la 1.0, los cambios incompatibles.

En el repositorio de Paquets cambian cuatro sitios:

| Archivo | Cambio |
|---|---|
| `Cargo.toml` | `version = "0.1.1"` en `[package]` |
| `macros/Cargo.toml` | el mismo `version = "0.1.1"` |
| `Cargo.toml` | la versión fijada de las macros: `paquets-macros = { version = "=0.1.1", path = "macros" }` |
| `CHANGELOG.md` | una sección para la nueva versión |

Desde un teléfono, edita cada archivo en el sitio web de GitHub (icono del lápiz), en una rama nueva: la CI la comprueba y la fusiona en `master`. Después publica una release con la etiqueta nueva (`v0.1.1`). Si la nueva versión de Paquets necesita una nueva versión de uno de los paquetes, publica primero ese paquete.

## Si algo sale mal

| Mensaje | Causa | Solución |
|---|---|---|
| `Publish these packages on crates.io first…` | uno de los cuatro paquetes todavía no está publicado | publícalo desde su repositorio y vuelve a lanzar la release de Paquets |
| `no matching package named 'vitesse' found` (u otro paquete) | lo mismo, visto por `cargo publish` | lo mismo |
| `The tag (…) does not match the version of Cargo.toml` | la etiqueta y la versión no coinciden | elimina la release, corrige la versión o la etiqueta y vuelve a publicar |
| `Add the CARGO_REGISTRY_TOKEN secret` o un `401`/`403` de crates.io | el token falta, es incorrecto, ha caducado o le falta un permiso | crea un token nuevo con publish-new y publish-update y actualiza el secreto |
| `crate version … is already uploaded` | esta versión ya está en crates.io | sube el número de versión |
| `the remote server responded with an error: … verified email` | tu correo de crates.io no está verificado | verifícalo en Account Settings de crates.io |
| `crate name is already taken` | otra persona tiene este nombre en crates.io | en un fork, publica con otro nombre |

> [!WARNING]
> Una versión publicada no se puede modificar ni eliminar nunca. Si una release está rota, publica una versión corregida y luego retira (*yank*) la rota con `cargo yank paquets --version 0.1.1` (desde un ordenador): los proyectos que ya la usan siguen funcionando, y los proyectos nuevos no la seleccionarán.

## Próximos pasos

- [Automatización](automation.md): la CI que comprueba los paquetes antes de cada release.
- [Los cuatro paquetes](packages.md): lo que hace cada paquete.
- [Instalación](installation.md): lo que hacen los usuarios cuando Paquets está en crates.io.
