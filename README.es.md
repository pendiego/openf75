<div align="center">

<img src="app/app-icon.svg" width="96" height="96" alt="">

# OpenF75

**Configurador de código abierto para el teclado AULA F75.**<br>
Iluminación, colores por tecla, remapeo, macros y perfiles, sin el software del fabricante.

[![Descargar](https://img.shields.io/badge/descargar-releases-e8e6df)](https://github.com/pendiego/openf75/releases/latest)
![Windows](https://img.shields.io/badge/Windows-probado-2ea44f?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-experimental-f0ad4e?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-experimental-f0ad4e?logo=apple&logoColor=white)
[![Licencia MIT](https://img.shields.io/badge/licencia-MIT-blue)](#licencia)

[**Descargar**](https://github.com/pendiego/openf75/releases/latest) · [Funciones](#funciones) · [Instalación](#instalación) · [CLI](#línea-de-comandos) · [Limitaciones](#limitaciones)

[English](README.md) · [Português](README.pt-BR.md) · **Español** · [Français](README.fr.md) · [Deutsch](README.de.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md)

<br>

<img src="docs/screenshots/iluminacao.png" alt="Pestaña Iluminación: modelo 3D del F75 con el efecto Rainbow y la galería de efectos con vistas previas en vivo" width="860">

</div>

## Funciones

| Función | Qué hace |
|---|---|
| **Iluminación** | Los 20 efectos del teclado con vista previa en vivo sobre un modelo 3D, brillo, velocidad y color |
| **Por tecla** | Pinta las teclas una a una o por área y grábalas en el teclado |
| **Audio reactivo** | Las teclas se convierten en un ecualizador del sonido que suena en el computador |
| **Teclas** | Remapea las capas Base y Fn: teclas, multimedia, macros o desactivar |
| **Macros** | Graba secuencias con retardo por evento, directamente en la memoria del teclado |
| **Energía** | Tiempo de suspensión, debounce y función de la rueda (volumen o iluminación) |
| **Perfiles** | Guarda, aplica, importa y exporta configuraciones; copia de seguridad automática en la primera conexión |
| **Paleta de comandos** | <kbd>Ctrl</kbd> <kbd>K</kbd> (<kbd>⌘</kbd> <kbd>K</kbd> en macOS) para ir a cualquier pestaña o acción |

Tema claro y oscuro y actualización automática.

<table>
  <tr>
    <td><img src="docs/screenshots/teclas.png" alt="Pestaña Teclas: remapeo de la capa Base"></td>
    <td><img src="docs/screenshots/macros.png" alt="Pestaña Macros: editor de eventos con tecla, acción y retardo"></td>
  </tr>
  <tr>
    <td align="center"><sub>Remapeo de las capas</sub></td>
    <td align="center"><sub>Editor de macros</sub></td>
  </tr>
</table>

### Idiomas (en desarrollo)

La interfaz de la app estará disponible en portugués (Brasil), inglés, español, francés, alemán, japonés y chino simplificado en la próxima versión. Al abrirla por primera vez, la app seguirá el idioma del sistema y usará el inglés si este no está en la lista. Para cambiarlo, usa el selector en la barra de estado inferior de la ventana, junto a la versión del firmware, o busca el idioma en la paleta de comandos; la elección quedará guardada.

La CLI y los mensajes de error de Rust seguirán escritos en portugués; la app mostrará los errores conocidos en el idioma elegido.

### Compatibilidad

| | Cable USB `258A:010C` | Receptor 2.4 GHz `3554:FA09` | Bluetooth |
|---|:---:|:---:|:---:|
| AULA F75 | ✅ | ✅ | ❌ |
| AULA F75 KR | ❌ | ❌ | ❌ |

| Sistema | Paquete | Estado |
|---|---|---|
| Windows 10/11 | Instalador NSIS | Probado con el teclado |
| Linux (x64) | `.deb` y AppImage | Validación en CI pendiente, sin prueba en hardware |
| macOS 13+ (Intel y Apple Silicon) | `.dmg` | Validación en CI pendiente, sin prueba en hardware |

## Instalación

Descarga el paquete de tu sistema en la [página de releases](https://github.com/pendiego/openf75/releases/latest).

> [!IMPORTANT]
> En Windows, **cierra el software de AULA (`OemDrv.exe`)** antes de grabar: los dos compiten por la misma interfaz del teclado. OpenF75 avisa cuando está abierto.

<details>
<summary><b>Windows</b></summary>

Ejecuta el instalador. No hace falta ningún driver ni regla extra. La app se actualiza sola.

</details>

<details>
<summary><b>Linux</b></summary>

- **`.deb`**: instala la regla udev que libera el teclado para el usuario conectado. Para actualizar, instala el `.deb` nuevo.
- **AppImage**: se actualiza sola, pero la regla udev no viene incluida. Instálala una vez:

  ```sh
  sudo cp packaging/linux/70-openf75.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  ```

La regla usa `TAG+="uaccess"`, que depende de systemd-logind. Si la app no logra abrir el teclado, el mensaje de error apunta a ella.

El audio reactivo captura el monitor de la salida predeterminada (PulseAudio o PipeWire).

</details>

<details>
<summary><b>macOS</b></summary>

La app tiene firma ad-hoc y no está notarizada (no hay certificado de Apple). La primera vez, haz clic derecho en la app y elige **Abrir**. Si macOS sigue bloqueándola:

```sh
xattr -dr com.apple.quarantine /Applications/OpenF75.app
```

macOS no permite que las apps capturen el sonido del sistema. Para el audio reactivo, instala un dispositivo de loopback como [BlackHole](https://github.com/ExistentialAudio/BlackHole) y envía la salida hacia él. macOS pedirá permiso de micrófono: por ahí es como la app lee el loopback.

</details>

Las macros, perfiles y copias de seguridad se guardan en:

| Windows | macOS | Linux |
|---|---|---|
| `%APPDATA%\openf75\` | `~/Library/Application Support/openf75/` | `$XDG_CONFIG_HOME/openf75/` o `~/.config/openf75/` |

## Línea de comandos

La CLI `openf75` hace lo mismo que la app, para scripts. Necesita Rust estable; en Linux instala antes `libudev-dev`, `libasound2-dev` y `pkg-config`.

```sh
cargo install --path crates/openf75-cli   # instala el comando openf75 en el PATH

openf75 status                                   # solo lectura
openf75 dump --out backup.json                   # solo lectura: hazlo antes de grabar
openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
openf75 perkey --all 000000 W=FF0000 A=FF0000
openf75 remap base CapsLock key:0x29
openf75 macro-text F12 "ola"
openf75 sleep 300
openf75 debounce 1
openf75 restore backup.json
```

## Desarrollo

```sh
cd app
npm install
npm run tauri dev            # app en modo desarrollo
npm run check                # svelte-check
cargo test -p openf75        # pruebas del protocolo
```

Build del instalador: `npm run tauri build` en Windows; en Linux añade `-- --bundles deb,appimage`; en macOS, `-- --target universal-apple-darwin --bundles app,dmg`.

| Ruta | Contenido |
|---|---|
| [`crates/openf75`](crates/openf75) | Biblioteca Rust: protocolo HID, dispositivo, audio, archivos |
| [`crates/openf75-cli`](crates/openf75-cli) | CLI `openf75` |
| [`app/`](app) | App de escritorio (Tauri 2 + Svelte 5) |
| [`docs/PROTOCOL.md`](docs/PROTOCOL.md) | Protocolo del F75, recuperado por ingeniería inversa |

<details>
<summary><b>Publicar una versión</b></summary>

1. Aumenta `version` en `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml` y `app/package.json`; actualiza `app/package-lock.json` (`npm install --package-lock-only` en `app/`) y `Cargo.lock` (`cargo check -p app`).
2. Haz el commit, crea el tag y envíalo: `git tag vX.Y.Z` y `git push origin vX.Y.Z`.
3. El workflow [`release.yml`](.github/workflows/release.yml) compila en `windows-latest`, `ubuntu-22.04` y `macos-latest`, uno a la vez (`max-parallel: 1`), porque `tauri-action` combina el `latest.json` que lee el actualizador.

La app busca una versión nueva al abrirse y muestra el botón **Actualizar**. La actualización se verifica con la firma del updater. En Linux, solo el AppImage se actualiza solo.

Secretos del repositorio (Settings → Secrets and variables → Actions):
- `TAURI_SIGNING_PRIVATE_KEY`: contenido de `%USERPROFILE%\.tauri\openf75.key`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: contenido de `%USERPROFILE%\.tauri\openf75.key.password`.

> [!CAUTION]
> La clave privada nunca entra en el repositorio. Si se pierde, las apps instaladas dejan de aceptar actualizaciones: habrá que reinstalar a mano y cambiar el `pubkey` de `tauri.conf.json`.

</details>

## Limitaciones

**Teclado y conexión**

- **Solo AULA F75.** El F75 KR (distribución distinta) y el Bluetooth no son compatibles.
- **Cable USB:** el protocolo se implementó a partir de ingeniería inversa y solo tiene pruebas unitarias; nunca se ha probado con el teclado.
- **Receptor 2.4 GHz:**
  - La paleta de colores (`0x49`) no responde: el color mostrado es el predeterminado, las copias de seguridad no incluyen la paleta, la restauración la omite y los colores personalizados se rechazan (usa los slots 0-6 o el cable).
  - Los modos reactivos al audio y en tiempo real funcionan a unos 3 fps.
  - Una capa de teclas llega con 504 de sus 512 bytes; el resto se rellena con ceros.
  - La primera respuesta puede tardar unos 400 ms.
  - Las escrituras fallan con el teclado en suspensión o apagado; pulsa una tecla para despertarlo.
- **Tiempo de suspensión:** solo se aplica en modo inalámbrico.
- **Rueda:** el byte que elige volumen o iluminación se encontró a través del receptor; no se ha confirmado por cable.
- **Firmware:** OpenF75 no actualiza el firmware. El driver de AULA tampoco lo escribe: llama a un actualizador externo que no está en el paquete actual del F75.
- **Macros:** 4096 bytes en total, incluidos los nombres; nombres de hasta 30 caracteres; retardos de hasta 1.048.575 ms (unos 17 min) por evento.

**Sistemas**

- **Windows:** cierra el software de AULA (`OemDrv.exe`) antes de escribir. Solo se ofrece un instalador NSIS x64.
- **Linux y macOS no se han probado con el teclado,** y los paquetes aún no han pasado la CI. Ejecuta `openf75 status` y `openf75 dump` antes de escribir nada en esos sistemas.
- **Linux:** solo x64. El acceso al teclado depende de la regla de udev, que usa `TAG+="uaccess"` y necesita systemd-logind; el AppImage no instala la regla. Solo el AppImage se actualiza solo; el `.deb` se actualiza a mano. El modo reactivo al audio necesita PulseAudio o PipeWire.
- **macOS:** solo 13 o posterior. La app no está notarizada, así que Gatekeeper bloquea el primer inicio. El sistema no permite capturar su sonido: el modo reactivo al audio necesita un dispositivo de loopback (BlackHole y similares) y permiso de micrófono.

**Idioma**

- Las traducciones de la interfaz aún están en desarrollo. La CLI y los mensajes de error que vienen de Rust están en portugués; la app traduce solo los errores que conoce.

## Licencia

[MIT](LICENSE). Proyecto independiente, sin afiliación con AULA. Ningún archivo del fabricante forma parte del repositorio.
