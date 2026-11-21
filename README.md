<div align="center">

<img src="app/app-icon.svg" width="96" height="96" alt="">

# OpenF75

**Open source configurator for the AULA F75 keyboard.**<br>
Lighting, per-key colors, key remapping, macros and profiles, without the manufacturer's software.

[![Download](https://img.shields.io/badge/download-releases-e8e6df)](https://github.com/pendiego/openf75/releases/latest)
![Windows](https://img.shields.io/badge/Windows-tested-2ea44f?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-experimental-f0ad4e?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-experimental-f0ad4e?logo=apple&logoColor=white)
[![MIT License](https://img.shields.io/badge/license-MIT-blue)](#license)

[**Download**](https://github.com/pendiego/openf75/releases/latest) · [Features](#features) · [Installation](#installation) · [CLI](#command-line) · [Limitations](#limitations)

**English** · [Português](README.pt-BR.md) · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md)

<br>

<img src="docs/screenshots/iluminacao.png" alt="Lighting tab: 3D model of the F75 with the Rainbow effect and the effects gallery with live previews" width="860">

</div>

## Features

| Feature | What it does |
|---|---|
| **Lighting** | The keyboard's 20 effects with a live preview on a 3D model, brightness, speed and color |
| **Per key** | Paint keys one by one or by area and write them to the keyboard |
| **Audio reactive** | The keys become an equalizer for the sound playing on your computer |
| **Keys** | Remap the Base and Fn layers: keys, media, macros or disable |
| **Macros** | Record sequences with a delay per event, stored directly in the keyboard's memory |
| **Power** | Sleep time, debounce and wheel function (volume or lighting) |
| **Profiles** | Save, apply, import and export settings; automatic backup on first connection |
| **Command palette** | <kbd>Ctrl</kbd> <kbd>K</kbd> (<kbd>⌘</kbd> <kbd>K</kbd> on macOS) to jump to any tab or action |

Light and dark theme and automatic updates.

<table>
  <tr>
    <td><img src="docs/screenshots/teclas.png" alt="Keys tab: Base layer remapping"></td>
    <td><img src="docs/screenshots/macros.png" alt="Macros tab: event editor with key, action and delay"></td>
  </tr>
  <tr>
    <td align="center"><sub>Layer remapping</sub></td>
    <td align="center"><sub>Macro editor</sub></td>
  </tr>
</table>

### Languages (in development)

The app interface is being translated into Portuguese (Brazil), English, Spanish, French, German, Japanese and Simplified Chinese for the next release, which is still in development. On first launch the app will follow the system language and fall back to English when it is not in the list. To switch, there will be a selector in the status bar at the bottom of the window (next to the firmware version) and also the command palette; your choice will be saved.

The CLI and the Rust error messages are written in Portuguese; the app will show known errors in the selected language.

### Compatibility

| | USB cable `258A:010C` | 2.4 GHz receiver `3554:FA09` | Bluetooth |
|---|:---:|:---:|:---:|
| AULA F75 | ✅ | ✅ | ❌ |
| AULA F75 KR | ❌ | ❌ | ❌ |

| System | Package | Status |
|---|---|---|
| Windows 10/11 | NSIS installer | Tested with the keyboard |
| Linux (x64) | `.deb` and AppImage | CI validation pending, not tested on hardware |
| macOS 13+ (Intel and Apple Silicon) | `.dmg` | CI validation pending, not tested on hardware |

## Installation

Download the package for your system from the [releases page](https://github.com/pendiego/openf75/releases/latest).

> [!IMPORTANT]
> On Windows, **close the AULA software (`OemDrv.exe`)** before writing to the keyboard: both programs compete for the same keyboard interface. OpenF75 warns you when it is open.

<details>
<summary><b>Windows</b></summary>

Run the installer. There is no driver or extra rule. The app updates itself.

</details>

<details>
<summary><b>Linux</b></summary>

- **`.deb`**: installs the udev rule that gives the logged-in user access to the keyboard. To update, install the new `.deb`.
- **AppImage**: updates itself, but the udev rule does not come with it. Install it once:

  ```sh
  sudo cp packaging/linux/70-openf75.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  ```

The rule uses `TAG+="uaccess"`, which depends on systemd-logind. If the app cannot open the keyboard, the error message points to it.

Audio reactive mode captures the monitor of the default output (PulseAudio or PipeWire).

</details>

<details>
<summary><b>macOS</b></summary>

The app is ad-hoc signed and not notarized (there is no Apple certificate). The first time, right-click the app and choose **Open**. If macOS keeps blocking it:

```sh
xattr -dr com.apple.quarantine /Applications/OpenF75.app
```

macOS does not let apps capture system audio. For audio reactive mode, install a loopback device such as [BlackHole](https://github.com/ExistentialAudio/BlackHole) and send the output to it. macOS will ask for microphone permission: that is how the app reads the loopback.

</details>

Macros, profiles and backups are stored in:

| Windows | macOS | Linux |
|---|---|---|
| `%APPDATA%\openf75\` | `~/Library/Application Support/openf75/` | `$XDG_CONFIG_HOME/openf75/` or `~/.config/openf75/` |

## Command line

The `openf75` CLI does the same as the app, for scripts. It needs stable Rust; on Linux, install `libudev-dev`, `libasound2-dev` and `pkg-config` first.

```sh
cargo install --path crates/openf75-cli   # installs the openf75 command on PATH

openf75 status                                   # read-only
openf75 dump --out backup.json                   # read-only: do this before writing
openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
openf75 perkey --all 000000 W=FF0000 A=FF0000
openf75 remap base CapsLock key:0x29
openf75 macro-text F12 "ola"
openf75 sleep 300
openf75 debounce 1
openf75 restore backup.json
```

## Development

```sh
cd app
npm install
npm run tauri dev            # app in dev mode
npm run check                # svelte-check
cargo test -p openf75        # protocol tests
```

Installer build: `npm run tauri build` on Windows; on Linux append `-- --bundles deb,appimage`; on macOS, `-- --target universal-apple-darwin --bundles app,dmg`.

| Path | Contents |
|---|---|
| [`crates/openf75`](crates/openf75) | Rust library: HID protocol, device, audio, files |
| [`crates/openf75-cli`](crates/openf75-cli) | `openf75` CLI |
| [`app/`](app) | Desktop app (Tauri 2 + Svelte 5) |
| [`docs/PROTOCOL.md`](docs/PROTOCOL.md) | F75 protocol, recovered by reverse engineering |

<details>
<summary><b>Publishing a release</b></summary>

1. Increase `version` in `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml` and `app/package.json`; update `app/package-lock.json` (`npm install --package-lock-only` in `app/`) and `Cargo.lock` (`cargo check -p app`).
2. Commit, create the tag and push: `git tag vX.Y.Z` and `git push origin vX.Y.Z`.
3. The [`release.yml`](.github/workflows/release.yml) workflow builds on `windows-latest`, `ubuntu-22.04` and `macos-latest`, one at a time (`max-parallel: 1`), because `tauri-action` merges the `latest.json` that the updater reads.

The app looks for a new version on startup and shows the **Update** button. The update is verified by the updater signature. On Linux, only the AppImage updates itself.

Repository secrets (Settings → Secrets and variables → Actions):
- `TAURI_SIGNING_PRIVATE_KEY`: contents of `%USERPROFILE%\.tauri\openf75.key`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: contents of `%USERPROFILE%\.tauri\openf75.key.password`.

> [!CAUTION]
> The private key never goes into the repository. If it is lost, installed apps stop accepting updates: you will have to reinstall by hand and replace the `pubkey` in `tauri.conf.json`.

</details>

## Limitations

**Keyboard and connection**

- **AULA F75 only.** The F75 KR (different layout) and Bluetooth are not supported.
- **USB cable:** the protocol was implemented from reverse engineering and only has unit tests; it has never been exercised with the keyboard.
- **2.4 GHz receiver:**
  - The color palette (`0x49`) does not respond: the displayed color is the default, backups do not include the palette, restore skips it, and custom colors are rejected (use slots 0-6 or the cable).
  - Audio reactive and real-time modes run at about 3 fps.
  - A key layer arrives with 504 of its 512 bytes; the rest is filled with zeros.
  - The first reply can take about 400 ms.
  - Writes fail while the keyboard is asleep or powered off; press a key to wake it.
- **Sleep timer:** it only applies in wireless mode.
- **Wheel:** the byte that picks volume or lighting was found through the receiver; it has not been confirmed over the cable.
- **Firmware:** OpenF75 does not update firmware. The AULA driver does not write it either: it calls an external updater that is not in the current F75 package.
- **Macros:** 4096 bytes in total, names included; names up to 30 characters; delays up to 1,048,575 ms (about 17 min) per event.

**Systems**

- **Windows:** close the AULA software (`OemDrv.exe`) before writing. Only an x64 NSIS installer is provided.
- **Linux and macOS have not been tested with the keyboard,** and the packages have not passed CI yet. Run `openf75 status` and `openf75 dump` before writing anything on those systems.
- **Linux:** x64 only. Keyboard access depends on the udev rule, which uses `TAG+="uaccess"` and needs systemd-logind; the AppImage does not install the rule. Only the AppImage updates itself; the `.deb` is updated by hand. Audio reactive needs PulseAudio or PipeWire.
- **macOS:** 13 or later only. The app is not notarized, so Gatekeeper blocks the first launch. The system does not allow capturing its sound: audio reactive needs a loopback device (BlackHole and similar) and microphone permission.

**Language**

- The interface translations are still in development. The CLI and error messages coming from Rust are in Portuguese; the app translates only the errors it knows.

## License

[MIT](LICENSE). Independent project, not affiliated with AULA. No file from the manufacturer is part of the repository.
